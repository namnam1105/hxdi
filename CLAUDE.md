# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

**hexi** is a performance-focused hex-dumper written in Rust, targeting near-C performance. Dump mode is fully implemented; a TUI hex-editor mode is planned but not yet started.

## Commands

```bash
cargo build --release          # production build
cargo test --tests             # run all tests
cargo test --test hex_tests    # run a single test file
cargo test --tests -- --nocapture  # run tests with stdout visible
```

No clippy or rustfmt configuration exists; use defaults.

## Architecture

Three source modules plus an integration test suite:

- **`args.rs`** — CLI parsing (`clap` derive), input loading, and `fool_check()` validation (errors if all output columns are disabled). `read_input()` enforces a 100 MB file size limit; `--force-large` bypasses it.
- **`hex_read.rs`** — Core rendering. This is the hot-path module. `dump_hex()` wraps stdout in a 64 KB `BufWriter` and calls `print_row()` for each 16-byte row. Color output batches consecutive same-category bytes into a single ANSI escape to minimise syscalls. All format work uses stack buffers, not `format!()`.
- **`main.rs`** — Thin entry point: parse → validate → read → render.

Data flow: `Args::parse()` → `fool_check()` → `read_input()` → `dump_hex()` → `BufWriter<Stdout>`.

The only external dependency is `clap 4.6`.

## Performance Conventions

`hex_read.rs` is deliberately low-level. When editing it:
- Keep flag checks hoisted outside the row loop.
- Prefer stack-allocated byte arrays over `String`/`Vec` allocations inside hot loops.
- The `speed_tests` integration tests encode explicit throughput budgets (e.g. 512 KB in <3 s, 10 MB in <10 s) — run them after any change to the rendering path.
