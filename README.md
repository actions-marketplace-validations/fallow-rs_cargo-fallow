# cargo-fallow

`cargo fallow` is the Rust part of Fallow. It is a Cargo subcommand that
analyzes a Cargo workspace. It uses `cargo metadata` and syntactic analysis
with `syn`. It runs on stable Rust and needs no nightly toolchain.

## Status

Pre-release, M0. The tool reads the workspace and reports two gates. It does
not report findings yet. The JSON shape can change before the first release.

## Install

Install from source only. The crate is not on crates.io.

```bash
git clone https://github.com/fallow-rs/cargo-fallow.git
cd cargo-fallow
cargo install --path .
```

M0 depends on `fallow-report-protocol` through a relative path. Check out the
fallow repository next to this repository as `../fallow-report-protocol`. CI
does the same at a pinned commit. This setup is for M0 only. A crates.io
version replaces it before the first publish.

`cargo-fallow` installs one binary, `cargo-fallow`. It never installs a binary
named `fallow`. That name belongs to the JavaScript CLI.

## Usage

```bash
cargo fallow                      # same as `cargo fallow check`
cargo fallow check --format json --quiet
cargo fallow check --manifest-path path/to/Cargo.toml
cargo fallow check --fail-on-parse-error
```

You can also run the binary directly: `cargo-fallow check`.

| Flag | Meaning |
| --- | --- |
| `--manifest-path <PATH>` | The `Cargo.toml` of the workspace. The default is the manifest that Cargo finds from the current directory. |
| `--format human\|json\|sarif` | The output format. The default is `human`. |
| `--quiet` | Hide the progress lines on stderr. |
| `--fail-on-parse-error` | Exit 1 when a target root file does not parse. |

`cargo fallow` runs `cargo metadata --no-deps`. It does not resolve
dependencies, it does not use the network and it does not write `Cargo.lock`.

## Output formats

- `human`: a short summary on stdout. Progress lines go to stderr.
- `json`: the report on stdout. The report has `schema_version`, `version`,
  `kind`, `language`, `workspace`, `total_issues`, `findings` and
  `gate_outcomes`.
- `sarif`: a SARIF 2.1.0 log on stdout. Each file of a failed gate is a
  result, and each failed gate is a tool execution notification. The level
  is `error` for an enforced failure and `warning` for other failures.

The gates are `error-severity-findings` and `parse-error`. They use the same
names as the gates of the JavaScript CLI. A gate fails the run when its
`status` is `fail` and `enforced` is `true`.

A target root file that cannot be read as UTF-8 text is not a parse error.
The tool writes a warning to stderr, and the `parse-error` gate does not
judge the file. `--quiet` does not hide the warning.

With `--format json`, an error goes to stdout as an error document:
`{ "error": true, "message": "...", "exit_code": 2, "code": "..." }`. In the
other formats, the error goes to stderr.

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | No enforced gate failed. |
| `1` | An enforced gate failed. |
| `2` | The input was not valid, or the run could not complete. |

## License

MIT. See `LICENSE`.
