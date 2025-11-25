# Deimos Rising Port

This is a port of the classic game **Deimos Rising** to Rust, using the retrowin32 emulator as a base. The approach allows you to gradually replace x86 functions with Rust implementations while keeping the game running.

## Overview

This project uses retrowin32's x86 emulator to run the original Deimos Rising executable, while selectively replacing specific functions with Rust implementations. This allows you to:

- Port functions incrementally from Ghidra decompilation to Rust
- Call back into original x86 code when needed
- Directly read and modify game memory
- Keep the upstream retrowin32 clean and trackable

## Building

```bash
cargo build -p deimos-rising --profile=lto
```

## Running

```bash
cargo run -p deimos-rising --profile=lto -- path/to/DeimosRising.exe
```

### Command-line Options

```
  -C, --chdir       change working directory before running
  --win32-trace     winapi systems to trace; see HACKING.md for docs
  --debug           enable debug logging
  --help, help      display usage information
```

## Architecture

### Project Structure

```
deimos-rising/
├── src/
│   ├── game/         # Game implementation
│   ├── main.rs       # Entry point, CLI handling
│   ├── hooks.rs      # Function hooks/replacements
│   ├── host.rs       # Host OS integration (copied from cli)
│   ├── fs.rs         # File system support
│   ├── time.rs       # Time/ticks support
│   └── sdl.rs        # SDL graphics/input support
└── Cargo.toml
```

### How Hooks Work

The hooking system patches a breakpoint (`int3` instruction) at the target address. When execution reaches that address:

1. The x86 emulator triggers a `DebugBreak` state
2. The machine checks if there's a hook registered for that address
3. If yes, your Rust function is called with `&mut Machine`
4. Your function:
   - Reads arguments from the stack (ESP points to return address)
   - Implements the function logic in Rust
   - Sets the return value in EAX
   - Adjusts ESP to clean up arguments (for stdcall)
   - Sets EIP to the return address
   - Returns `true` to indicate it handled the call
