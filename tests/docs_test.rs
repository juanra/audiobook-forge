//! Keep the documentation in step with the code.
//!
//! Run `UPDATE_DOCS=1 cargo test --test docs_test` to regenerate the generated
//! pages after changing the CLI.

use audiobook_forge::cli::Cli;
use audiobook_forge::Config;
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

/// Markdown files that document usage for end users.
fn documentation_files() -> Vec<PathBuf> {
    let mut files = vec![repo_path("README.md")];
    let mut wiki: Vec<PathBuf> = fs::read_dir(repo_path("docs/wiki"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .collect();
    wiki.sort();
    files.extend(wiki);
    files
}

/// Every `audiobook-forge ...` command inside a fenced code block, with
/// backslash continuations joined and trailing comments removed.
fn documented_commands(markdown: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut in_code_block = false;
    let mut pending: Option<String> = None;

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            pending = None;
            continue;
        }
        if !in_code_block {
            continue;
        }

        let text = match pending.take() {
            Some(previous) => format!("{previous} {trimmed}"),
            None if trimmed.starts_with("audiobook-forge ") => trimmed.to_string(),
            None => continue,
        };
        match text.strip_suffix('\\') {
            Some(head) => pending = Some(head.trim_end().to_string()),
            None => commands.push(text),
        }
    }
    commands
}

#[test]
fn documented_commands_parse() {
    let mut failures = Vec::new();
    let mut checked = 0;

    for file in documentation_files() {
        let markdown = fs::read_to_string(&file).unwrap();
        for command in documented_commands(&markdown) {
            // Command substitution and pipelines are shell, not CLI syntax.
            let command = command.split(" | ").next().unwrap();
            if command.contains("$(") {
                continue;
            }
            let words = match shell_words::split(command) {
                Ok(words) => words,
                Err(e) => {
                    failures.push(format!("{}: {command}\n  {e}", file.display()));
                    continue;
                }
            };
            let words: Vec<String> = words
                .into_iter()
                .take_while(|w| !w.starts_with('#'))
                .collect();
            // Synopsis lines such as `build [OPTIONS]` describe the shape, not a call.
            if words
                .iter()
                .any(|w| w.starts_with('[') || w == "<SUBCOMMAND>")
            {
                continue;
            }
            checked += 1;
            if let Err(e) = Cli::try_parse_from(&words) {
                use clap::error::ErrorKind;
                if matches!(e.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) {
                    continue;
                }
                let reason = e.to_string();
                failures.push(format!(
                    "{}: {command}\n  {}",
                    file.display(),
                    reason.lines().next().unwrap_or_default()
                ));
            }
        }
    }

    assert!(
        checked > 20,
        "only {checked} commands found; is the extraction broken?"
    );
    assert!(
        failures.is_empty(),
        "documented commands the CLI rejects:\n{}",
        failures.join("\n")
    );
}

#[test]
fn cli_reference_is_current() {
    let path = repo_path("docs/wiki/CLI-Reference.md");
    let generated = format!(
        "<!-- Generated from the clap definitions by tests/docs_test.rs. Do not edit by hand. -->\n\n{}",
        clap_markdown::help_markdown::<Cli>()
    );

    if std::env::var_os("UPDATE_DOCS").is_some() {
        fs::write(&path, &generated).unwrap();
        return;
    }
    let committed = fs::read_to_string(&path).unwrap_or_default();
    assert!(
        committed == generated,
        "docs/wiki/CLI-Reference.md is stale; run `UPDATE_DOCS=1 cargo test --test docs_test`"
    );
}

#[test]
fn config_template_matches_defaults() {
    let template: Config = serde_yaml::from_str(include_str!("../templates/config.yaml")).unwrap();
    let as_yaml = |config: &Config| serde_yaml::to_string(config).unwrap();
    assert_eq!(
        as_yaml(&template),
        as_yaml(&Config::default()),
        "templates/config.yaml and Config::default() disagree"
    );
}
