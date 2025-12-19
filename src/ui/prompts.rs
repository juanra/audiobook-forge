//! Interactive prompts for metadata matching

use crate::models::{AudibleMetadata, CurrentMetadata, MatchCandidate, MatchConfidence, AudibleAuthor};
use anyhow::Result;
use console::style;
use inquire::{Confirm, CustomType, Select, Text};

/// User's choice after viewing candidates
pub enum UserChoice {
    SelectMatch(usize), // Index into candidates
    Skip,               // Leave file unchanged
    ManualEntry,        // Enter metadata manually
    CustomSearch,       // Search with different terms
}

/// Display match candidates and prompt for selection
pub fn prompt_match_selection(
    current: &CurrentMetadata,
    candidates: &[MatchCandidate],
) -> Result<UserChoice> {
    println!("\n{}", style("Match Candidates:").bold().cyan());
    println!(
        "Current: {} by {}",
        current.title.as_deref().unwrap_or("Unknown"),
        current.author.as_deref().unwrap_or("Unknown")
    );
    println!();

    // Build options for inquire::Select
    let mut options = Vec::new();

    for (i, candidate) in candidates.iter().enumerate() {
        let percentage = (1.0 - candidate.distance.total_distance()) * 100.0;
        let color_fn: fn(String) -> String = match candidate.confidence {
            MatchConfidence::Strong => style_green,
            MatchConfidence::Medium => style_yellow,
            MatchConfidence::Low => style_red,
            MatchConfidence::None => style_dim,
        };

        let label = format!(
            "{}. [{:>5.1}%] {} by {} ({}, {})",
            i + 1,
            percentage,
            candidate.metadata.title,
            candidate
                .metadata
                .authors
                .first()
                .map(|a| a.name.as_str())
                .unwrap_or("Unknown"),
            candidate
                .metadata
                .published_year
                .map(|y| y.to_string())
                .unwrap_or_else(|| "N/A".to_string()),
            format_duration(candidate.metadata.runtime_length_ms),
        );

        options.push(color_fn(label));
    }

    // Add action options
    options.push(style("───────────────────────────────────").dim().to_string());
    options.push(style("[S] Skip this file").yellow().to_string());
    options.push(style("[M] Enter metadata manually").cyan().to_string());
    options.push(style("[R] Search with different terms").blue().to_string());

    // Show select menu
    let selection = Select::new("Select an option:", options)
        .with_page_size(15)
        .prompt()?;

    // Parse selection
    // Find first digit in the selection string (skipping any ANSI color codes)
    for (idx, ch) in selection.chars().enumerate() {
        if ch.is_ascii_digit() {
            let digit = ch.to_digit(10).unwrap() as usize;
            // Validate it's a valid candidate index
            if digit >= 1 && digit <= candidates.len() {
                return Ok(UserChoice::SelectMatch(digit - 1));
            }
            // If we found a digit but it's not valid, stop searching
            break;
        }
        // Stop searching after first 20 characters to avoid matching digits in titles
        if idx >= 20 {
            break;
        }
    }

    // Check for action options
    if selection.contains("[S]") {
        Ok(UserChoice::Skip)
    } else if selection.contains("[M]") {
        Ok(UserChoice::ManualEntry)
    } else if selection.contains("[R]") {
        Ok(UserChoice::CustomSearch)
    } else {
        Ok(UserChoice::Skip) // Fallback
    }
}

/// Show detailed comparison and confirm selection
pub fn confirm_match(
    current: &CurrentMetadata,
    selected: &MatchCandidate,
) -> Result<bool> {
    println!("\n{}", style("Metadata Changes:").bold().cyan());
    println!();

    show_field_change(
        "Title",
        current.title.as_deref(),
        Some(&selected.metadata.title),
    );

    show_field_change(
        "Author",
        current.author.as_deref(),
        selected
            .metadata
            .authors
            .first()
            .map(|a| a.name.as_str()),
    );

    if let Some(subtitle) = &selected.metadata.subtitle {
        show_field_change("Subtitle", None, Some(subtitle));
    }

    if let Some(narrator) = selected.metadata.narrators.first() {
        show_field_change("Narrator", None, Some(narrator));
    }

    show_field_change(
        "Year",
        current.year.as_ref().map(|y| y.to_string()).as_deref(),
        selected
            .metadata
            .published_year
            .as_ref()
            .map(|y| y.to_string())
            .as_deref(),
    );

    if let Some(publisher) = &selected.metadata.publisher {
        show_field_change("Publisher", None, Some(publisher));
    }

    println!();

    Ok(Confirm::new("Apply these changes?")
        .with_default(true)
        .prompt()?)
}

/// Prompt for manual metadata entry
pub fn prompt_manual_metadata() -> Result<AudibleMetadata> {
    println!("\n{}", style("Enter Metadata Manually:").bold().cyan());

    let title = Text::new("Title:").prompt()?;

    let author_name = Text::new("Author:").prompt()?;

    let narrator = Text::new("Narrator (optional):")
        .with_default("")
        .prompt()?;

    let year: Option<u32> = CustomType::new("Year (optional):")
        .with_error_message("Please enter a valid year or leave empty")
        .prompt_skippable()?;

    // Create minimal AudibleMetadata
    Ok(AudibleMetadata {
        asin: String::from("manual"),
        title,
        subtitle: None,
        authors: vec![AudibleAuthor {
            asin: None,
            name: author_name,
        }],
        narrators: if narrator.is_empty() {
            vec![]
        } else {
            vec![narrator]
        },
        publisher: None,
        published_year: year,
        description: None,
        cover_url: None,
        isbn: None,
        genres: vec![],
        tags: vec![],
        series: vec![],
        language: None,
        runtime_length_ms: None,
        rating: None,
        is_abridged: None,
    })
}

/// Prompt for custom search terms
pub fn prompt_custom_search() -> Result<(Option<String>, Option<String>)> {
    println!("\n{}", style("Custom Search:").bold().cyan());

    let title = Text::new("Title (optional):")
        .with_default("")
        .prompt()?;

    let author = Text::new("Author (optional):")
        .with_default("")
        .prompt()?;

    let title_opt = if title.is_empty() {
        None
    } else {
        Some(title)
    };
    let author_opt = if author.is_empty() {
        None
    } else {
        Some(author)
    };

    Ok((title_opt, author_opt))
}

/// Helper: show field change
fn show_field_change(field: &str, old: Option<&str>, new: Option<&str>) {
    let old_display = old.unwrap_or("(none)");
    let new_display = new.unwrap_or("(none)");

    if old != new {
        println!(
            "  {}: {} → {}",
            style(field).bold(),
            style(old_display).dim(),
            style(new_display).green()
        );
    } else {
        println!(
            "  {}: {}",
            style(field).bold(),
            style(new_display).dim()
        );
    }
}

/// Helper: format duration from milliseconds
fn format_duration(ms: Option<u64>) -> String {
    match ms {
        Some(ms) => {
            let hours = ms / 3_600_000;
            let minutes = (ms % 3_600_000) / 60_000;
            format!("{}h {}m", hours, minutes)
        }
        None => "N/A".to_string(),
    }
}

/// Helper color functions
fn style_green(s: String) -> String {
    style(s).green().to_string()
}
fn style_yellow(s: String) -> String {
    style(s).yellow().to_string()
}
fn style_red(s: String) -> String {
    style(s).red().to_string()
}
fn style_dim(s: String) -> String {
    style(s).dim().to_string()
}
