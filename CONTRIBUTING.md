# Contributing to Soteric

Thank you for your interest in contributing to Soteric! We welcome contributions from the community, whether it's bug reports, feature requests, documentation improvements, or code changes.

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)
- For macOS features: Xcode development tools

### Setting Up Your Environment

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/yourusername/Soteric.git
   cd Soteric
   ```
3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/harsh-seth/Soteric.git
   ```
4. Build the project:
   ```bash
   cargo build
   ```
5. Run tests:
   ```bash
   cargo test
   ```

## Development Workflow

### Before You Start

1. Check existing [issues](https://github.com/harsh-seth/Soteric/issues) and [pull requests](https://github.com/harsh-seth/Soteric/pulls) to avoid duplicate work
2. For new features, open an issue to discuss your approach before implementing
3. Keep commits focused and atomic

### Code Style

- Follow Rust conventions enforced by `rustfmt`:
  ```bash
  cargo fmt
  ```
- Lint your code with `clippy`:
  ```bash
  cargo clippy -- -D warnings
  ```
- Write descriptive commit messages explaining the "why" behind changes

### Testing

- Add tests for new functionality
- Ensure all tests pass locally:
  ```bash
  cargo test
  ```
- Test on macOS if your changes touch biometric authentication

## Submitting Changes

### Pull Request Process

1. Create a feature branch from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```
2. Make your changes and commit them with clear messages
3. Format and lint your code:
   ```bash
   cargo fmt
   cargo clippy
   ```
4. Push to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```
5. Open a pull request against the main repository with:
   - A clear title describing your change
   - A description of what changed and why
   - Reference to any related issues (e.g., "Fixes #123")
   - Evidence that tests pass locally

### PR Guidelines

- Keep PRs focused on a single feature or fix
- Keep changes minimal and easy to review
- Respond to feedback promptly
- Rebase and force-push to resolve conflicts (don't merge main into your branch)

## Reporting Issues

### Bug Reports

Include:
- Expected behavior
- Actual behavior
- Steps to reproduce
- OS and Rust version (`rustc --version`)
- Relevant code snippets or configuration

### Feature Requests

Include:
- Use case and motivation
- Proposed API or behavior
- Alternative approaches considered

## Code of Conduct

- Be respectful and inclusive
- Welcome diverse perspectives
- Report unacceptable behavior to the maintainers

## License

By contributing to Soteric, you agree that your contributions will be licensed under the same license as the project (check LICENSE file for details).

## Questions?

Feel free to open a discussion issue or reach out to the maintainers directly. We're here to help!

---

Thank you for making Soteric better!
