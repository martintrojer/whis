# Part V: CLI Application

With the core library understood, let's explore how `whis-cli` uses it to provide a command-line interface.

## What You'll Learn

This part covers the terminal-based application:

- **Chapter 16: CLI Structure** - clap derive macros, subcommands, argument parsing
- **Chapter 17: Hotkey System** - Platform-specific global keyboard shortcuts
- **Chapter 18: IPC** - Unix socket communication for daemon control

## The CLI's Role

The `whis-cli` crate provides:

1. **One-shot recording** - `whis record` for terminal users
2. **Background daemon** - `whis serve` for hotkey-based recording
3. **Daemon control** - `whis stop` and `whis status` commands

```admonish info
The CLI demonstrates a key pattern: **platform abstraction**. Linux uses `rdev` for hotkeys, while other platforms use `global-hotkey`.
```

## Code Organization

Files in `crates/whis-cli/src/`:

- `main.rs` - Entry point and clap command router
- `commands/*.rs` - Individual command implementations
- `hotkey/mod.rs` - Platform abstraction layer
- `ipc.rs` - Unix socket message passing

## Time Estimate

- **Quick read**: ~45 minutes
- **Thorough read with code exploration**: ~2 hours
- **With exercises**: +45 minutes

---

Let's begin with [Chapter 16: CLI Structure with clap](./ch16-clap.md).
