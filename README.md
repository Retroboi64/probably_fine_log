# probably_fine_log

A **fast, zero-dependency** error logger for Rust — works on every platform including embedded targets.

## Features

| Feature | Detail |
|---|---|
| **Zero dependencies** | Only the Rust standard library |
| **Lock-free filtering** | `AtomicUsize` max-level check — filtered calls cost ~1 ns |
| **Thread-safe** | Global logger installed with `OnceLock`; no mutex on the hot path |
| **Structured records** | File, line, module path, and millisecond timestamp on every record |
| **Pluggable** | Implement the `Logger` trait to write to files, ring buffers, sockets, … |
| **All platforms** | Linux, macOS, Windows, WASM, embedded (`no_std` with `default-features = false`) |

## Quick start

```toml
# Cargo.toml
[dependencies]
errlog = "0.1"
```

```rust
use probably_fine_log::{set_logger, set_max_level, StderrLogger, Level};
use probably_fine_log::{error, warn, info, debug, trace};

fn main() {
    // Install once at program start
    set_logger(StderrLogger::new()).unwrap();
    set_max_level(Level::Debug);

    info!("Listening on port {}", 8080);
    debug!("Connection pool size: {}", 16);
    warn!("Retry {}/{}", 2, 3);
    error!("Fatal: {}", "disk full");
}
```

Output (with colour):

```
2024-01-15T12:34:56.012Z [INFO ] my_app                           Listening on port 8080 (src/main.rs:9)
2024-01-15T12:34:56.013Z [DEBUG] my_app                           Connection pool size: 16 (src/main.rs:10)
2024-01-15T12:34:56.014Z [WARN ] my_app                           Retry 2/3 (src/main.rs:11)
2024-01-15T12:34:56.015Z [ERROR] my_app                           Fatal: disk full (src/main.rs:12)
```

## Custom logger

```rust
use probably_fine_log::{Logger, Record, set_logger};

struct MyLogger;

impl Logger for MyLogger {
    fn log(&self, record: &Record<'_>) {
        // write to a file, network socket, ring buffer…
        eprintln!("[{}] {}", record.level, record.message);
    }
    fn flush(&self) { /* optional */ }
}

fn main() {
    set_logger(MyLogger).unwrap();
}
```

## Level filtering

Filtering is a single **relaxed atomic load** — below the noise floor of any I/O:

```rust
set_max_level(Level::Warn); // only Error + Warn pass through
```

## Log levels

| Macro | Level | Numeric |
|---|---|---|
| `error!` | `Level::Error` | 1 |
| `warn!`  | `Level::Warn`  | 2 |
| `info!`  | `Level::Info`  | 3 |
| `debug!` | `Level::Debug` | 4 |
| `trace!` | `Level::Trace` | 5 |

Records with a numeric value **greater than** the current max are dropped before the logger is called.

## `no_std` support

Disable the default `std` feature to compile on bare-metal targets:

```toml
[dependencies]
probably_fine_log = { version = "0.1", default-features = false }
```

In `no_std` mode the macros and `NullLogger` are available; `StderrLogger`, timestamp formatting, and `set_logger` require `std`.

## License

MIT
