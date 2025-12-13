//! CLI command handlers

use crate::cli::commands::{BuildArgs, ConfigCommands, OrganizeArgs};
use crate::core::{Analyzer, BatchProcessor, Organizer, RetryConfig, Scanner};
use crate::models::Config;
use crate::utils::{ConfigManager, DependencyChecker};
use anyhow::{Context, Result};
use console::style;
use std::path::PathBuf;

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

    // Require at least 2 MP3 files to consider it an audiobook (BookCase A)
    if mp3_count >= 2 {
        Ok(Some(current_dir))
    } else {
        Ok(None)
    }
}

/// Handle the build command
pub async fn handle_build(args: BuildArgs, config: Config) -> Result<()> {
    // Determine root directory (CLI arg > config > auto-detect > error)
    let (root, auto_detected) = if let Some(root_path) = args.root.or(config.directories.source.clone()) {
        (root_path, false)
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
    let scanner = Scanner::new();
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

    // Use Apple Silicon encoder if configured, otherwise auto-detect
    let use_apple_silicon = config.advanced.use_apple_silicon_encoder.unwrap_or(true);

    // Parse max concurrent encodes from config
    let max_concurrent = if config.performance.max_concurrent_encodes == "auto" {
        num_cpus::get() // Use all CPU cores
    } else {
        config.performance.max_concurrent_encodes
            .parse::<usize>()
            .unwrap_or(num_cpus::get())
            .clamp(1, 16)
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
        use_apple_silicon,
        config.performance.enable_parallel_encoding,
        max_concurrent,
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
    let scanner = Scanner::new();
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
