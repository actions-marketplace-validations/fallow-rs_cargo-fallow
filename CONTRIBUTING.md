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
`std::process::exit`. Only the workspace loader in `src/workspace.rs` can
spawn a process. `.clippy.toml` holds that rule.

## Quality gates

The gates are the same as in the fallow repository, where they apply.
`hk.pkl` runs the local gates. Run `hk install` once per clone to enable
the hooks. Run `npm ci` once to install commitlint for the commit-msg hook.

| Gate | Command | Hook | CI |
| --- | --- | --- | --- |
| Format | `cargo fmt --all -- --check` | pre-commit, pre-push | `ci.yml` |
| Clippy | `cargo clippy --all-targets --locked -- -D warnings` | pre-commit, pre-push | `ci.yml` |
| Tests | `cargo nextest run --locked` or `cargo test --locked` | none | `ci.yml` |
| Documentation | `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items` | pre-push | `ci.yml` |
| Typos | `typos` | pre-commit | `ci.yml` |
| Hidden unicode | `python3 scripts/scan-hidden-unicode.py --mode committed` | pre-commit | `ci.yml` |
| Conflict markers | `sh scripts/check-conflict-markers.sh` | pre-commit | `ci.yml` |
| Narrator comments | `node scripts/check-comment-quality.mjs --all` | pre-commit | `ci.yml` |
| File size | `node scripts/check-file-size.mjs --base origin/main` | pre-commit | `ci.yml` |
| Script tests | `npm run test:scripts` | none | `ci.yml` |
| Dependency policy | `cargo deny check` | none | `ci.yml` |
| Security advisories | `cargo audit` | none | `ci.yml` |
| Unused dependencies | `cargo shear` | pre-push | `ci.yml` |
| Workflow security | `actionlint` and `zizmor .github/` | none | `ci.yml` |
| MSRV | `cargo +1.97.0 check --locked` | none | `ci.yml` |
| Commit messages | `commitlint` | commit-msg | `commitlint.yml` |
| DCO sign-off | `node .github/scripts/check-dco.mjs` | commit-msg | `commitlint.yml` |
| Line coverage | `cargo llvm-cov` | none | `coverage.yml`, floor 92.0% (same as fallow) |
| Visibility audit | `scripts/check-hawk.sh` | none | `hawk.yml`, weekly |

A file over 2000 lines must not grow. Move new code to a new module.

The tools are `typos`, `cargo-shear`, `cargo-deny`, `cargo-audit`,
`cargo-nextest`, `cargo-llvm-cov`, `actionlint`, `zizmor`, Node.js 22 and
Python 3. The Hawk audit needs `cargo hawk` 0.1.9 on toolchain 1.97.1.

### Gates that this repository does not have yet

These fallow gates do not apply now:

- Miri: the crate has no unsafe code, and `unsafe_code` is `forbid`.
- Fuzz: the crate does not parse untrusted input yet.
- The napi, VS Code, Zed, Docker and npm jobs: the crate has no such
  surface.
- CodeQL and OpenSSF Scorecard: both need the public GitHub repository.
- Benchmarks: the crate has no hot path to measure yet.
- Mutation testing: M1 adds it, and uses it on this crate first.

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
