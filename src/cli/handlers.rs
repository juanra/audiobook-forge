//! CLI command handlers

use crate::cli::commands::{BuildArgs, ConfigCommands, OrganizeArgs, MetadataCommands, MatchArgs};
use crate::core::{Analyzer, BatchProcessor, Organizer, RetryConfig, Scanner};
use crate::models::{Config, AudibleRegion, CurrentMetadata, MetadataSource};
use crate::utils::{ConfigManager, DependencyChecker, AudibleCache, scoring, extraction};
use crate::audio::{AacEncoder, AudibleClient, detect_asin};
use crate::ui::{prompt_match_selection, prompt_manual_metadata, prompt_custom_search, UserChoice};
use anyhow::{Context, Result, bail};
use console::style;
use std::path::PathBuf;
use std::str::FromStr;

/// Resolve which AAC encoder to use based on config (handles backward compatibility)
fn resolve_encoder(config: &Config, cli_override: Option<&str>) -> AacEncoder {
    // CLI argument takes highest priority
    if let Some(encoder_str) = cli_override {
        if let Some(encoder) = AacEncoder::from_str(encoder_str) {
            tracing::info!("Using encoder from CLI argument: {}", encoder.name());
            return encoder;
        } else {
            tracing::warn!("Unknown encoder '{}', falling back to auto-detection", encoder_str);
        }
    }

    // Handle backward compatibility with old use_apple_silicon_encoder field
    if let Some(use_apple_silicon) = config.advanced.use_apple_silicon_encoder {
        let encoder = if use_apple_silicon {
            AacEncoder::AppleSilicon
        } else {
            AacEncoder::Native
        };
        tracing::info!("Using encoder from legacy config: {}", encoder.name());
        return encoder;
    }

    // Use new aac_encoder field
    match config.advanced.aac_encoder.to_lowercase().as_str() {
        "auto" => {
            let encoder = crate::audio::get_encoder();
            tracing::info!("Auto-detected encoder: {}", encoder.name());
            encoder
        }
        encoder_str => {
            if let Some(encoder) = AacEncoder::from_str(encoder_str) {
                tracing::info!("Using configured encoder: {}", encoder.name());
                encoder
            } else {
                tracing::warn!(
                    "Unknown encoder '{}' in config, falling back to auto-detection",
                    encoder_str
                );
                let encoder = crate::audio::get_encoder();
                tracing::info!("Auto-detected encoder: {}", encoder.name());
                encoder
            }
        }
    }
}

/// Try to detect if current directory is an audiobook folder
fn try_detect_current_as_audiobook() -> Result<Option<PathBuf>> {
    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?;

    // Safety check: Don't auto-detect from filesystem root
    if current_dir.parent().is_none() {
        return Ok(None);
    }

    // Check for MP3 files in current directory
    let entries = std::fs::read_dir(&current_dir)
        .context("Failed to read current directory")?;

    let mp3_count = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("mp3") || ext.eq_ignore_ascii_case("m4a"))
                .unwrap_or(false)
        })
        .count();

    // Require at least 1 MP3 file to consider it an audiobook (BookCase A or B)
    if mp3_count >= 1 {
        Ok(Some(current_dir))
    } else {
        Ok(None)
    }
}

/// Check if a directory is itself an audiobook folder
fn is_audiobook_folder(path: &std::path::Path) -> Result<bool> {
    if !path.is_dir() {
        return Ok(false);
    }

    let entries = std::fs::read_dir(path)
        .context("Failed to read directory")?;

    let audio_count = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| {
                    ext.eq_ignore_ascii_case("mp3") ||
                    ext.eq_ignore_ascii_case("m4a") ||
                    ext.eq_ignore_ascii_case("m4b")
                })
                .unwrap_or(false)
        })
        .count();

    Ok(audio_count >= 1)
}

/// Handle the build command
pub async fn handle_build(args: BuildArgs, config: Config) -> Result<()> {
    // Determine root directory (CLI arg > config > auto-detect > error)
    let (root, auto_detected) = if let Some(root_path) = args.root.or(config.directories.source.clone()) {
        // Check if root itself is an audiobook folder
        if is_audiobook_folder(&root_path)? {
            println!(
                "{} Detected audiobook folder (not library): {}",
                style("→").cyan(),
                style(root_path.display()).yellow()
            );
            (root_path, true)
        } else {
            (root_path, false)
        }
    } else {
        // Try auto-detecting current directory
        if let Some(current) = try_detect_current_as_audiobook()? {
            println!(
                "{} Auto-detected audiobook folder: {}",
                style("→").cyan(),
                style(current.display()).yellow()
            );
            (current, true)
        } else {
            anyhow::bail!(
                "No root directory specified. Use --root, configure directories.source, or run from inside an audiobook folder"
            );
        }
    };

    if !auto_detected {
        println!(
            "{} Scanning audiobooks in: {}",
            style("→").cyan(),
            style(root.display()).yellow()
        );
    }

    // Scan for audiobooks
    let scanner = Scanner::from_config(&config);
    let mut book_folders = if auto_detected {
        // Auto-detect mode: treat current dir as single book
        vec![scanner.scan_single_directory(&root)?]
    } else {
        // Normal mode: scan for multiple books
        scanner
            .scan_directory(&root)
            .context("Failed to scan directory")?
    };

    if book_folders.is_empty() {
        println!("{} No audiobooks found", style("✗").red());
        return Ok(());
    }

    println!(
        "{} Found {} audiobook(s)",
        style("✓").green(),
        style(book_folders.len()).cyan()
    );

    // Filter by skip_existing if configured
    if config.processing.skip_existing && !args.force {
        book_folders.retain(|b| b.m4b_files.is_empty());
        println!(
            "{} After filtering existing: {} audiobook(s)",
            style("→").cyan(),
            style(book_folders.len()).cyan()
        );
    }

    if book_folders.is_empty() {
        println!(
            "{} All audiobooks already processed (use --force to reprocess)",
            style("ℹ").blue()
        );
        return Ok(());
    }

    // Dry run mode
    if args.dry_run {
        println!("\n{} DRY RUN MODE - No changes will be made\n", style("ℹ").blue());
        for book in &book_folders {
            println!(
                "  {} {} ({} files, {:.1} min)",
                style("→").cyan(),
                style(&book.name).yellow(),
                book.mp3_files.len(),
                book.get_total_duration() / 60.0
            );
        }
        return Ok(());
    }

    // Analyze all books
    println!("\n{} Analyzing tracks...", style("→").cyan());
    let analyzer_workers = args.parallel.unwrap_or(config.processing.parallel_workers);
    let analyzer = Analyzer::with_workers(analyzer_workers as usize)?;

    for book in &mut book_folders {
        analyzer
            .analyze_book_folder(book)
            .await
            .with_context(|| format!("Failed to analyze {}", book.name))?;
    }

    println!("{} Analysis complete", style("✓").green());

    // Fetch Audible metadata if enabled
    if args.fetch_audible || config.metadata.audible.enabled {
        println!("\n{} Fetching Audible metadata...", style("→").cyan());

        let audible_region = args.audible_region
            .as_deref()
            .or(Some(&config.metadata.audible.region))
            .and_then(|r| AudibleRegion::from_str(r).ok())
            .unwrap_or(AudibleRegion::US);

        let retry_config = crate::core::RetryConfig::with_settings(
            config.metadata.audible.api_max_retries as usize,
            std::time::Duration::from_secs(config.metadata.audible.api_retry_delay_secs),
            std::time::Duration::from_secs(config.metadata.audible.api_max_retry_delay_secs),
            2.0,
        );
        let client = AudibleClient::with_config(
            audible_region,
            config.metadata.audible.rate_limit_per_minute,
            retry_config,
        )?;
        let cache = AudibleCache::with_ttl_hours(config.metadata.audible.cache_duration_hours)?;

        for book in &mut book_folders {
            // Try ASIN detection first
            if let Some(asin) = detect_asin(&book.name) {
                tracing::debug!("Detected ASIN {} in folder: {}", asin, book.name);
                book.detected_asin = Some(asin.clone());

                // Try cache first
                match cache.get(&asin).await {
                    Some(cached) => {
                        book.audible_metadata = Some(cached);
                        println!("  {} {} (ASIN: {}, cached)", style("✓").green(), book.name, asin);
                    }
                    None => {
                        // Fetch from API
                        match client.fetch_by_asin(&asin).await {
                            Ok(metadata) => {
                                // Cache the result
                                let _ = cache.set(&asin, &metadata).await;
                                book.audible_metadata = Some(metadata);
                                println!("  {} {} (ASIN: {})", style("✓").green(), book.name, asin);
                            }
                            Err(e) => {
                                tracing::warn!("Failed to fetch metadata for {}: {:?}", book.name, e);
                                println!("  {} {} - fetch failed", style("⚠").yellow(), book.name);
                            }
                        }
                    }
                }
            } else if args.audible_auto_match || config.metadata.audible.auto_match {
                // Try auto-matching by title
                tracing::debug!("Attempting auto-match for: {}", book.name);

                match client.search(Some(&book.name), None).await {
                    Ok(results) if !results.is_empty() => {
                        let asin = &results[0].asin;
                        tracing::debug!("Auto-matched {} to ASIN: {}", book.name, asin);
                        book.detected_asin = Some(asin.clone());

                        // Try cache first
                        match cache.get(asin).await {
                            Some(cached) => {
                                book.audible_metadata = Some(cached);
                                println!("  {} {} (matched: {}, cached)", style("✓").green(), book.name, asin);
                            }
                            None => {
                                // Fetch from API
                                match client.fetch_by_asin(asin).await {
                                    Ok(metadata) => {
                                        // Cache the result
                                        let _ = cache.set(asin, &metadata).await;
                                        book.audible_metadata = Some(metadata);
                                        println!("  {} {} (matched: {})", style("✓").green(), book.name, asin);
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to fetch metadata after match for {}: {:?}", book.name, e);
                                        println!("  {} {} - fetch failed", style("⚠").yellow(), book.name);
                                    }
                                }
                            }
                        }
                    }
                    Ok(_) => {
                        tracing::debug!("No Audible match found for: {}", book.name);
                        println!("  {} {} - no match found", style("○").dim(), book.name);
                    }
                    Err(e) => {
                        tracing::warn!("Search failed for {}: {:?}", book.name, e);
                        println!("  {} {} - search failed", style("⚠").yellow(), book.name);
                    }
                }
            } else {
                tracing::debug!("No ASIN detected and auto-match disabled for: {}", book.name);
            }
        }

        let fetched_count = book_folders.iter().filter(|b| b.audible_metadata.is_some()).count();
        println!("{} Fetched metadata for {}/{} books",
            style("✓").green(),
            style(fetched_count).cyan(),
            book_folders.len()
        );
    }

    // Determine output directory
    let output_dir = if auto_detected {
        // When auto-detected, default to current directory
        args.out.unwrap_or(root.clone())
    } else {
        // Normal mode: respect config
        args.out.or_else(|| {
            if config.directories.output == "same_as_source" {
                Some(root.clone())
            } else {
                Some(PathBuf::from(&config.directories.output))
            }
        }).context("No output directory specified")?
    };

    // Create batch processor with config settings
    let workers = args.parallel.unwrap_or(config.processing.parallel_workers) as usize;
    let keep_temp = args.keep_temp || config.processing.keep_temp_files;

    // Resolve encoder (handles backward compatibility with legacy config)
    let encoder = resolve_encoder(&config, args.aac_encoder.as_deref());

    // Parse max concurrent encodes from config
    let max_concurrent = if config.performance.max_concurrent_encodes == "auto" {
        num_cpus::get() // Use all CPU cores
    } else {
        config.performance.max_concurrent_encodes
            .parse::<usize>()
            .unwrap_or(num_cpus::get())
            .clamp(1, 16)
    };

    // Parse max concurrent files per book from config
    let max_concurrent_files = if config.performance.max_concurrent_files_per_book == "auto" {
        num_cpus::get()
    } else {
        config.performance.max_concurrent_files_per_book
            .parse::<usize>()
            .unwrap_or(8)
            .clamp(1, 32)
    };

    // Create retry config from settings
    let retry_config = RetryConfig::with_settings(
        config.processing.max_retries as usize,
        std::time::Duration::from_secs(config.processing.retry_delay),
        std::time::Duration::from_secs(30),
        2.0,
    );

    let batch_processor = BatchProcessor::with_options(
        workers,
        keep_temp,
        encoder,
        config.performance.enable_parallel_encoding,
        max_concurrent,
        max_concurrent_files,
        args.quality.clone(),
        retry_config,
    );

    // Process batch
    println!("\n{} Processing {} audiobook(s)...\n", style("→").cyan(), book_folders.len());

    let results = batch_processor
        .process_batch(
            book_folders,
            &output_dir,
            &config.quality.chapter_source,
        )
        .await;

    // Print results
    println!();
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;

    for result in &results {
        if result.success {
            println!(
                "  {} {} ({:.1}s, {})",
                style("✓").green(),
                style(&result.book_name).yellow(),
                result.processing_time,
                if result.used_copy_mode {
                    "copy mode"
                } else {
                    "transcode"
                }
            );
        } else {
            println!(
                "  {} {} - {}",
                style("✗").red(),
                style(&result.book_name).yellow(),
                result.error_message.as_deref().unwrap_or("Unknown error")
            );
        }
    }

    println!(
        "\n{} Batch complete: {} successful, {} failed",
        style("✓").green(),
        style(successful).green(),
        if failed > 0 {
            style(failed).red()
        } else {
            style(failed).dim()
        }
    );

    Ok(())
}

/// Handle the organize command
pub fn handle_organize(args: OrganizeArgs, config: Config) -> Result<()> {
    // Determine root directory
    let root = args
        .root
        .or(config.directories.source.clone())
        .context("No root directory specified. Use --root or configure directories.source")?;

    println!(
        "{} Scanning audiobooks in: {}",
        style("→").cyan(),
        style(root.display()).yellow()
    );

    // Scan for audiobooks
    let scanner = Scanner::from_config(&config);
    let book_folders = scanner
        .scan_directory(&root)
        .context("Failed to scan directory")?;

    if book_folders.is_empty() {
        println!("{} No audiobooks found", style("✗").red());
        return Ok(());
    }

    println!(
        "{} Found {} audiobook(s)",
        style("✓").green(),
        style(book_folders.len()).cyan()
    );

    // Create organizer
    let organizer = Organizer::with_dry_run(root, &config, args.dry_run);

    // Dry run notice
    if args.dry_run {
        println!("\n{} DRY RUN MODE - No changes will be made\n", style("ℹ").blue());
    }

    // Organize books
    let results = organizer.organize_batch(book_folders);

    // Print results
    println!();
    for result in &results {
        let action_str = result.action.description();

        if result.success {
            match result.destination_path {
                Some(ref dest) => {
                    println!(
                        "  {} {} → {}",
                        style("✓").green(),
                        style(&result.book_name).yellow(),
                        style(dest.display()).cyan()
                    );
                }
                None => {
                    println!(
                        "  {} {} ({})",
                        style("→").dim(),
                        style(&result.book_name).dim(),
                        style(action_str).dim()
                    );
                }
            }
        } else {
            println!(
                "  {} {} - {}",
                style("✗").red(),
                style(&result.book_name).yellow(),
                result.error_message.as_deref().unwrap_or("Unknown error")
            );
        }
    }

    let moved = results
        .iter()
        .filter(|r| r.success && r.destination_path.is_some())
        .count();
    let skipped = results.iter().filter(|r| r.destination_path.is_none()).count();
    let failed = results.iter().filter(|r| !r.success).count();

    println!(
        "\n{} Organization complete: {} moved, {} skipped, {} failed",
        style("✓").green(),
        style(moved).green(),
        style(skipped).dim(),
        if failed > 0 {
            style(failed).red()
        } else {
            style(failed).dim()
        }
    );

    Ok(())
}

/// Handle the config command
pub fn handle_config(command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::Init { force } => {
            let config_path = ConfigManager::default_config_path()?;

            if config_path.exists() && !force {
                println!(
                    "{} Configuration file already exists: {}",
                    style("✗").red(),
                    style(config_path.display()).yellow()
                );
                println!("Use --force to overwrite");
                return Ok(());
            }

            // Create config directory if needed
            ConfigManager::ensure_config_dir()?;

            // Create default config
            let config = Config::default();
            ConfigManager::save(&config, Some(&config_path))?;

            println!(
                "{} Configuration file created: {}",
                style("✓").green(),
                style(config_path.display()).yellow()
            );
        }

        ConfigCommands::Show { config: _ } => {
            let config_path = ConfigManager::default_config_path()?;
            let config = ConfigManager::load(&config_path)?;
            let yaml = serde_yaml::to_string(&config)?;
            println!("{}", yaml);
        }

        ConfigCommands::Path => {
            let config_path = ConfigManager::default_config_path()?;
            println!("{}", config_path.display());
        }

        ConfigCommands::Validate { config: _ } => {
            let config_path = ConfigManager::default_config_path()?;
            ConfigManager::load(&config_path)?;
            println!(
                "{} Configuration is valid",
                style("✓").green()
            );
        }

        ConfigCommands::Edit => {
            let config_path = ConfigManager::default_config_path()?;
            println!("{} Opening editor for: {}", style("→").cyan(), style(config_path.display()).yellow());
            // TODO: Implement editor opening
            println!("{} Editor integration not yet implemented", style("ℹ").blue());
        }
    }

    Ok(())
}

/// Handle the check command
pub fn handle_check() -> Result<()> {
    println!("{} Checking system dependencies...\n", style("→").cyan());

    let results = vec![
        ("FFmpeg", DependencyChecker::check_ffmpeg().found),
        ("AtomicParsley", DependencyChecker::check_atomic_parsley().found),
        ("MP4Box", DependencyChecker::check_mp4box().found),
    ];

    let all_found = results.iter().all(|(_, found)| *found);

    for (tool, found) in &results {
        if *found {
            println!("  {} {}", style("✓").green(), style(tool).cyan());

            // Show encoder information for FFmpeg
            if *tool == "FFmpeg" {
                let available_encoders = DependencyChecker::get_available_encoders();
                let selected_encoder = DependencyChecker::get_selected_encoder();

                if !available_encoders.is_empty() {
                    print!("    AAC Encoders: ");
                    for (i, encoder) in available_encoders.iter().enumerate() {
                        if i > 0 {
                            print!(", ");
                        }
                        if *encoder == selected_encoder {
                            print!("{} {}", style(encoder).green(), style("(selected)").dim());
                        } else {
                            print!("{}", style(encoder).dim());
                        }
                    }
                    println!();
                }
            }
        } else {
            println!("  {} {} (not found)", style("✗").red(), style(tool).yellow());
        }
    }

    println!();
    if all_found {
        println!("{} All dependencies found", style("✓").green());
    } else {
        println!("{} Some dependencies are missing", style("✗").red());
        println!("\nInstall missing dependencies:");
        println!("  macOS:   brew install ffmpeg atomicparsley gpac");
        println!("  Ubuntu:  apt install ffmpeg atomicparsley gpac");
    }

    Ok(())
}

/// Handle the metadata command
pub async fn handle_metadata(command: MetadataCommands, config: Config) -> Result<()> {
    match command {
        MetadataCommands::Fetch { asin, title, author, region, output } => {
            println!("{} Fetching Audible metadata...", style("→").cyan());

            // Parse region
            let audible_region = AudibleRegion::from_str(&region)
                .unwrap_or(AudibleRegion::US);

            // Create client and cache
            let client = AudibleClient::with_rate_limit(
                audible_region,
                config.metadata.audible.rate_limit_per_minute
            )?;
            let cache = AudibleCache::with_ttl_hours(config.metadata.audible.cache_duration_hours)?;

            // Fetch metadata
            let metadata = if let Some(asin_val) = asin {
                // Direct ASIN lookup
                println!("  {} Looking up ASIN: {}", style("→").cyan(), asin_val);

                // Try cache first
                if let Some(cached) = cache.get(&asin_val).await {
                    println!("  {} Using cached metadata", style("✓").green());
                    cached
                } else {
                    let fetched = client.fetch_by_asin(&asin_val).await?;
                    cache.set(&asin_val, &fetched).await?;
                    fetched
                }
            } else if title.is_some() || author.is_some() {
                // Search by title/author
                println!("  {} Searching: title={:?}, author={:?}",
                    style("→").cyan(), title, author);

                let results = client.search(title.as_deref(), author.as_deref()).await?;

                if results.is_empty() {
                    bail!("No results found for search query");
                }

                // Display search results
                println!("\n{} Found {} result(s):", style("✓").green(), results.len());
                for (i, result) in results.iter().enumerate().take(5) {
                    println!("  {}. {} by {}",
                        i + 1,
                        style(&result.title).yellow(),
                        style(result.authors_string()).cyan()
                    );
                }

                // Fetch first result
                println!("\n{} Fetching details for first result...", style("→").cyan());
                let asin_to_fetch = &results[0].asin;

                if let Some(cached) = cache.get(asin_to_fetch).await {
                    cached
                } else {
                    let fetched = client.fetch_by_asin(asin_to_fetch).await?;
                    cache.set(asin_to_fetch, &fetched).await?;
                    fetched
                }
            } else {
                bail!("Must provide --asin or --title/--author");
            };

            // Display metadata
            println!("\n{}", style("=".repeat(60)).dim());
            println!("{}: {}", style("Title").bold(), metadata.title);
            if let Some(subtitle) = &metadata.subtitle {
                println!("{}: {}", style("Subtitle").bold(), subtitle);
            }
            if !metadata.authors.is_empty() {
                println!("{}: {}", style("Author(s)").bold(), metadata.authors_string());
            }
            if !metadata.narrators.is_empty() {
                println!("{}: {}", style("Narrator(s)").bold(), metadata.narrators_string());
            }
            if let Some(publisher) = &metadata.publisher {
                println!("{}: {}", style("Publisher").bold(), publisher);
            }
            if let Some(year) = metadata.published_year {
                println!("{}: {}", style("Published").bold(), year);
            }
            if let Some(duration_min) = metadata.runtime_minutes() {
                let hours = duration_min / 60;
                let mins = duration_min % 60;
                println!("{}: {}h {}m", style("Duration").bold(), hours, mins);
            }
            if let Some(lang) = &metadata.language {
                println!("{}: {}", style("Language").bold(), lang);
            }
            if !metadata.genres.is_empty() {
                println!("{}: {}", style("Genres").bold(), metadata.genres.join(", "));
            }
            if !metadata.series.is_empty() {
                for series in &metadata.series {
                    let seq_info = if let Some(seq) = &series.sequence {
                        format!(" (Book {})", seq)
                    } else {
                        String::new()
                    };
                    println!("{}: {}{}", style("Series").bold(), series.name, seq_info);
                }
            }
            println!("{}: {}", style("ASIN").bold(), metadata.asin);
            println!("{}", style("=".repeat(60)).dim());

            // Save to file if requested
            if let Some(output_path) = output {
                let json = serde_json::to_string_pretty(&metadata)?;
                std::fs::write(&output_path, json)?;
                println!("\n{} Saved metadata to: {}",
                    style("✓").green(),
                    style(output_path.display()).yellow()
                );
            }

            Ok(())
        }

        MetadataCommands::Enrich {
            file,
            asin,
            auto_detect,
            region,
            chapters: _chapters,
            chapters_asin: _chapters_asin,
            update_chapters_only: _update_chapters_only,
            merge_strategy: _merge_strategy,
        } => {
            println!("{} Enriching M4B file with Audible metadata...", style("→").cyan());

            if !file.exists() {
                bail!("File does not exist: {}", file.display());
            }

            // Detect or use provided ASIN
            let asin_to_use = if let Some(asin_val) = asin {
                asin_val
            } else if auto_detect {
                detect_asin(&file.display().to_string())
                    .ok_or_else(|| anyhow::anyhow!("Could not detect ASIN from filename: {}", file.display()))?
            } else {
                bail!("Must provide --asin or use --auto-detect");
            };

            println!("  {} Using ASIN: {}", style("→").cyan(), asin_to_use);

            // Parse region
            let audible_region = AudibleRegion::from_str(&region)
                .unwrap_or(AudibleRegion::US);

            // Create client and cache
            let client = AudibleClient::with_rate_limit(
                audible_region,
                config.metadata.audible.rate_limit_per_minute
            )?;
            let cache = AudibleCache::with_ttl_hours(config.metadata.audible.cache_duration_hours)?;

            // Fetch metadata
            let metadata = if let Some(cached) = cache.get(&asin_to_use).await {
                println!("  {} Using cached metadata", style("✓").green());
                cached
            } else {
                println!("  {} Fetching from Audible...", style("→").cyan());
                let fetched = client.fetch_by_asin(&asin_to_use).await?;
                cache.set(&asin_to_use, &fetched).await?;
                fetched
            };

            println!("  {} Found: {}", style("✓").green(), metadata.title);

            // Download cover if available and enabled
            let cover_path = if config.metadata.audible.download_covers {
                if let Some(cover_url) = &metadata.cover_url {
                    println!("  {} Downloading cover art...", style("→").cyan());
                    let temp_cover = std::env::temp_dir().join(format!("{}.jpg", asin_to_use));
                    client.download_cover(cover_url, &temp_cover).await?;
                    println!("  {} Cover downloaded", style("✓").green());
                    Some(temp_cover)
                } else {
                    None
                }
            } else {
                None
            };

            // Inject metadata (this will be implemented in metadata.rs)
            println!("  {} Injecting metadata...", style("→").cyan());
            crate::audio::inject_audible_metadata(&file, &metadata, cover_path.as_deref()).await?;

            println!("\n{} Successfully enriched: {}",
                style("✓").green(),
                style(file.display()).yellow()
            );

            Ok(())
        }
    }
}

/// Handle the match command
pub async fn handle_match(args: MatchArgs, config: Config) -> Result<()> {
    // Determine files to process
    let files = get_files_to_process(&args)?;

    if files.is_empty() {
        println!("{} No M4B files found", style("✗").red());
        return Ok(());
    }

    println!(
        "{} Found {} M4B file(s)",
        style("✓").green(),
        style(files.len()).cyan()
    );

    // Initialize Audible client and cache
    let region = AudibleRegion::from_str(&args.region)?;
    let retry_config = crate::core::RetryConfig::with_settings(
        config.metadata.audible.api_max_retries as usize,
        std::time::Duration::from_secs(config.metadata.audible.api_retry_delay_secs),
        std::time::Duration::from_secs(config.metadata.audible.api_max_retry_delay_secs),
        2.0,
    );
    let client = AudibleClient::with_config(
        region,
        config.metadata.audible.rate_limit_per_minute,
        retry_config,
    )?;
    let cache = AudibleCache::with_ttl_hours(
        config.metadata.audible.cache_duration_hours
    )?;

    // Process each file
    let mut processed = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for (idx, file_path) in files.iter().enumerate() {
        println!(
            "\n{} [{}/{}] Processing: {}",
            style("→").cyan(),
            idx + 1,
            files.len(),
            style(file_path.display()).yellow()
        );

        match process_single_file(&file_path, &args, &client, &cache, &config).await {
            Ok(ProcessResult::Applied) => processed += 1,
            Ok(ProcessResult::Skipped) => skipped += 1,
            Err(e) => {
                eprintln!("{} Error: {}", style("✗").red(), e);
                failed += 1;
            }
        }
    }

    // Summary
    println!("\n{}", style("Summary:").bold().cyan());
    println!("  {} Processed: {}", style("✓").green(), processed);
    println!("  {} Skipped: {}", style("→").yellow(), skipped);
    if failed > 0 {
        println!("  {} Failed: {}", style("✗").red(), failed);
    }

    Ok(())
}

/// Result of processing a single file
enum ProcessResult {
    Applied,
    Skipped,
}

/// Process a single M4B file
async fn process_single_file(
    file_path: &PathBuf,
    args: &MatchArgs,
    client: &AudibleClient,
    _cache: &AudibleCache,
    config: &Config,
) -> Result<ProcessResult> {
    // Extract current metadata
    let mut current = if args.title.is_some() || args.author.is_some() {
        // Manual override
        CurrentMetadata {
            title: args.title.clone(),
            author: args.author.clone(),
            year: None,
            duration: None,
            source: MetadataSource::Manual,
        }
    } else {
        // Auto-extract
        extraction::extract_current_metadata(file_path)?
    };

    // Search loop (allows re-search)
    loop {
        // Search Audible
        let search_results = search_audible(&current, client).await?;

        if search_results.is_empty() {
            println!("{} No matches found on Audible", style("⚠").yellow());

            if args.auto {
                return Ok(ProcessResult::Skipped);
            }

            // Offer manual entry or skip
            match prompt_no_results_action()? {
                NoResultsAction::ManualEntry => {
                    let manual_metadata = prompt_manual_metadata()?;
                    apply_metadata(file_path, &manual_metadata, args, config).await?;
                    return Ok(ProcessResult::Applied);
                }
                NoResultsAction::CustomSearch => {
                    let (title, author) = prompt_custom_search()?;
                    current.title = title;
                    current.author = author;
                    current.source = MetadataSource::Manual;
                    continue; // Re-search
                }
                NoResultsAction::Skip => {
                    return Ok(ProcessResult::Skipped);
                }
            }
        }

        // Score and rank candidates
        let candidates = scoring::score_and_sort(&current, search_results);

        // Auto mode: select best match
        if args.auto {
            let best = &candidates[0];
            println!(
                "  {} Auto-selected: {} ({:.1}%)",
                style("✓").green(),
                best.metadata.title,
                (1.0 - best.distance.total_distance()) * 100.0
            );

            if !args.dry_run {
                apply_metadata(file_path, &best.metadata, args, config).await?;
            }
            return Ok(ProcessResult::Applied);
        }

        // Interactive mode
        match prompt_match_selection(&current, &candidates)? {
            UserChoice::SelectMatch(idx) => {
                let selected = &candidates[idx];

                // Show what's about to be applied
                println!(
                    "  {} Applying: {} by {}",
                    style("→").cyan(),
                    style(&selected.metadata.title).yellow(),
                    style(selected.metadata.authors.first().map(|a| a.name.as_str()).unwrap_or("Unknown")).cyan()
                );

                // Apply directly - selecting is confirming
                if !args.dry_run {
                    apply_metadata(file_path, &selected.metadata, args, config).await?;
                } else {
                    println!("  {} Dry run - metadata not applied", style("→").yellow());
                }
                return Ok(ProcessResult::Applied);
            }
            UserChoice::Skip => {
                return Ok(ProcessResult::Skipped);
            }
            UserChoice::ManualEntry => {
                let manual_metadata = prompt_manual_metadata()?;
                if !args.dry_run {
                    apply_metadata(file_path, &manual_metadata, args, config).await?;
                }
                return Ok(ProcessResult::Applied);
            }
            UserChoice::CustomSearch => {
                let (title, author) = prompt_custom_search()?;
                current.title = title;
                current.author = author;
                current.source = MetadataSource::Manual;
                continue; // Re-search
            }
        }
    }
}

/// Get list of M4B files to process
fn get_files_to_process(args: &MatchArgs) -> Result<Vec<PathBuf>> {
    if let Some(file) = &args.file {
        // Single file mode
        if !file.exists() {
            bail!("File not found: {}", file.display());
        }
        if !is_m4b_file(file) {
            bail!("File is not an M4B: {}", file.display());
        }
        Ok(vec![file.clone()])
    } else if let Some(dir) = &args.dir {
        // Directory mode
        if !dir.is_dir() {
            bail!("Not a directory: {}", dir.display());
        }

        let mut files = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && is_m4b_file(&path) {
                files.push(path);
            }
        }

        files.sort();
        Ok(files)
    } else {
        bail!("Must specify --file or --dir");
    }
}

/// Check if file is M4B
fn is_m4b_file(path: &PathBuf) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("m4b"))
        .unwrap_or(false)
}

/// Search Audible API
async fn search_audible(
    current: &CurrentMetadata,
    client: &AudibleClient,
) -> Result<Vec<crate::models::AudibleMetadata>> {
    // Build search query
    let title = current.title.as_deref();
    let author = current.author.as_deref();

    if title.is_none() && author.is_none() {
        bail!("Need at least title or author to search");
    }

    // Search Audible (now returns full metadata via two-step process)
    let metadata_results = client.search(title, author).await?;

    Ok(metadata_results)
}

/// Apply metadata to M4B file
async fn apply_metadata(
    file_path: &PathBuf,
    metadata: &crate::models::AudibleMetadata,
    args: &MatchArgs,
    config: &Config,
) -> Result<()> {
    // Download cover if needed
    let cover_path = if !args.keep_cover && metadata.cover_url.is_some() && config.metadata.audible.download_covers {
        let temp_cover = std::env::temp_dir().join(format!("{}.jpg", metadata.asin));

        if let Some(cover_url) = &metadata.cover_url {
            let retry_config = crate::core::RetryConfig::with_settings(
                config.metadata.audible.api_max_retries as usize,
                std::time::Duration::from_secs(config.metadata.audible.api_retry_delay_secs),
                std::time::Duration::from_secs(config.metadata.audible.api_max_retry_delay_secs),
                2.0,
            );
            let client = AudibleClient::with_config(
                AudibleRegion::US, // Region doesn't matter for covers
                config.metadata.audible.rate_limit_per_minute,
                retry_config,
            )?;
            client.download_cover(cover_url, &temp_cover).await?;
            Some(temp_cover)
        } else {
            None
        }
    } else {
        None
    };

    // Inject metadata
    crate::audio::inject_audible_metadata(file_path, metadata, cover_path.as_deref()).await?;

    println!(
        "  {} Metadata applied successfully{}",
        style("✓").green(),
        if cover_path.is_some() {
            " (including cover art)"
        } else {
            ""
        }
    );

    Ok(())
}

/// Action to take when no results found
enum NoResultsAction {
    ManualEntry,
    CustomSearch,
    Skip,
}

/// Prompt for action when no results found
fn prompt_no_results_action() -> Result<NoResultsAction> {
    use inquire::Select;

    let options = vec![
        "[S]kip this file",
        "Search with [D]ifferent terms",
        "Enter metadata [M]anually",
    ];

    let selection = Select::new("What would you like to do?", options).prompt()?;

    match selection {
        "[S]kip this file" => Ok(NoResultsAction::Skip),
        "Search with [D]ifferent terms" => Ok(NoResultsAction::CustomSearch),
        "Enter metadata [M]anually" => Ok(NoResultsAction::ManualEntry),
        _ => Ok(NoResultsAction::Skip),
    }
}
