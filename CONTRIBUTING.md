# Contributing to RustyImage

Thank you for your interest in contributing to **RustyImage**! We welcome bug reports, feature suggestions, documentation improvements, and pull requests.

## How to Contribute

1. **Fork & Clone**
   - Fork the repository on GitHub.
   - Clone your fork locally:
     ```bash
     git clone <your-fork-url>
     cd rustyimage
     ```

2. **Branching**
   - Create a feature or bugfix branch from `main`:
     ```bash
     git checkout -b feature/my-cool-feature
     ```

3. **Development & Verification**
   - Ensure Rust 1.80+ is installed (`rustup update`).
   - Run tests before making changes:
     ```bash
     cargo test
     ```
   - Keep code clean and idiomatic. Run linter and formatting checks:
     ```bash
     cargo clippy
     cargo fmt --check
     ```

4. **Submitting a Pull Request**
   - Push your branch to your fork.
   - Open a Pull Request against main.
   - Describe the motivation, changes made, and test verification performed.

## Reporting Issues

If you encounter bugs or have feature requests:
- Check existing GitHub Issues first.
- If not already reported, open a new issue with steps to reproduce, sample image file format, and environment details (Windows version, display scaling).

## Code of Conduct

Please keep discussions constructive, polite, and welcoming to all contributors.
