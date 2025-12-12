//! Folder organization for audiobooks

use crate::models::{BookFolder, BookCase, Config};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Result of an organization operation
#[derive(Debug, Clone)]
pub struct OrganizeResult {
    /// Book name
    pub book_name: String,
    /// Source path
    pub source_path: PathBuf,
    /// Destination path (None if no action taken)
    pub destination_path: Option<PathBuf>,
    /// Action taken
    pub action: OrganizeAction,
    /// Success flag
    pub success: bool,
    /// Error message (if failed)
    pub error_message: Option<String>,
}

/// Type of organization action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrganizeAction {
    /// Moved to conversion folder
    MovedToConvert,
    /// Moved to M4B folder
    MovedToM4B,
    /// Skipped (already in correct location)
    Skipped,
    /// Skipped (Case D - not a valid audiobook)
    SkippedInvalid,
}

impl OrganizeAction {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::MovedToConvert => "Moved to conversion folder",
            Self::MovedToM4B => "Moved to M4B folder",
            Self::Skipped => "Already in correct location",
            Self::SkippedInvalid => "Skipped (not a valid audiobook)",
        }
    }
}

/// Organizer for managing audiobook folder structure
pub struct Organizer {
    /// Root directory
    root: PathBuf,
    /// M4B folder name
    m4b_folder: String,
    /// Conversion folder name
    convert_folder: String,
    /// Dry run mode (don't actually move files)
    dry_run: bool,
}

impl Organizer {
    /// Create a new organizer
    pub fn new(root: PathBuf, config: &Config) -> Self {
        Self {
            root,
            m4b_folder: config.organization.m4b_folder.clone(),
            convert_folder: config.organization.convert_folder.clone(),
            dry_run: false,
        }
    }

    /// Create organizer with dry run mode
    pub fn with_dry_run(root: PathBuf, config: &Config, dry_run: bool) -> Self {
        Self {
            root,
            m4b_folder: config.organization.m4b_folder.clone(),
            convert_folder: config.organization.convert_folder.clone(),
            dry_run,
        }
    }

    /// Organize a single book folder
    pub fn organize_book(&self, book: &BookFolder) -> Result<OrganizeResult> {
        let book_name = book.name.clone();
        let source_path = book.folder_path.clone();

        // Determine target folder based on book case
        let (target_folder_name, action) = match book.case {
            BookCase::A | BookCase::B => {
                // Needs conversion
                (&self.convert_folder, OrganizeAction::MovedToConvert)
            }
            BookCase::C => {
                // Already M4B
                (&self.m4b_folder, OrganizeAction::MovedToM4B)
            }
            BookCase::D => {
                // Invalid audiobook - skip
                return Ok(OrganizeResult {
                    book_name,
                    source_path,
                    destination_path: None,
                    action: OrganizeAction::SkippedInvalid,
                    success: true,
                    error_message: None,
                });
            }
        };

        let target_folder = self.root.join(target_folder_name);

        // Check if already in target folder
        if let Some(parent) = source_path.parent() {
            if parent == target_folder {
                return Ok(OrganizeResult {
                    book_name,
                    source_path,
                    destination_path: None,
                    action: OrganizeAction::Skipped,
                    success: true,
                    error_message: None,
                });
            }
        }

        // Determine destination path
        let destination_path = target_folder.join(
            source_path
                .file_name()
                .context("Invalid source path")?,
        );

        // Handle naming conflicts
        let final_destination = self.resolve_naming_conflict(&destination_path)?;

        // Execute move (or simulate in dry run mode)
        if self.dry_run {
            tracing::info!(
                "[DRY RUN] Would move: {} -> {}",
                source_path.display(),
                final_destination.display()
            );
        } else {
            // Create target folder if it doesn't exist
            if !target_folder.exists() {
                fs::create_dir_all(&target_folder)
                    .with_context(|| format!("Failed to create folder: {}", target_folder.display()))?;
            }

            // Move the folder
            fs::rename(&source_path, &final_destination)
                .with_context(|| {
                    format!(
                        "Failed to move {} to {}",
                        source_path.display(),
                        final_destination.display()
                    )
                })?;

            tracing::info!(
                "Moved: {} -> {}",
                source_path.display(),
                final_destination.display()
            );
        }

        Ok(OrganizeResult {
            book_name,
            source_path,
            destination_path: Some(final_destination),
            action,
            success: true,
            error_message: None,
        })
    }

    /// Organize multiple books
    pub fn organize_batch(&self, books: Vec<BookFolder>) -> Vec<OrganizeResult> {
        let mut results = Vec::new();

        for book in books {
            match self.organize_book(&book) {
                Ok(result) => results.push(result),
                Err(e) => {
                    tracing::error!("Failed to organize {}: {}", book.name, e);
                    results.push(OrganizeResult {
                        book_name: book.name.clone(),
                        source_path: book.folder_path.clone(),
                        destination_path: None,
                        action: OrganizeAction::Skipped,
                        success: false,
                        error_message: Some(e.to_string()),
                    });
                }
            }
        }

        results
    }

    /// Resolve naming conflicts by appending numbers
    fn resolve_naming_conflict(&self, path: &Path) -> Result<PathBuf> {
        if !path.exists() {
            return Ok(path.to_path_buf());
        }

        let parent = path.parent().context("Invalid path")?;
        let base_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .context("Invalid filename")?;

        // Try appending numbers until we find an available name
        for i in 2..=999 {
            let new_name = format!("{}_{}", base_name, i);
            let new_path = parent.join(&new_name);

            if !new_path.exists() {
                tracing::warn!(
                    "Naming conflict: {} -> {}",
                    path.display(),
                    new_path.display()
                );
                return Ok(new_path);
            }
        }

        anyhow::bail!("Could not resolve naming conflict for {}", path.display())
    }

    /// Get target folder path for a book case
    pub fn get_target_folder(&self, case: BookCase) -> Option<PathBuf> {
        match case {
            BookCase::A | BookCase::B => Some(self.root.join(&self.convert_folder)),
            BookCase::C => Some(self.root.join(&self.m4b_folder)),
            BookCase::D => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::OrganizationConfig;
    use tempfile::tempdir;

    fn create_test_config() -> Config {
        let mut config = Config::default();
        config.organization = OrganizationConfig {
            m4b_folder: "M4B".to_string(),
            convert_folder: "To_Convert".to_string(),
        };
        config
    }

    #[test]
    fn test_organizer_creation() {
        let config = create_test_config();
        let organizer = Organizer::new(PathBuf::from("/tmp"), &config);
        assert_eq!(organizer.m4b_folder, "M4B");
        assert_eq!(organizer.convert_folder, "To_Convert");
        assert!(!organizer.dry_run);
    }

    #[test]
    fn test_organizer_dry_run() {
        let config = create_test_config();
        let organizer = Organizer::with_dry_run(PathBuf::from("/tmp"), &config, true);
        assert!(organizer.dry_run);
    }

    #[test]
    fn test_organize_action_description() {
        assert_eq!(
            OrganizeAction::MovedToConvert.description(),
            "Moved to conversion folder"
        );
        assert_eq!(
            OrganizeAction::MovedToM4B.description(),
            "Moved to M4B folder"
        );
        assert_eq!(
            OrganizeAction::Skipped.description(),
            "Already in correct location"
        );
        assert_eq!(
            OrganizeAction::SkippedInvalid.description(),
            "Skipped (not a valid audiobook)"
        );
    }

    #[test]
    fn test_get_target_folder() {
        let config = create_test_config();
        let organizer = Organizer::new(PathBuf::from("/audiobooks"), &config);

        assert_eq!(
            organizer.get_target_folder(BookCase::A),
            Some(PathBuf::from("/audiobooks/To_Convert"))
        );
        assert_eq!(
            organizer.get_target_folder(BookCase::B),
            Some(PathBuf::from("/audiobooks/To_Convert"))
        );
        assert_eq!(
            organizer.get_target_folder(BookCase::C),
            Some(PathBuf::from("/audiobooks/M4B"))
        );
        assert_eq!(organizer.get_target_folder(BookCase::D), None);
    }

    #[test]
    fn test_organize_invalid_book() {
        let dir = tempdir().unwrap();
        let config = create_test_config();
        let organizer = Organizer::new(dir.path().to_path_buf(), &config);

        let mut book = BookFolder::new(dir.path().join("Invalid"));
        book.case = BookCase::D;

        let result = organizer.organize_book(&book).unwrap();
        assert!(result.success);
        assert_eq!(result.action, OrganizeAction::SkippedInvalid);
        assert!(result.destination_path.is_none());
    }

    #[test]
    fn test_organize_batch() {
        let dir = tempdir().unwrap();
        let config = create_test_config();
        let organizer = Organizer::with_dry_run(dir.path().to_path_buf(), &config, true);

        // Create test books
        let book1_dir = dir.path().join("Book1");
        fs::create_dir(&book1_dir).unwrap();
        let mut book1 = BookFolder::new(book1_dir);
        book1.case = BookCase::A;

        let book2_dir = dir.path().join("Book2");
        fs::create_dir(&book2_dir).unwrap();
        let mut book2 = BookFolder::new(book2_dir);
        book2.case = BookCase::C;

        let results = organizer.organize_batch(vec![book1, book2]);
        assert_eq!(results.len(), 2);
        assert!(results[0].success);
        assert!(results[1].success);
    }

    #[test]
    fn test_resolve_naming_conflict() {
        let dir = tempdir().unwrap();
        let config = create_test_config();
        let organizer = Organizer::new(dir.path().to_path_buf(), &config);

        // Create existing file
        let existing = dir.path().join("book");
        fs::create_dir(&existing).unwrap();

        // Resolve conflict
        let resolved = organizer.resolve_naming_conflict(&existing).unwrap();
        assert_eq!(resolved, dir.path().join("book_2"));

        // Create book_2, resolve again
        fs::create_dir(&resolved).unwrap();
        let resolved2 = organizer.resolve_naming_conflict(&existing).unwrap();
        assert_eq!(resolved2, dir.path().join("book_3"));
    }
}
