//! Integration tests for audiobook-forge

use audiobook_forge::core::{Analyzer, Organizer, Processor, Scanner};
use audiobook_forge::models::{BookCase, Config};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a test directory structure with mock audiobook folders
fn create_test_audiobooks(root: &std::path::Path) -> std::io::Result<()> {
    // Book 1: Multiple MP3s (Case A)
    let book1 = root.join("The_Hobbit");
    fs::create_dir(&book1)?;
    fs::write(book1.join("01-Chapter1.mp3"), b"fake mp3 data")?;
    fs::write(book1.join("02-Chapter2.mp3"), b"fake mp3 data")?;
    fs::write(book1.join("03-Chapter3.mp3"), b"fake mp3 data")?;
    fs::write(book1.join("cover.jpg"), b"fake image")?;

    // Book 2: Single MP3 (Case B)
    let book2 = root.join("1984");
    fs::create_dir(&book2)?;
    fs::write(book2.join("1984_Complete.mp3"), b"fake mp3 data")?;
    fs::write(book2.join("folder.jpg"), b"fake image")?;

    // Book 3: Existing M4B (Case C)
    let book3 = root.join("Sapiens");
    fs::create_dir(&book3)?;
    fs::write(book3.join("Sapiens.m4b"), b"fake m4b data")?;

    // Book 4: Invalid (Case D)
    let book4 = root.join("Random_Files");
    fs::create_dir(&book4)?;
    fs::write(book4.join("notes.txt"), b"some notes")?;
    fs::write(book4.join("image.png"), b"random image")?;

    Ok(())
}

#[test]
fn test_scanner_integration() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create test structure
    create_test_audiobooks(root).unwrap();

    // Scan directory
    let scanner = Scanner::new();
    let books = scanner.scan_directory(root).unwrap();

    // Should find 3 valid audiobooks (Cases A, B, C)
    assert_eq!(books.len(), 3);

    // Verify book cases
    let cases: Vec<BookCase> = books.iter().map(|b| b.case).collect();
    assert!(cases.contains(&BookCase::A)); // The_Hobbit
    assert!(cases.contains(&BookCase::B)); // 1984
    assert!(cases.contains(&BookCase::C)); // Sapiens
    assert!(!cases.contains(&BookCase::D)); // Random_Files should be filtered
}

#[test]
fn test_organizer_integration() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create test structure
    create_test_audiobooks(root).unwrap();

    // Scan directory
    let scanner = Scanner::new();
    let books = scanner.scan_directory(root).unwrap();

    // Organize books (dry run)
    let config = Config::default();
    let organizer = Organizer::with_dry_run(root.to_path_buf(), &config, true);
    let results = organizer.organize_batch(books);

    // Should have 3 results
    assert_eq!(results.len(), 3);

    // All should succeed (dry run doesn't fail)
    assert!(results.iter().all(|r| r.success));
}

#[test]
fn test_organizer_actual_move() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create test structure
    create_test_audiobooks(root).unwrap();

    // Scan directory
    let scanner = Scanner::new();
    let books = scanner.scan_directory(root).unwrap();

    // Organize books (actual move)
    let config = Config::default();
    let organizer = Organizer::new(root.to_path_buf(), &config);
    let results = organizer.organize_batch(books);

    // All should succeed
    assert!(results.iter().all(|r| r.success));

    // Check folders were created
    let m4b_folder = root.join("M4B");
    let convert_folder = root.join("To_Convert");

    assert!(m4b_folder.exists());
    assert!(convert_folder.exists());

    // Check books were moved
    assert!(m4b_folder.join("Sapiens").exists()); // Case C
    assert!(convert_folder.join("The_Hobbit").exists()); // Case A
    assert!(convert_folder.join("1984").exists()); // Case B
}

#[test]
fn test_scanner_with_hidden_directories() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create hidden directory
    let hidden = root.join(".hidden_book");
    fs::create_dir(&hidden).unwrap();
    fs::write(hidden.join("01.mp3"), b"fake mp3").unwrap();

    // Create normal directory
    let normal = root.join("Normal_Book");
    fs::create_dir(&normal).unwrap();
    fs::write(normal.join("01.mp3"), b"fake mp3").unwrap();

    // Scan directory
    let scanner = Scanner::new();
    let books = scanner.scan_directory(root).unwrap();

    // Should only find the normal book
    assert_eq!(books.len(), 1);
    assert_eq!(books[0].name, "Normal_Book");
}

#[test]
fn test_naming_conflict_resolution() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create book
    let book1 = root.join("Book1");
    fs::create_dir(&book1).unwrap();
    fs::write(book1.join("01.mp3"), b"fake mp3").unwrap();

    // Create To_Convert folder with existing "Book1"
    let convert_folder = root.join("To_Convert");
    fs::create_dir(&convert_folder).unwrap();
    let existing = convert_folder.join("Book1");
    fs::create_dir(&existing).unwrap();

    // Scan and organize
    let scanner = Scanner::new();
    let books = scanner.scan_directory(root).unwrap();

    let config = Config::default();
    let organizer = Organizer::new(root.to_path_buf(), &config);
    let results = organizer.organize_batch(books);

    // Should succeed
    assert!(results[0].success);

    // Should be renamed to Book1_2
    let renamed = convert_folder.join("Book1_2");
    assert!(renamed.exists());
}

#[test]
fn test_cover_art_detection() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Book with cover.jpg
    let book1 = root.join("Book1");
    fs::create_dir(&book1).unwrap();
    fs::write(book1.join("01.mp3"), b"fake mp3").unwrap();
    fs::write(book1.join("cover.jpg"), b"fake image").unwrap();

    // Book with folder.png
    let book2 = root.join("Book2");
    fs::create_dir(&book2).unwrap();
    fs::write(book2.join("01.mp3"), b"fake mp3").unwrap();
    fs::write(book2.join("folder.png"), b"fake image").unwrap();

    // Book with random.jpg (should not be detected as cover)
    let book3 = root.join("Book3");
    fs::create_dir(&book3).unwrap();
    fs::write(book3.join("01.mp3"), b"fake mp3").unwrap();
    fs::write(book3.join("random.jpg"), b"fake image").unwrap();

    // Scan
    let scanner = Scanner::new();
    let books = scanner.scan_directory(root).unwrap();

    // Book1 and Book2 should have cover, Book3 should not
    let book1_result = books.iter().find(|b| b.name == "Book1").unwrap();
    let book2_result = books.iter().find(|b| b.name == "Book2").unwrap();
    let book3_result = books.iter().find(|b| b.name == "Book3").unwrap();

    assert!(book1_result.cover_file.is_some());
    assert!(book2_result.cover_file.is_some());
    assert!(book3_result.cover_file.is_none());
}

#[test]
fn test_natural_sorting() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create book with unnaturally ordered filenames
    let book = root.join("Book");
    fs::create_dir(&book).unwrap();
    fs::write(book.join("Chapter_10.mp3"), b"fake").unwrap();
    fs::write(book.join("Chapter_2.mp3"), b"fake").unwrap();
    fs::write(book.join("Chapter_1.mp3"), b"fake").unwrap();
    fs::write(book.join("Chapter_20.mp3"), b"fake").unwrap();

    // Scan
    let scanner = Scanner::new();
    let books = scanner.scan_directory(root).unwrap();

    // Files should be naturally sorted
    let file_names: Vec<String> = books[0]
        .mp3_files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();

    assert_eq!(file_names[0], "Chapter_1.mp3");
    assert_eq!(file_names[1], "Chapter_2.mp3");
    assert_eq!(file_names[2], "Chapter_10.mp3");
    assert_eq!(file_names[3], "Chapter_20.mp3");
}

#[test]
fn test_cue_file_detection() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Book with CUE file
    let book = root.join("Book");
    fs::create_dir(&book).unwrap();
    fs::write(book.join("audiobook.mp3"), b"fake mp3").unwrap();
    fs::write(book.join("audiobook.cue"), b"fake cue").unwrap();

    // Scan
    let scanner = Scanner::new();
    let books = scanner.scan_directory(root).unwrap();

    // Should detect CUE file
    assert!(books[0].cue_file.is_some());
}

#[test]
fn test_m4a_files_treated_as_mp3() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Book with M4A files (should be treated like MP3)
    let book = root.join("Book");
    fs::create_dir(&book).unwrap();
    fs::write(book.join("01.m4a"), b"fake m4a").unwrap();
    fs::write(book.join("02.m4a"), b"fake m4a").unwrap();

    // Scan
    let scanner = Scanner::new();
    let books = scanner.scan_directory(root).unwrap();

    // Should find book as Case A (multiple files to convert)
    assert_eq!(books.len(), 1);
    assert_eq!(books[0].case, BookCase::A);
    assert_eq!(books[0].mp3_files.len(), 2); // M4A treated as MP3
}
