
# Contributing to bitmex-cli

Thank you for your interest in making bitmex-cli better! We welcome contributions of all kinds—whether it's code, documentation, bug reports, or feature ideas.


## How to Contribute

To get started, open pull requests (PRs) or issues here on GitHub. Our maintainers review submissions and aim to respond as quickly as possible. Please note that not every PR will be accepted, and some may require changes before merging. We appreciate your patience and collaboration.


## Issues and Feature Requests

Please use [GitHub Issues](https://github.com/BitMEX/bitmex-cli/issues) for bug reports and feature requests.

**Bug reports:**  
To help us resolve issues quickly, include:

- The command you ran (please redact any API keys or secrets).
- The output you received (using `-o json` for structured output is helpful).
- What you expected to happen instead.
- Your OS and architecture (`uname -a`).
- The CLI version (`bitmex --version`).

**Feature requests:**  
Tell us what you’d like the CLI to do and why. If possible, include a usage example. Prefix the title with `Feature request:` to help us find it easily.


## Submitting Changes

1. Fork the repository.
2. Create a branch from `master`.
3. Make your changes.
4. Run `cargo build` and `cargo test` locally to ensure everything works.
5. Open a pull request against `master`.


### PR Guidelines

- Keep PRs focused—one logical change per PR is best.
- Write clear, descriptive commit messages.
- Add or update tests for new behavior.
- Avoid unrelated formatting or refactoring changes.
- Enable "Allow edits from maintainers" on your PR so we can help with small adjustments if needed.


### What Makes a Good PR

- Bug fixes with a reproducer or test case.
- New commands or flags that extend existing command groups.
- Documentation improvements.
- Test coverage for previously untested code paths.


### What Will Likely Be Declined

- Large architectural changes without prior discussion (please open an issue first).
- Changes that break backward compatibility without strong justification.
- Features that require changes to BitMEX exchange infrastructure outside the CLI.


## Code Style

- Follow the existing patterns in the codebase.
- Run `cargo clippy -- -D warnings` before submitting.
- Avoid nightly-only features; the crate must build on stable Rust.


## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).


## Security

If you discover a security vulnerability, please **do not** open a public issue. Instead, follow the responsible disclosure process described in [DISCLAIMER.md](DISCLAIMER.md).
