//! Interactive prompts for metadata matching

use crate::models::{AudibleMetadata, CurrentMetadata, MatchCandidate, MatchConfidence, AudibleAuthor};
use anyhow::Result;
use console::style;
use inquire::list_option::ListOption;
use inquire::{Confirm, CustomType, Select, Text};

/// User's choice after viewing candidates
#[derive(Debug)]
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

    // Action options are appended after the candidates. Selection is dispatched by
    // the chosen POSITION (see `dispatch_selection`) — NOT by parsing the label,
    // which previously misread ANSI SGR digits (e.g. "\x1b[32m") as the option
    // number and applied the wrong candidate (see issue #12).
    options.push(style("───────────────────────────────────").dim().to_string());
    options.push(style("[S] Skip this file").yellow().to_string());
    options.push(style("[M] Enter metadata manually").cyan().to_string());
    options.push(style("[R] Search with different terms").blue().to_string());

    // Show select menu and read back the chosen POSITION, not the label text.
    let ListOption { index, .. } = Select::new("Select an option:", options)
        .with_page_size(15)
        .raw_prompt()?;

    Ok(dispatch_selection(index, candidates.len()))
}

/// Map the chosen menu position to a [`UserChoice`].
///
/// Options are laid out as: `[0..n)` candidates, `[n]` separator, `[n+1]` Skip,
/// `[n+2]` Manual entry, `[n+3]` Custom search. Dispatching by position (rather
/// than by parsing the color-escaped label) is what fixes issue #12.
fn dispatch_selection(index: usize, num_candidates: usize) -> UserChoice {
    let separator_idx = num_candidates;
    let skip_idx = separator_idx + 1;
    let manual_idx = skip_idx + 1;
    let search_idx = manual_idx + 1;

    if index < num_candidates {
        UserChoice::SelectMatch(index)
    } else if index == skip_idx {
        UserChoice::Skip
    } else if index == manual_idx {
        UserChoice::ManualEntry
    } else if index == search_idx {
        UserChoice::CustomSearch
    } else {
        // Separator line or anything unexpected: leave the file unchanged.
        UserChoice::Skip
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

#[cfg(test)]
mod tests {
    use super::*;

    // Regression test for issue #12: selecting a candidate by position must map to
    // that exact candidate. The previous implementation reverse-parsed the index
    // out of the color-escaped label and misread ANSI SGR digits, so selecting
    // option 1 applied option 3's metadata.
    #[test]
    fn dispatch_selects_candidate_by_position() {
        // 3 candidates -> positions 0,1,2 are candidates.
        for i in 0..3 {
            match dispatch_selection(i, 3) {
                UserChoice::SelectMatch(idx) => assert_eq!(idx, i),
                other => panic!("position {i} should select candidate {i}, got {other:?}"),
            }
        }
    }

    #[test]
    fn dispatch_maps_action_rows_after_candidates() {
        // Layout for 3 candidates: [0,1,2]=candidates, 3=separator, 4=Skip,
        // 5=Manual, 6=Search.
        assert!(matches!(dispatch_selection(3, 3), UserChoice::Skip)); // separator
        assert!(matches!(dispatch_selection(4, 3), UserChoice::Skip));
        assert!(matches!(dispatch_selection(5, 3), UserChoice::ManualEntry));
        assert!(matches!(dispatch_selection(6, 3), UserChoice::CustomSearch));
    }

    #[test]
    fn dispatch_out_of_range_is_skip() {
        assert!(matches!(dispatch_selection(99, 3), UserChoice::Skip));
    }

    #[test]
    fn dispatch_single_candidate() {
        assert!(matches!(dispatch_selection(0, 1), UserChoice::SelectMatch(0)));
        assert!(matches!(dispatch_selection(2, 1), UserChoice::Skip)); // Skip row
    }
}
