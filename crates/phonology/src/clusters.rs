//! Source-grounded consonant-cluster inspection.

use std::{error::Error, fmt};

use crate::WordKind;

const CONSONANTS: &str = "bcdfgjklmnprstvxz";
const SIBILANTS: &str = "cjsz";
const SONORANTS: &str = "lmnr";
const UNVOICED: &str = "ptkfcsx";
const VOICED: &str = "bdgvjz";
const SEC36: &[&str] = &["3.6"];
const SEC37: &[&str] = &["3.7"];
const SEC36_37: &[&str] = &["3.6", "3.7"];

const INITIAL_PAIRS: [&str; 48] = [
    "bl", "br", "cf", "ck", "cl", "cm", "cn", "cp", "cr", "ct", "dj", "dr", "dz", "fl", "fr", "gl",
    "gr", "jb", "jd", "jg", "jm", "jv", "kl", "kr", "ml", "mr", "pl", "pr", "sf", "sk", "sl", "sm",
    "sn", "sp", "sr", "st", "tc", "tr", "ts", "vl", "vr", "xl", "xr", "zb", "zd", "zg", "zm", "zv",
];
const INITIAL_TRIPLES: [&str; 36] = [
    "cfr", "cfl", "sfr", "sfl", "jvr", "jvl", "zvr", "zvl", "cpr", "cpl", "spr", "spl", "jbr",
    "jbl", "zbr", "zbl", "ctr", "str", "jdr", "zdr", "ckr", "ckl", "skr", "skl", "jgr", "jgl",
    "zgr", "zgl", "cmr", "cml", "smr", "sml", "jmr", "jml", "zmr", "zml",
];
const FORBIDDEN_MEDIAL_TRIPLES: [&str; 4] = ["ndj", "ndz", "ntc", "nts"];

/// The consonant cluster's position in a declared word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ClusterPosition {
    /// At the beginning of the word.
    Initial,
    /// Between written word material on both sides.
    Medial,
    /// At the end of the word.
    Final,
}

/// A source-grounded reason that a cluster requires attention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ClusterViolation {
    /// Two adjacent consonant letters are identical.
    DuplicateNeighbors,
    /// Adjacent obstruents mix voiced and unvoiced categories.
    VoicingMismatch,
    /// Both adjacent letters belong to c, j, s, z.
    MultipleSibilants,
    /// An adjacent pair is cx, kx, xc, or xk.
    ForbiddenXPair,
    /// A non-name initial pair is absent from the source list.
    InitialPairUnlisted,
    /// A non-name initial triple is absent from the source list.
    InitialTripleUnlisted,
    /// A medial triple's last pair is not a listed initial pair.
    MedialLastPairUnlisted,
    /// A medial triple is one of ndj, ndz, ntc, or nts.
    ForbiddenMedialTriple,
    /// A non-name word ends in a consonant.
    NonNameFinalConsonant,
    /// A non-name cluster exceeds the supported source contexts.
    UnsupportedLength,
}

impl ClusterViolation {
    /// Returns the stable diagnostic code for this reason.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::DuplicateNeighbors => "cluster.duplicate-neighbors",
            Self::VoicingMismatch => "cluster.voicing-mismatch",
            Self::MultipleSibilants => "cluster.multiple-sibilants",
            Self::ForbiddenXPair => "cluster.forbidden-x-pair",
            Self::InitialPairUnlisted => "cluster.initial-pair-unlisted",
            Self::InitialTripleUnlisted => "cluster.initial-triple-unlisted",
            Self::MedialLastPairUnlisted => "cluster.medial-last-pair-unlisted",
            Self::ForbiddenMedialTriple => "cluster.forbidden-medial-triple",
            Self::NonNameFinalConsonant => "cluster.non-name-final-consonant",
            Self::UnsupportedLength => "cluster.unsupported-length",
        }
    }

    /// Returns the grammar sections supporting this reason.
    #[must_use]
    pub const fn source_sections(self) -> &'static [&'static str] {
        match self {
            Self::DuplicateNeighbors
            | Self::VoicingMismatch
            | Self::MultipleSibilants
            | Self::ForbiddenXPair
            | Self::NonNameFinalConsonant => SEC36,
            Self::InitialPairUnlisted
            | Self::InitialTripleUnlisted
            | Self::MedialLastPairUnlisted => SEC37,
            Self::ForbiddenMedialTriple => SEC37,
            Self::UnsupportedLength => SEC36_37,
        }
    }
}

/// The unresolved §3.6/§3.7 description of four consonants in non-name medial context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClusterSourceTension {
    last_three_initial_triple: Option<&'static str>,
}

impl ClusterSourceTension {
    /// Returns the stable source-tension diagnostic code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        "analysis.source-length-tension"
    }

    /// Returns both conflicting grammar sections.
    #[must_use]
    pub const fn source_sections(self) -> &'static [&'static str] {
        SEC36_37
    }

    /// Returns a listed onset-triple witness among the final three consonants.
    #[must_use]
    pub const fn last_three_initial_triple(self) -> Option<&'static str> {
        self.last_three_initial_triple
    }
}

/// A checked cluster inspection with local findings and source-length tension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterInspection {
    cluster: String,
    position: ClusterPosition,
    kind: WordKind,
    violations: Vec<ClusterViolation>,
    source_length_tension: Option<ClusterSourceTension>,
}

impl ClusterInspection {
    /// Returns the exact cluster spelling supplied to the inspector.
    #[must_use]
    pub fn cluster(&self) -> &str {
        &self.cluster
    }

    /// Returns the declared cluster position.
    #[must_use]
    pub const fn position(&self) -> ClusterPosition {
        self.position
    }

    /// Returns the caller-declared word class.
    #[must_use]
    pub const fn word_kind(&self) -> WordKind {
        self.kind
    }

    /// Returns every source-rule finding in deterministic rule order.
    #[must_use]
    pub fn violations(&self) -> &[ClusterViolation] {
        &self.violations
    }

    /// Reports the unresolved §3.6/§3.7 four-consonant length tension.
    #[must_use]
    pub const fn has_source_length_tension(&self) -> bool {
        self.source_length_tension.is_some()
    }

    /// Returns the typed source tension, including its source sections and witness.
    #[must_use]
    pub const fn source_length_tension(&self) -> Option<ClusterSourceTension> {
        self.source_length_tension
    }

    /// Returns a listed onset-triple witness in the cluster's last three letters.
    #[must_use]
    pub const fn last_three_initial_triple(&self) -> Option<&'static str> {
        match self.source_length_tension {
            Some(tension) => tension.last_three_initial_triple(),
            None => None,
        }
    }
}

/// A malformed consonant-cluster request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ClusterError {
    /// No consonants were supplied.
    Empty,
    /// The byte offset contains a character outside the 17-letter inventory.
    InvalidSymbol {
        /// Original UTF-8 byte offset.
        offset: usize,
        /// The unaccepted character.
        found: char,
    },
}

impl fmt::Display for ClusterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty consonant cluster"),
            Self::InvalidSymbol { offset, found } => {
                write!(f, "invalid consonant {found:?} at byte {offset}")
            }
        }
    }
}
impl Error for ClusterError {}

/// Returns the 48 source-listed onset pairs in their original order.
#[must_use]
pub const fn initial_pairs() -> &'static [&'static str] {
    &INITIAL_PAIRS
}

/// Returns the 36 source-listed onset triples in their original order.
#[must_use]
pub const fn initial_triples() -> &'static [&'static str] {
    &INITIAL_TRIPLES
}

/// Inspects one consonant run under a caller-declared context and word class.
///
/// # Errors
///
/// Returns [`ClusterError::Empty`] for empty input or
/// [`ClusterError::InvalidSymbol`] when a character is not one of the 17
/// lowercase morphological consonants.
pub fn inspect_cluster(
    input: &str,
    position: ClusterPosition,
    kind: WordKind,
) -> Result<ClusterInspection, ClusterError> {
    if input.is_empty() {
        return Err(ClusterError::Empty);
    }
    for (offset, ch) in input.char_indices() {
        if !ch.is_ascii() || !CONSONANTS.contains(ch) {
            return Err(ClusterError::InvalidSymbol { offset, found: ch });
        }
    }

    let chars: Vec<char> = input.chars().collect();
    let is_name = kind == WordKind::Cmevla;
    let mut violations = Vec::new();
    for pair in chars.windows(2) {
        for reason in pair_violations(pair[0], pair[1]) {
            push_once(&mut violations, reason);
        }
    }

    if !is_name {
        match position {
            ClusterPosition::Initial if chars.len() == 2 => {
                if !INITIAL_PAIRS.contains(&input) {
                    push_once(&mut violations, ClusterViolation::InitialPairUnlisted);
                }
            }
            ClusterPosition::Initial if chars.len() == 3 => {
                if !INITIAL_TRIPLES.contains(&input) {
                    push_once(&mut violations, ClusterViolation::InitialTripleUnlisted);
                }
            }
            ClusterPosition::Initial if chars.len() > 3 => {
                push_once(&mut violations, ClusterViolation::UnsupportedLength);
            }
            ClusterPosition::Medial if chars.len() == 3 => {
                let last_pair = &input[1..];
                if !INITIAL_PAIRS.contains(&last_pair) {
                    push_once(&mut violations, ClusterViolation::MedialLastPairUnlisted);
                }
                if FORBIDDEN_MEDIAL_TRIPLES.contains(&input) {
                    push_once(&mut violations, ClusterViolation::ForbiddenMedialTriple);
                }
            }
            ClusterPosition::Medial if chars.len() == 4 => {}
            ClusterPosition::Medial if chars.len() > 4 => {
                push_once(&mut violations, ClusterViolation::UnsupportedLength);
            }
            ClusterPosition::Final => {
                push_once(&mut violations, ClusterViolation::NonNameFinalConsonant);
                if chars.len() > 3 {
                    push_once(&mut violations, ClusterViolation::UnsupportedLength);
                }
            }
            _ => {}
        }
    }

    let source_length_tension = !is_name && position == ClusterPosition::Medial && chars.len() == 4;
    let last_three_initial_triple = if source_length_tension {
        let last = &input[input.len() - 3..];
        INITIAL_TRIPLES
            .iter()
            .copied()
            .find(|triple| *triple == last)
    } else {
        None
    };

    let source_length_tension = source_length_tension.then_some(ClusterSourceTension {
        last_three_initial_triple,
    });

    Ok(ClusterInspection {
        cluster: input.to_owned(),
        position,
        kind,
        violations,
        source_length_tension,
    })
}

fn pair_violations(left: char, right: char) -> Vec<ClusterViolation> {
    let mut reasons = Vec::new();
    if left == right {
        reasons.push(ClusterViolation::DuplicateNeighbors);
    }
    if !SONORANTS.contains(left)
        && !SONORANTS.contains(right)
        && ((UNVOICED.contains(left) && VOICED.contains(right))
            || (VOICED.contains(left) && UNVOICED.contains(right)))
    {
        reasons.push(ClusterViolation::VoicingMismatch);
    }
    if SIBILANTS.contains(left) && SIBILANTS.contains(right) {
        reasons.push(ClusterViolation::MultipleSibilants);
    }
    if matches!(
        (left, right),
        ('c', 'x') | ('k', 'x') | ('x', 'c') | ('x', 'k')
    ) {
        reasons.push(ClusterViolation::ForbiddenXPair);
    }
    reasons
}

fn push_once(items: &mut Vec<ClusterViolation>, reason: ClusterViolation) {
    if !items.contains(&reason) {
        items.push(reason);
    }
}
