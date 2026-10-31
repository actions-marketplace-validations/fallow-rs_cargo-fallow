# Contributing to cargo-fallow

Thank you for your help. This guide covers the build, the checks and the
sign-off rule.

## Build and test

Check out the fallow repository next to this repository as
`../fallow-report-protocol`. M0 reads `fallow-report-protocol` from that path.

```bash
cargo build
cargo test
```

The tests copy each fixture under `tests/fixtures/` to a temporary directory.
A test never writes into the repository.

## Lints and format

```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo deny check
```

`Cargo.toml` enables the Clippy `pedantic`, `nursery` and `cargo` groups and a
set of restriction lints. Do not use `#[allow]` without a `reason`. Use
`writeln!` on a locked stdout or stderr, not `println!`. Do not call
`std::process::exit`.

`hk.pkl` runs `cargo fmt` and `cargo clippy` before each commit. Run
`hk install` once per clone to enable it.

## Developer Certificate of Origin

cargo-fallow uses the
[Developer Certificate of Origin](https://developercertificate.org/) (DCO).
With a sign-off, you confirm that you wrote the change or that you have the
right to submit it under the project license.

Add a `Signed-off-by` line to every commit with the `-s` flag:

```bash
git commit -s -m "fix: describe the change"
```

Git writes the line from your `user.name` and `user.email`:

```text
Signed-off-by: Your Name <you@example.com>
```

The email must match the commit author. The `-S` flag adds a cryptographic
signature. Use both: `git commit -S -s`.

## Commit messages

Use conventional commit prefixes, for example `feat:`, `fix:` and `chore:`.

## Prose rule

Write all prose in ASD-STE100 Simplified Technical English. This includes
comments, docs, commit messages and pull request text.

- Write short sentences: 20 words for an instruction, 25 for a description.
- Use the active voice and simple tenses.
- Use one word for one meaning.
- Do not use idioms or contractions.
