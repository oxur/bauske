use std::{fmt, str::FromStr};

use crate::{ParseSegmentError, ParseSymbolError, SymbolKind, alphabet};

/// The articulatory place category used by `representatives-v1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Place {
    /// Bilabial articulation.
    Bilabial,
    /// Labiodental articulation.
    Labiodental,
    /// Alveolar articulation.
    Alveolar,
    /// Postalveolar articulation.
    Postalveolar,
    /// Velar articulation.
    Velar,
    /// Glottal articulation.
    Glottal,
}

/// The consonant manner category used by `representatives-v1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Manner {
    /// Complete closure and release.
    Plosive,
    /// Narrow turbulent constriction.
    Fricative,
    /// Nasal airflow.
    Nasal,
    /// Lateral approximant.
    LateralApproximant,
    /// Trill articulation.
    Trill,
}

/// The voicing value used by `representatives-v1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Voicing {
    /// Vocal folds are not vibrating.
    Voiceless,
    /// Vocal folds are vibrating.
    Voiced,
}

/// Vowel height category used by `representatives-v1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Height {
    /// Open vowel.
    Open,
    /// Open-mid vowel.
    OpenMid,
    /// Mid vowel.
    Mid,
    /// Close-mid vowel.
    CloseMid,
    /// Close vowel.
    Close,
}

/// Vowel backness category used by `representatives-v1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Backness {
    /// Front vowel.
    Front,
    /// Central vowel.
    Central,
    /// Back vowel.
    Back,
}

/// Vowel rounding category used by `representatives-v1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rounding {
    /// Unrounded vowel.
    Unrounded,
    /// Rounded vowel.
    Rounded,
}

/// The feature payload for one selected representative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Features {
    /// Consonant articulation coordinates.
    Consonant {
        /// Articulatory place.
        place: Place,
        /// Articulatory manner.
        manner: Manner,
        /// Voicing.
        voicing: Voicing,
    },
    /// Vowel quality coordinates.
    Vowel {
        /// Vowel height.
        height: Height,
        /// Vowel backness.
        backness: Backness,
        /// Lip rounding.
        rounding: Rounding,
    },
}

impl Features {
    /// Returns the feature-class label.
    #[must_use]
    pub const fn class(self) -> SegmentClass {
        match self {
            Self::Consonant { .. } => SegmentClass::Consonant,
            Self::Vowel { .. } => SegmentClass::Vowel,
        }
    }
}

/// One of the model's two independently defined comparison classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SegmentClass {
    /// Consonantal articulation, including apostrophe.
    Consonant,
    /// Vowel quality.
    Vowel,
}

impl fmt::Display for SegmentClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Consonant => "consonant",
            Self::Vowel => "vowel",
        })
    }
}

/// A validated alphabet symbol, including punctuation and boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AlphabetSymbol(u8);

impl AlphabetSymbol {
    /// Returns the source spelling of this symbol.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        alphabet()[usize::from(self.0)].symbol()
    }

    /// Returns the language-source symbol category.
    #[must_use]
    pub fn kind(self) -> SymbolKind {
        alphabet()[usize::from(self.0)].kind()
    }

    /// Returns the immutable registry record for this symbol.
    #[must_use]
    pub fn record(self) -> &'static crate::SymbolRecord {
        &alphabet()[usize::from(self.0)]
    }
}

impl FromStr for AlphabetSymbol {
    type Err = ParseSymbolError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.is_empty() {
            return Err(ParseSymbolError::Empty);
        }
        if input.chars().count() != 1 {
            return Err(ParseSymbolError::MultipleSymbols);
        }
        alphabet()
            .iter()
            .position(|record| record.symbol() == input)
            .map(|index| Self(index as u8))
            .ok_or(ParseSymbolError::UnknownSymbol)
    }
}

/// One validated representative with numeric features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Segment(AlphabetSymbol);

impl Segment {
    /// Returns the orthographic representative symbol.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.0.as_str()
    }

    /// Returns the comparison class for this representative.
    #[must_use]
    pub fn class(self) -> SegmentClass {
        self.features().class()
    }

    /// Returns the typed features selected for this representative.
    #[must_use]
    pub fn features(self) -> Features {
        self.0
            .record()
            .features()
            .expect("validated Segment always has representative features")
    }
}

impl FromStr for Segment {
    type Err = ParseSegmentError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.is_empty() {
            return Err(ParseSegmentError::Empty);
        }
        if input.chars().count() != 1 {
            return Err(ParseSegmentError::MultipleSymbols);
        }
        let symbol = input
            .parse::<AlphabetSymbol>()
            .map_err(|error| match error {
                ParseSymbolError::Empty => ParseSegmentError::Empty,
                ParseSymbolError::MultipleSymbols => ParseSegmentError::MultipleSymbols,
                ParseSymbolError::UnknownSymbol => ParseSegmentError::UnknownSymbol,
            })?;
        if symbol.record().features().is_none() {
            return Err(ParseSegmentError::Boundary(symbol.kind()));
        }
        Ok(Self(symbol))
    }
}
