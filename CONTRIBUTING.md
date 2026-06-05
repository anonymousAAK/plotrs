# Contributing to plotkit

Thanks for your interest in plotkit! Contributions of all kinds -- bug fixes, new chart types, performance improvements, docs -- are welcome.

## Getting Started

1. **Fork** the repository on GitHub.
2. **Clone** your fork and create a feature branch off `main`:
   ```bash
   git checkout -b feat/my-feature main
   ```
3. Make your changes, push the branch, and open a **Pull Request** against `main`.

## Development Setup

Make sure you have a recent stable Rust toolchain installed, then:

```bash
cargo build            # compile the library
cargo test             # run the full test suite
cargo fmt              # format code (must pass CI)
cargo clippy --all-targets --all-features -- -D warnings   # lint (must be warning-free)
```

All four commands must pass cleanly before you submit a PR.

## Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/). Every commit message should use one of these prefixes:

| Prefix       | Purpose                          |
|--------------|----------------------------------|
| `feat:`      | New feature                      |
| `fix:`       | Bug fix                          |
| `docs:`      | Documentation only               |
| `perf:`      | Performance improvement          |
| `refactor:`  | Code change that is not a fix or feature |
| `test:`      | Adding or updating tests         |
| `chore:`     | Maintenance / tooling            |
| `ci:`        | CI configuration changes         |

Example: `feat: add horizontal bar chart support`

## Visual / Rendering Changes

Any PR that affects rendered output (SVG, layout, colors, etc.) **must** include updated golden snapshots:

```bash
cargo insta review
```

Review each snapshot diff carefully and accept only intentional changes. The updated snapshot files should be committed alongside your code changes.

## Documentation

Every public item (`pub fn`, `pub struct`, `pub enum`, `pub trait`, etc.) requires a `///` doc comment that includes at least one doctest:

```rust
/// Adds a title to the current axes.
///
/// ```
/// # use plotkit::Figure;
/// let mut fig = Figure::new();
/// fig.axes_mut().set_title("My Plot");
/// ```
pub fn set_title(&mut self, title: &str) { /* ... */ }
```

Doctests are run as part of `cargo test`, so make sure they compile and pass.

## Pull Request Guidelines

- **One logical change per PR.** Keep PRs focused -- separate refactors from features from bug fixes.
- **CI must be green.** All checks (build, test, fmt, clippy) must pass before a PR can be merged.
- Write a clear PR description explaining *what* changed and *why*.
- If your PR closes an issue, reference it (e.g., `Closes #42`).

## Questions?

Open an issue or start a discussion on the repository -- happy to help you get started.
