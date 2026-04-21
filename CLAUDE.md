# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

**hexi** is a performance-focused hex-dumper and interactive TUI hex editor written in Rust, targeting near-C performance. Dump mode is fully implemented. TUI mode is implemented but incomplete (dialogs, editing, find/goto not yet wired).

## Commands

```bash
cargo build --release          # production build
cargo test --tests             # run all tests
cargo test --test hex_tests    # run a single test file
cargo test --tests -- --nocapture  # run tests with stdout visible
```

No clippy or rustfmt configuration exists; use defaults.

## Architecture

### Modules

- **`args.rs`** — CLI parsing (`clap` derive), input loading, `fool_check()` (errors if all output columns disabled). `read_input()` enforces 100 MB limit; `--force-large` bypasses it.
- **`hex_read.rs`** — Dump mode hot path. `dump_hex()` wraps stdout in 64 KB `BufWriter`, calls `print_row()` per 16-byte row. Color output batches consecutive same-category bytes into single ANSI escapes. All format work uses stack buffers, not `format!()`.
- **`main.rs`** — Entry point: parse → `fool_check()` → read → branch on `tui_no` flag.
- **`tui/`** — Interactive editor (ratatui + crossterm).

### TUI module (`src/tui/`)

| File | Responsibility |
|------|---------------|
| `mod.rs` | Terminal setup/teardown, `run_loop()` (16 ms poll, ~60 fps) |
| `app.rs` | `App` struct — all mutable editor state, no ratatui imports |
| `types.rs` | All enums: `EditMode`, `ActivePane`, `NibbleHalf`, `Dialog`, `UnsavedFocus`, `SearchMode`, `GotoMode`, plus `FindState`/`GotoState` |
| `render.rs` | `draw()` dispatcher; layout computation; all ratatui rendering |
| `events.rs` | `handle_event()` — dispatches by `app.dialog` then `app.active_pane`; mouse handling; copy via OSC 52 |
| `actions.rs` | `Action` enum returned by event handler to `run_loop` |

### Data flow

**Dump mode:** `Args::parse()` → `fool_check()` → `read_input()` → `dump_hex()` → `BufWriter<Stdout>`

**TUI mode:** `Args::parse()` → `read_input()` → `tui::run()` → `run_loop` → `render::draw()` / `events::handle_event()`

### Layout (TUI)

5 vertical bands: `title(1) | separator(1) | header(0-1) | editor(fill) | hints(1)`

Editor band horizontal split: `offset(dynamic) | hex(Min, stretches) | sep(0-1) | ascii(bpr+2)`

`bytes_per_row` is computed each frame from actual hex pane width: `(hex_inner + 1) / 3`. `bpr_override: Option<usize>` locks it (`,`/`.` keys, mouse drag on separator).

### Key state in `App`

- `sel_anchor: Option<usize>` + `cursor: usize` → selection range via `sel_range()`
- `bpr_override: Option<usize>` — None = auto-fill from width
- `offset_extra: i16` — added to auto-computed offset digit count
- `status_msg_until: Option<Instant>` — cleared by `run_loop` on expiry
- Hit-test bounds (`hex_content_x/w`, `ascii_content_x/w`, `editor_content_y`) written each frame by `render::draw()`, read by `events.rs` for mouse-to-byte conversion

### Performance conventions (`hex_read.rs`)

- Keep flag checks hoisted outside the row loop.
- Prefer stack-allocated byte arrays over `String`/`Vec` inside hot loops.
- `speed_tests` integration tests encode explicit throughput budgets — run after any change to the rendering path.

### Unimplemented stubs

`Dialog::Find`, `Dialog::Goto`, `Dialog::UnsavedChanges` are defined in `types.rs` and triggered by keybinds but their render/event handlers are stubs (`_ => Action::None`). `EditMode::Insert` and `EditMode::ReadOnly` are defined but never constructed.
