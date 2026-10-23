use std::{error::Error, fmt};

use crate::SymbolKind;

/// A symbol string could not be resolved in the alphabet registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseSymbolError {
    /// No input was supplied.
    Empty,
    /// More than one Unicode scalar was supplied.
    MultipleSymbols,
    /// The single symbol is not in the source alphabet.
    UnknownSymbol,
}

impl fmt::Display for ParseSymbolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "empty alphabet symbol",
            Self::MultipleSymbols => "expected one alphabet symbol",
            Self::UnknownSymbol => "unknown alphabet symbol",
        })
    }
}
impl Error for ParseSymbolError {}

/// A string could not be parsed as one scored representative segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseSegmentError {
    /// No input was supplied.
    Empty,
    /// More than one Unicode scalar was supplied.
    MultipleSymbols,
    /// The single symbol is not in the source alphabet.
    UnknownSymbol,
    /// The symbol is a boundary and has no numeric segment representation.
    Boundary(SymbolKind),
}

impl fmt::Display for ParseSegmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty segment"),
            Self::MultipleSymbols => f.write_str("expected one segment symbol"),
            Self::UnknownSymbol => f.write_str("unknown segment symbol"),
            Self::Boundary(kind) => write!(f, "{kind} has no numeric segment representation"),
        }
    }
}
impl Error for ParseSegmentError {}

/// A requested comparison is undefined by the selected model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CompareError {
    /// The model defines separate consonant and vowel subspaces only.
    DifferentClasses {
        /// Class of the left representative.
        left: crate::SegmentClass,
        /// Class of the right representative.
        right: crate::SegmentClass,
    },
}

impl fmt::Display for CompareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DifferentClasses { left, right } => {
                write!(f, "comparison is undefined between {left} and {right}")
            }
        }
    }
}
impl Error for CompareError {}
