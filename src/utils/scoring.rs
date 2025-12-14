//! Scoring and distance calculation for metadata matching

use crate::models::{AudibleMetadata, CurrentMetadata, MatchCandidate, MetadataDistance, MatchConfidence};

/// Calculate distance between current metadata and Audible candidate
pub fn calculate_distance(
    current: &CurrentMetadata,
    candidate: &AudibleMetadata,
) -> MetadataDistance {
    let mut distance = MetadataDistance::new();

    // Title comparison (weight: 0.4)
    if let Some(cur_title) = &current.title {
        let cand_title = &candidate.title;
        let title_dist = string_distance(cur_title, cand_title);
        distance.add_penalty("title", title_dist, 0.4);
    }

    // Author comparison (weight: 0.3)
    if let Some(cur_author) = &current.author {
        // Compare against all Audible authors, use best match
        let author_dist = candidate.authors.iter()
            .map(|a| string_distance(cur_author, &a.name))
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1.0);
        distance.add_penalty("author", author_dist, 0.3);
    }

    // Year comparison (weight: 0.1)
    if let (Some(cur_year), Some(cand_year)) = (current.year, candidate.published_year) {
        let year_dist = year_distance(cur_year, cand_year);
        distance.add_penalty("year", year_dist, 0.1);
    }

    // Duration comparison (weight: 0.2)
    if let (Some(cur_dur), Some(cand_dur_ms)) = (current.duration, candidate.runtime_length_ms) {
        let cand_dur_sec = cand_dur_ms as f64 / 1000.0;
        let dur_dist = duration_distance(cur_dur, cand_dur_sec);
        distance.add_penalty("duration", dur_dist, 0.2);
    }

    distance
}

/// Normalized string distance using Levenshtein (0.0 = identical, 1.0 = completely different)
pub fn string_distance(a: &str, b: &str) -> f64 {
    // Normalize: lowercase, trim, remove "the" prefix
    let a_norm = normalize_string(a);
    let b_norm = normalize_string(b);

    // Use strsim::normalized_levenshtein (returns 0.0-1.0 similarity)
    let similarity = strsim::normalized_levenshtein(&a_norm, &b_norm);

    // Convert similarity to distance
    1.0 - similarity
}

/// Year distance with tolerance (1.0 = off by >10 years)
fn year_distance(a: u32, b: u32) -> f64 {
    let diff = (a as i32 - b as i32).abs();
    (diff as f64 / 10.0).min(1.0)
}

/// Duration distance with 5% tolerance (1.0 = off by >20%)
fn duration_distance(a: f64, b: f64) -> f64 {
    let diff_ratio = ((a - b).abs() / a.max(b)).max(0.0);

    // 0-5% difference = 0.0 distance (acceptable)
    // 5-20% difference = linear scale to 0.75
    // >20% difference = 1.0 distance
    if diff_ratio < 0.05 {
        0.0
    } else if diff_ratio < 0.20 {
        ((diff_ratio - 0.05) / 0.15) * 0.75
    } else {
        1.0
    }
}

/// Normalize string for comparison
pub fn normalize_string(s: &str) -> String {
    let mut normalized = s.to_lowercase().trim().to_string();

    // Remove leading "the " if present
    if normalized.starts_with("the ") {
        normalized = normalized[4..].to_string();
    }

    // Remove special characters but keep spaces
    normalized.retain(|c| c.is_alphanumeric() || c.is_whitespace());

    normalized
}

/// Score candidates and sort by distance (best first)
pub fn score_and_sort(
    current: &CurrentMetadata,
    candidates: Vec<AudibleMetadata>,
) -> Vec<MatchCandidate> {
    let mut scored: Vec<MatchCandidate> = candidates
        .into_iter()
        .map(|metadata| {
            let distance = calculate_distance(current, &metadata);
            let confidence = determine_confidence(distance.total_distance());
            MatchCandidate {
                distance,
                metadata,
                confidence,
            }
        })
        .collect();

    // Sort by distance (ascending = best first)
    scored.sort_by(|a, b| {
        a.distance.total_distance()
            .partial_cmp(&b.distance.total_distance())
            .unwrap()
    });

    scored
}

/// Determine confidence level based on distance
pub fn determine_confidence(distance: f64) -> MatchConfidence {
    if distance < 0.04 {
        MatchConfidence::Strong
    } else if distance < 0.12 {
        MatchConfidence::Medium
    } else if distance < 0.20 {
        MatchConfidence::Low
    } else {
        MatchConfidence::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_distance() {
        // Identical strings
        assert_eq!(string_distance("Hello World", "Hello World"), 0.0);
        assert_eq!(string_distance("hello world", "HELLO WORLD"), 0.0);

        // Similar strings
        let dist = string_distance("Project Hail Mary", "Project Haile Mary");
        assert!(dist > 0.0 && dist < 0.15);  // Small typo

        // Different strings
        let dist = string_distance("Completely Different", "Not the Same");
        assert!(dist > 0.5);
    }

    #[test]
    fn test_normalize_string() {
        assert_eq!(normalize_string("The Hobbit"), "hobbit");
        assert_eq!(normalize_string("  Project Hail Mary  "), "project hail mary");
        assert_eq!(normalize_string("Author's Name"), "authors name");
        assert_eq!(normalize_string("Title! @ # $"), "title");
    }

    #[test]
    fn test_year_distance() {
        assert_eq!(year_distance(2020, 2020), 0.0);  // Same year
        assert_eq!(year_distance(2020, 2025), 0.5);  // 5 years apart
        assert_eq!(year_distance(2020, 2030), 1.0);  // 10 years apart
        assert!(year_distance(2020, 2035) >= 1.0);    // >10 years (clamped to 1.0)
    }

    #[test]
    fn test_duration_distance() {
        // Within 5% tolerance
        assert_eq!(duration_distance(3600.0, 3620.0), 0.0);  // ~0.5% diff

        // 5-20% range
        let dist = duration_distance(3600.0, 3960.0);  // 10% diff
        assert!(dist > 0.0 && dist < 0.75);

        // Over 20% difference
        let dist = duration_distance(3600.0, 4500.0);  // 25% diff
        assert_eq!(dist, 1.0);
    }

    #[test]
    fn test_determine_confidence() {
        assert_eq!(determine_confidence(0.02), MatchConfidence::Strong);
        assert_eq!(determine_confidence(0.08), MatchConfidence::Medium);
        assert_eq!(determine_confidence(0.15), MatchConfidence::Low);
        assert_eq!(determine_confidence(0.50), MatchConfidence::None);
    }
}
