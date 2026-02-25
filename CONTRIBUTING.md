# Contributing to oxiddd

First off, thank you for considering contributing to `oxiddd`. As a tool designed for digital forensics and incident response (DFIR), maintaining extreme reliability and data integrity is our top priority.

## Guiding Principles

- **Integrity First**: Every change must ensure that disk data is never modified and that hashes are calculated accurately.
- **Safety over Speed**: While performance is a goal, the stability of the acquisition process is paramount.
- **Zero Dependencies**: Minimize adding new dependencies. Every new crate must be audited for security and performance impact.

## How to Contribute

### Reporting Bugs

- Use the **Bug Report** template.
- Include the exact command used.
- Specify the hardware context (disk type, connection method) if relevant.

### Proposing Features

- Use the **Feature Request** template.
- Explain the forensic value of the feature.

### Development Workflow

1.  **Fork** the repository and create your branch from `main`.
2.  **Install dependencies**: Ensure you have Rust and `musl-tools` installed.
3.  **Run Tests**: `cargo test` must pass.
4.  **Linting**: Run `cargo clippy` and `cargo fmt`. We do not accept code with clippy warnings.
5.  **No `unwrap()`**: Avoid `unwrap()` or `expect()` on potentially failing operations. Handle errors gracefully with proper forensic logging.

## Code of Conduct

This project adheres to a professional and respectful environment. By participating, you are expected to uphold this standard.

## Licensing

By contributing to `oxiddd`, you agree that your contributions will be licensed under the project's **GPL-3.0 License**.
