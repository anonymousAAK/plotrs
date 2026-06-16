# Contributing

Contributions of all kinds — bug fixes, new chart types, performance work, and
documentation — are welcome. The authoritative guide lives in
[CONTRIBUTING.md](https://github.com/anonymousAAK/plotrs/blob/main/CONTRIBUTING.md)
at the repository root; this page summarises the essentials.

## Development setup

Work on a feature branch off `main`, then run the full local check before
opening a pull request:

```bash
cargo build            # compile the workspace
cargo test             # run the full test suite (including doctests)
cargo fmt              # format (must pass CI)
cargo clippy --all-targets --all-features -- -D warnings   # lint, warning-free
```

All four must pass cleanly before a PR is ready.

## Visual / rendering changes

plotkit uses golden snapshot tests (via `insta`) to guard rendered output. Any
change that affects what a chart looks like — layout, colors, SVG structure,
ticks — will change a snapshot. Review and accept the diffs with:

```bash
cargo insta review
```

Inspect each diff carefully and accept only intentional changes, then commit
the updated snapshot files alongside your code.

## Documentation and tests

- Every public item (`pub fn`, `pub struct`, `pub enum`, `pub trait`) needs a
  `///` doc comment with at least one doctest. Doctests run under
  `cargo test`, so they must compile and pass.
- Add or update tests for the behaviour you change. Keep each PR to one logical
  change.

## Commit convention

Commits follow [Conventional Commits](https://www.conventionalcommits.org/):
`feat:`, `fix:`, `docs:`, `perf:`, `refactor:`, `test:`, `chore:`, `ci:`.

```text
feat: add horizontal bar chart support
```

## Pull requests

- Keep PRs focused — one logical change each.
- Ensure CI is green (build, test, fmt, clippy).
- Describe *what* changed and *why*; reference any issue you close
  (e.g. `Closes #42`).

Questions? Open an issue or start a discussion on the repository.
