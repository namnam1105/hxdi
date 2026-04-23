<div align="center">

# hxdi

![Rust](https://img.shields.io/badge/rust-stable-orange?logo=rust&style=for-the-badge)
![License](https://img.shields.io/crates/l/hxdi?style=for-the-badge)
![Version](https://img.shields.io/crates/v/hxdi?style=for-the-badge)
![Downloads](https://img.shields.io/crates/d/hxdi?style=for-the-badge)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/namnam1105/hxdi/release.yml?style=for-the-badge)

*a fast, flexible hex editor for the terminal — written in Rust with near-C dump performance and a nano-style TUI.*

</div>

---

## features

- near-C performance dumper
- interactive TUI with nano-like controls
- colored output with batched ANSI escapes
- auto-adjusting `bytes_per_row` based on terminal width
- fully configurable via CLI flags — disable any column, bypass file size warnings

## install

```bash
cargo install hxdi
```

## build from source

```bash
git clone https://github.com/namnam1105/hxdi.git
cd hxdi
cargo build --release
```

## usage
| flag | description |
|------|-------------|
| `-t, --tui-no` | disable TUI, dump to stdout (read-only) |
| `-d, --disable-header` | disable column header |
| `-c, --color-no` | disable colored output |
| `-o, --offsets-no` | disable offset column |
| `-n, --no-hex` | disable hex column |
| `-a, --ascii-no` | disable ASCII column |
| `-f, --force-large` | skip 100 MB file size warning |

## roadmap

- [x] dump mode
- [x] `ratatui` TUI (read, overwrite, insert)
- [x] publish to crates.io
- [ ] do something ig

## contributing

issues and PRs are welcome. a few things to keep in mind:

- use [conventional commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `chore:` etc.)
- run `cargo test --tests` before submitting
- keep hot-path code (`hex_read.rs`) allocation-free — no `format!()` or `Vec` inside loops
