//! Error types for plotkit.

use std::fmt;

/// Errors that can occur in plotkit operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum PlotError {
    /// X and Y series have different lengths.
    SeriesLengthMismatch {
        /// Expected length (from the first series).
        expected: usize,
        /// Actual length of the mismatched series.
        got: usize,
    },
    /// Data series is empty.
    EmptyData,
    /// Referenced column not found (for DataFrame integrations).
    UnknownColumn(String),
    /// An I/O error occurred.
    Io(std::io::Error),
    /// The requested output format is not supported.
    UnsupportedFormat(String),
    /// Failed to load or render with a font.
    FontLoad(String),
}

impl fmt::Display for PlotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SeriesLengthMismatch { expected, got } => {
                write!(f, "series length mismatch: expected {expected}, got {got}")
            }
            Self::EmptyData => write!(f, "data series is empty"),
            Self::UnknownColumn(name) => write!(f, "unknown column: {name}"),
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::UnsupportedFormat(fmt_str) => write!(f, "unsupported format: {fmt_str}"),
            Self::FontLoad(msg) => write!(f, "font loading error: {msg}"),
        }
    }
}

impl std::error::Error for PlotError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for PlotError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

/// A specialized Result type for plotkit operations.
pub type Result<T> = std::result::Result<T, PlotError>;
