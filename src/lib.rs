//! # errlog
//!
//! A fast, zero-dependency error logger for all platforms.
//!
//! ## Quick Start
//!
//! ```rust
//! use errlog::{StderrLogger, Level, set_logger, set_max_level};
//! use errlog::{error, warn, info, debug};
//!
//! fn main() {
//!     set_logger(StderrLogger::new()).unwrap();
//!     set_max_level(Level::Debug);
//!
//!     info!("Server starting on port {}", 8080);
//!     warn!("Config file missing, using defaults");
//!     error!("Failed to bind socket: {}", "address in use");
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use std::fmt;
#[cfg(feature = "std")]
use std::sync::OnceLock;
#[cfg(feature = "std")]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(feature = "std")]
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(usize)]
pub enum Level {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

impl Level {
    #[inline]
    pub fn from_usize(n: usize) -> Option<Self> {
        match n {
            1 => Some(Level::Error),
            2 => Some(Level::Warn),
            3 => Some(Level::Info),
            4 => Some(Level::Debug),
            5 => Some(Level::Trace),
            _ => None,
        }
    }

    #[inline]
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Error => "ERROR",
            Level::Warn => "WARN ",
            Level::Info => "INFO ",
            Level::Debug => "DEBUG",
            Level::Trace => "TRACE",
        }
    }
}

#[cfg(feature = "std")]
impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub struct Record<'a> {
    pub level: Level,
    pub message: &'a str,
    pub file: &'static str,
    pub line: u32,
    pub module_path: &'static str,
    pub timestamp_ms: Option<u64>,
}

pub trait Logger: Send + Sync {
    fn log(&self, record: &Record<'_>);
    fn flush(&self) {}
}

#[cfg(feature = "std")]
static LOGGER: OnceLock<Box<dyn Logger>> = OnceLock::new();

#[cfg(feature = "std")]
static MAX_LEVEL: AtomicUsize = AtomicUsize::new(Level::Warn as usize);

#[derive(Debug)]
pub struct SetLoggerError(());

#[cfg(feature = "std")]
impl fmt::Display for SetLoggerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("logger already set")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SetLoggerError {}

#[cfg(feature = "std")]
pub fn set_logger(logger: impl Logger + 'static) -> Result<(), SetLoggerError> {
    LOGGER.set(Box::new(logger)).map_err(|_| SetLoggerError(()))
}

#[cfg(feature = "std")]
#[inline]
pub fn set_max_level(level: Level) {
    MAX_LEVEL.store(level as usize, Ordering::Relaxed);
}

#[cfg(feature = "std")]
#[inline]
pub fn max_level() -> Option<Level> {
    Level::from_usize(MAX_LEVEL.load(Ordering::Relaxed))
}

#[cfg(feature = "std")]
#[inline]
pub fn enabled(level: Level) -> bool {
    (level as usize) <= MAX_LEVEL.load(Ordering::Relaxed)
}

#[cfg(feature = "std")]
#[doc(hidden)]
pub fn __private_log(
    level: Level,
    message: &str,
    file: &'static str,
    line: u32,
    module_path: &'static str,
) {
    if !enabled(level) {
        return;
    }
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as u64);

    let record = Record {
        level,
        message,
        file,
        line,
        module_path,
        timestamp_ms,
    };

    if let Some(logger) = LOGGER.get() {
        logger.log(&record);
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::__private_log(
            $crate::Level::Error,
            &format!($($arg)*),
            file!(), line!(), module_path!(),
        )
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::__private_log(
            $crate::Level::Warn,
            &format!($($arg)*),
            file!(), line!(), module_path!(),
        )
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::__private_log(
            $crate::Level::Info,
            &format!($($arg)*),
            file!(), line!(), module_path!(),
        )
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::__private_log(
            $crate::Level::Debug,
            &format!($($arg)*),
            file!(), line!(), module_path!(),
        )
    };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        $crate::__private_log(
            $crate::Level::Trace,
            &format!($($arg)*),
            file!(), line!(), module_path!(),
        )
    };
}

/// Writes every record to **stderr** in a structured
///
/// Output format:
/// ```text
/// 2024-01-15T12:34:56.789Z [ERROR] my_crate::module  Failed to open file: No such file (src/main.rs:42)
/// ```
#[cfg(feature = "std")]
pub struct StderrLogger {
    pub color: bool,
}

#[cfg(feature = "std")]
impl StderrLogger {
    pub fn new() -> Self {
        let color = std::env::var("TERM").map(|t| t != "dumb").unwrap_or(false);
        Self { color }
    }

    pub fn with_color(color: bool) -> Self {
        Self { color }
    }
}

#[cfg(feature = "std")]
impl Default for StderrLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "std")]
impl Logger for StderrLogger {
    fn log(&self, record: &Record<'_>) {
        use std::io::Write;

        let ts = match record.timestamp_ms {
            Some(ms) => format_timestamp(ms),
            None => "-----.--:--:--.---".to_string(),
        };

        let (pre, post) = if self.color {
            level_color(record.level)
        } else {
            ("", "")
        };

        let line = format!(
            "{ts} {pre}[{level}]{post} {module:<32} {msg} ({file}:{line})\n",
            ts = ts,
            pre = pre,
            level = record.level.as_str(),
            post = post,
            module = record.module_path,
            msg = record.message,
            file = record.file,
            line = record.line,
        );

        let _ = std::io::stderr().write_all(line.as_bytes());
    }

    fn flush(&self) {
        use std::io::Write;
        let _ = std::io::stderr().flush();
    }
}

pub struct NullLogger;

impl Logger for NullLogger {
    #[inline(always)]
    fn log(&self, _record: &Record<'_>) {}
}

#[cfg(feature = "std")]
fn level_color(level: Level) -> (&'static str, &'static str) {
    match level {
        Level::Error => ("\x1b[1;31m", "\x1b[0m"), // bold red
        Level::Warn => ("\x1b[1;33m", "\x1b[0m"),  // bold yellow
        Level::Info => ("\x1b[1;32m", "\x1b[0m"),  // bold green
        Level::Debug => ("\x1b[1;36m", "\x1b[0m"), // bold cyan
        Level::Trace => ("\x1b[2;37m", "\x1b[0m"), // dim white
    }
}

/// Convert milliseconds-since-epoch to `YYYY-MM-DDTHH:MM:SS.mmmZ`.
///
/// Hand-rolled with no allocations beyond the returned `String`.
#[cfg(feature = "std")]
fn format_timestamp(ms: u64) -> String {
    let secs = ms / 1000;
    let millis = ms % 1000;

    // Days since epoch
    let mut days = secs / 86400;
    let time_of_day = secs % 86400;
    let hh = time_of_day / 3600;
    let mm = (time_of_day % 3600) / 60;
    let ss = time_of_day % 60;

    // Gregorian calendar calculation
    let mut year = 1970u64;
    loop {
        let dy = days_in_year(year);
        if days < dy {
            break;
        }
        days -= dy;
        year += 1;
    }
    let mut month = 1u64;
    loop {
        let dm = days_in_month(month, year);
        if days < dm {
            break;
        }
        days -= dm;
        month += 1;
    }
    let day = days + 1;

    format!(
        "{year:04}-{month:02}-{day:02}T{hh:02}:{mm:02}:{ss:02}.{millis:03}Z",
        year = year,
        month = month,
        day = day,
        hh = hh,
        mm = mm,
        ss = ss,
        millis = millis,
    )
}

#[cfg(feature = "std")]
fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

#[cfg(feature = "std")]
fn days_in_year(y: u64) -> u64 {
    if is_leap(y) { 366 } else { 365 }
}

#[cfg(feature = "std")]
fn days_in_month(m: u64, y: u64) -> u64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(y) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct VecLogger(Arc<Mutex<Vec<String>>>);

    impl Logger for VecLogger {
        fn log(&self, record: &Record<'_>) {
            self.0.lock().unwrap().push(format!(
                "[{}] {}",
                record.level.as_str().trim(),
                record.message
            ));
        }
    }

    #[test]
    fn level_ordering() {
        assert!(Level::Error < Level::Warn);
        assert!(Level::Warn < Level::Info);
        assert!(Level::Info < Level::Debug);
        assert!(Level::Debug < Level::Trace);
    }

    #[test]
    fn level_roundtrip() {
        for n in 1..=5usize {
            let l = Level::from_usize(n).unwrap();
            assert_eq!(l as usize, n);
        }
        assert!(Level::from_usize(0).is_none());
        assert!(Level::from_usize(6).is_none());
    }

    #[test]
    fn timestamp_format() {
        // Unix epoch
        assert_eq!(format_timestamp(0), "1970-01-01T00:00:00.000Z");
        // 2024-01-15 12:34:56.789
        let ms: u64 = 1705318496789;
        let ts = format_timestamp(ms);
        assert!(ts.starts_with("2024-01-15T"));
        assert!(ts.ends_with("Z"));
    }

    #[test]
    fn null_logger_never_panics() {
        let r = Record {
            level: Level::Error,
            message: "boom",
            file: file!(),
            line: line!(),
            module_path: module_path!(),
            timestamp_ms: None,
        };
        NullLogger.log(&r); // must not panic
    }
}
