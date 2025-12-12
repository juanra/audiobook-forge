# Contributing to Audiobook Forge 🎧

First off, thank you for considering contributing to Audiobook Forge! It's people like you that make this tool better for everyone.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Code Style Guidelines](#code-style-guidelines)
- [Testing](#testing)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Features](#suggesting-features)

---

## 📜 Code of Conduct

This project and everyone participating in it is governed by our Code of Conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

### Our Standards

- **Be respectful** and inclusive
- **Be collaborative** and constructive
- **Focus on what is best** for the community
- **Show empathy** towards other community members

---

## 🚀 Getting Started

### Prerequisites

Before contributing, make sure you have:

- **Rust 1.75+** installed ([rustup.rs](https://rustup.rs/))
- **Git** for version control
- **FFmpeg, AtomicParsley, and MP4Box** installed
- Familiarity with Rust and async programming (Tokio)

### Quick Setup

```bash
# Fork and clone the repository
git clone https://github.com/YOUR_USERNAME/audiobook-forge
cd audiobook-forge

# Build the project
cargo build

# Run tests
cargo test

# Run the binary
cargo run -- check
```

---

## 🛠️ Development Setup

### 1. Fork the Repository

Click the "Fork" button at the top of the repository page.

### 2. Clone Your Fork

```bash
git clone https://github.com/YOUR_USERNAME/audiobook-forge
cd audiobook-forge
```

### 3. Add Upstream Remote

```bash
git remote add upstream https://github.com/juanra/audiobook-forge
git fetch upstream
```

### 4. Create a Branch

```bash
git checkout -b feature/my-awesome-feature
```

### 5. Install Development Tools

```bash
# Install rustfmt (code formatter)
rustup component add rustfmt

# Install clippy (linter)
rustup component add clippy

# Install cargo-watch (optional, for auto-rebuild)
cargo install cargo-watch
```

### 6. Build and Test

```bash
# Build
cargo build

# Run tests
cargo test

# Run clippy
cargo clippy -- -D warnings

# Format code
cargo fmt
```

---

## 🤝 How to Contribute

### Types of Contributions

We welcome various types of contributions:

1. **Bug fixes** - Fix issues in the codebase
2. **New features** - Add new functionality
3. **Documentation** - Improve README, docs, or code comments
4. **Tests** - Add or improve test coverage
5. **Performance** - Optimize existing code
6. **Refactoring** - Improve code quality without changing behavior

### Contribution Workflow

1. **Find or create an issue** - Check if an issue exists, or create one
2. **Discuss the approach** - Comment on the issue to discuss your plan
3. **Fork and branch** - Create a feature branch from `master`
4. **Implement changes** - Write code following our style guide
5. **Add tests** - Ensure new code is tested
6. **Run checks** - Format, lint, and test your code
7. **Commit** - Use conventional commit messages
8. **Push and PR** - Open a pull request with a clear description

---

## 🎨 Code Style Guidelines

### Rust Style

Follow the official [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/):

- Use `rustfmt` for formatting (run `cargo fmt` before committing)
- Use `clippy` for linting (run `cargo clippy`)
- Follow Rust naming conventions:
  - `snake_case` for functions, variables, modules
  - `PascalCase` for types, traits, enums
  - `SCREAMING_SNAKE_CASE` for constants

### Code Organization

```rust
// Order of items in a module:
// 1. Imports
// 2. Type definitions
// 3. Constants
// 4. Public functions
// 5. Private functions
// 6. Tests

use std::path::Path;
use anyhow::Result;

pub struct MyType {
    field: String,
}

const MAX_SIZE: usize = 1024;

pub fn public_function() -> Result<()> {
    // Implementation
    Ok(())
}

fn private_helper() {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Test implementation
    }
}
```

### Error Handling

- Use `anyhow::Result<T>` for application errors
- Use `thiserror` for custom error types
- Add context to errors with `.context()`

```rust
use anyhow::{Context, Result};

fn process_file(path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .context(format!("Failed to read file: {}", path.display()))?;

    // Process content...

    Ok(())
}
```

### Documentation

- Add doc comments to public APIs
- Use `///` for item documentation
- Use `//!` for module documentation
- Include examples in doc comments when helpful

```rust
/// Processes an audiobook directory and converts it to M4B format.
///
/// # Arguments
///
/// * `path` - Path to the audiobook directory
/// * `config` - Configuration options for processing
///
/// # Returns
///
/// Returns `Ok(ProcessingResult)` on success, or an error if processing fails.
///
/// # Example
///
/// ```no_run
/// let result = process_audiobook(Path::new("/path/to/book"), &config)?;
/// println!("Processed: {}", result.output_path.display());
/// ```
pub fn process_audiobook(path: &Path, config: &Config) -> Result<ProcessingResult> {
    // Implementation
}
```

### Async/Await

- Use `async/await` for I/O operations
- Use `tokio::spawn` for parallel tasks
- Use semaphores for resource limiting

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

async fn process_parallel(books: Vec<Book>, max_parallel: usize) -> Result<()> {
    let semaphore = Arc::new(Semaphore::new(max_parallel));

    let mut handles = vec![];

    for book in books {
        let permit = semaphore.clone().acquire_owned().await?;

        let handle = tokio::spawn(async move {
            let result = process_book(book).await;
            drop(permit);
            result
        });

        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await??;
    }

    Ok(())
}
```

---

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run tests in a specific module
cargo test models::
```

### Writing Tests

- Add unit tests in the same file as the code (`#[cfg(test)] mod tests`)
- Add integration tests in `tests/` directory
- Use descriptive test names: `test_<what>_<condition>_<expected_result>`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_profile_comparison_higher_bitrate_wins() {
        let q1 = QualityProfile {
            bitrate: 128000,
            sample_rate: 44100,
            channels: 2,
            codec: "mp3".to_string(),
            duration: 0.0,
        };

        let q2 = QualityProfile {
            bitrate: 192000,
            sample_rate: 44100,
            channels: 2,
            codec: "mp3".to_string(),
            duration: 0.0,
        };

        assert!(q2.is_better_than(&q1));
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### Test Coverage

- All public functions should have tests
- Test both success and failure cases
- Test edge cases (empty input, invalid data, etc.)
- Use `tempfile` crate for temporary files/directories in tests

---

## 📝 Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/) specification:

### Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `refactor`: Code refactoring (no behavior change)
- `perf`: Performance improvements
- `chore`: Build process, dependencies, etc.
- `style`: Code style changes (formatting, etc.)

### Examples

```bash
# Feature
git commit -m "feat(audio): add CUE file parsing support"

# Bug fix
git commit -m "fix(metadata): handle missing album tag gracefully"

# Documentation
git commit -m "docs(readme): update installation instructions"

# Breaking change
git commit -m "feat(config): change default parallel workers

BREAKING CHANGE: Default parallel workers changed from 4 to CPU cores / 2"
```

### Guidelines

- Use present tense ("add feature" not "added feature")
- Use imperative mood ("move cursor to..." not "moves cursor to...")
- Limit first line to 72 characters
- Reference issues: "fixes #123" or "closes #456"

---

## 🔄 Pull Request Process

### Before Submitting

1. **Update documentation** if you changed APIs
2. **Add tests** for new functionality
3. **Run all checks**:
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test
   cargo build --release
   ```
4. **Update CHANGELOG.md** if applicable

### PR Description Template

```markdown
## Description
Brief description of what this PR does.

## Motivation
Why is this change needed?

## Changes
- Change 1
- Change 2
- Change 3

## Testing
How was this tested?

## Screenshots (if applicable)
Add screenshots or output examples

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Comments added for complex code
- [ ] Documentation updated
- [ ] Tests added/updated
- [ ] All tests passing
- [ ] No new warnings from clippy
- [ ] CHANGELOG.md updated (if applicable)

## Related Issues
Fixes #123
Closes #456
```

### Review Process

1. **Automated checks** must pass (CI/CD)
2. **Code review** by at least one maintainer
3. **Changes requested** - address feedback and push updates
4. **Approval** - maintainer approves PR
5. **Merge** - maintainer merges into master

### After Merge

- Delete your feature branch
- Pull latest changes from upstream
- Continue with next contribution!

---

## 🐛 Reporting Bugs

### Before Reporting

1. **Check existing issues** - your bug might already be reported
2. **Try latest version** - bug might already be fixed
3. **Check dependencies** - run `audiobook-forge check`

### Bug Report Template

When creating a bug report, include:

```markdown
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce:
1. Run command '...'
2. With these files '...'
3. See error

**Expected behavior**
What you expected to happen.

**Actual behavior**
What actually happened.

**Output/Logs**
```
Paste error output here (use --verbose for detailed logs)
```

**Environment**
- OS: [e.g., macOS 13.0, Ubuntu 22.04]
- Rust version: [output of `rustc --version`]
- audiobook-forge version: [output of `audiobook-forge --version`]
- FFmpeg version: [output of `ffmpeg -version`]

**Additional context**
Any other relevant information.
```

---

## 💡 Suggesting Features

### Before Suggesting

1. **Check existing issues/discussions** - feature might already be proposed
2. **Consider scope** - does it fit the project's goals?
3. **Think about implementation** - is it technically feasible?

### Feature Request Template

```markdown
**Feature Description**
Clear description of the feature.

**Use Case**
What problem does this solve? Who benefits?

**Proposed Solution**
How could this be implemented?

**Alternatives Considered**
What other approaches did you consider?

**Additional Context**
Mockups, examples, references, etc.
```

---

## 📚 Additional Resources

### Documentation

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Documentation](https://tokio.rs/tokio/tutorial)
- [Clap Documentation](https://docs.rs/clap/)
- [Project Documentation](docs/)

### Getting Help

- **GitHub Discussions**: Ask questions, share ideas
- **GitHub Issues**: Report bugs, request features
- **Documentation**: Check `docs/` folder and `AGENTS.md`

---

## 🎯 Good First Issues

Looking for a place to start? Check out issues labeled:
- `good first issue` - Easy issues for newcomers
- `help wanted` - Issues where we need help
- `documentation` - Documentation improvements

---

## 🙏 Recognition

Contributors will be recognized in:
- GitHub contributors page
- CHANGELOG.md for significant contributions
- Acknowledgments section in README.md

---

Thank you for contributing to Audiobook Forge! 🎧✨
