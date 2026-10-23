//! Caller-annotated buffer events, separate from written phonological analysis.

use std::{error::Error, fmt};

use crate::{AnalysisDiagnostic, NucleusKind, StressLevel, WordAnalysis, WordKind};

const REALIZATION_POLICY: &str = "buffer-plan-v1";
const SEC38: &[&str] = &["3.8", "3.9"];

/// One caller-selected, unwritten buffer event at an original consonant position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferEvent {
    after_position: usize,
    performed_nucleus_index: usize,
    level: StressLevel,
    source_sections: &'static [&'static str],
}

impl BufferEvent {
    /// Returns the original byte offset of the consonant after which buffering occurs.
    #[must_use]
    pub const fn after_position(&self) -> usize {
        self.after_position
    }
    /// Returns the zero-based index of the performed buffer nucleus.
    #[must_use]
    pub const fn performed_nucleus_index(&self) -> usize {
        self.performed_nucleus_index
    }
    /// Returns the source-assigned stress category for a buffer.
    #[must_use]
    pub const fn level(&self) -> StressLevel {
        self.level
    }
    /// Returns the source sections supporting this event kind.
    #[must_use]
    pub const fn source_sections(&self) -> &'static [&'static str] {
        self.source_sections
    }
}

/// A typed invalid buffer annotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RealizationError {
    /// The original byte does not identify a morphological consonant.
    InvalidPosition {
        /// Requested original-input byte offset.
        position: usize,
    },
    /// A caller supplied the same original position twice.
    DuplicatePosition {
        /// Repeated original-input byte offset.
        position: usize,
    },
    /// Buffering conflicts with an explicitly selected syllabic nucleus.
    SyllabicNucleusConflict {
        /// Original-input byte offset of the selected syllabic consonant.
        position: usize,
    },
    /// The consonant is not followed by another consonant or final in a name.
    UnsupportedLocation {
        /// Original-input byte offset that has no supported buffer context.
        position: usize,
    },
}

impl fmt::Display for RealizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPosition { position } => {
                write!(f, "byte {position} is not a morphological consonant")
            }
            Self::DuplicatePosition { position } => {
                write!(f, "buffer position {position} was supplied more than once")
            }
            Self::SyllabicNucleusConflict { position } => write!(
                f,
                "buffer after byte {position} conflicts with a selected syllabic nucleus"
            ),
            Self::UnsupportedLocation { position } => write!(
                f,
                "byte {position} is not followed by a consonant or a name-final consonant"
            ),
        }
    }
}
impl Error for RealizationError {}

/// A selected plan that distinguishes written, performed, and countable nuclei.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealizationPlan {
    written_nucleus_count: usize,
    performed_nucleus_count: usize,
    countable_nucleus_count: usize,
    events: Vec<BufferEvent>,
    diagnostics: Vec<AnalysisDiagnostic>,
    source_id: &'static str,
    policy_id: &'static str,
}

impl RealizationPlan {
    /// Returns the number of nuclei in the written analysis.
    #[must_use]
    pub const fn written_nucleus_count(&self) -> usize {
        self.written_nucleus_count
    }
    /// Returns written nuclei plus caller-selected unwritten buffer events.
    #[must_use]
    pub const fn performed_nucleus_count(&self) -> usize {
        self.performed_nucleus_count
    }
    /// Returns written nuclei eligible for stress counting.
    #[must_use]
    pub const fn countable_nucleus_count(&self) -> usize {
        self.countable_nucleus_count
    }
    /// Returns validated buffer events in original-position order.
    #[must_use]
    pub fn events(&self) -> &[BufferEvent] {
        &self.events
    }
    /// Returns unchanged word-analysis diagnostics, including cluster findings.
    #[must_use]
    pub fn diagnostics(&self) -> &[AnalysisDiagnostic] {
        &self.diagnostics
    }
    /// Returns the acquired grammar snapshot identity.
    #[must_use]
    pub const fn source_id(&self) -> &'static str {
        self.source_id
    }
    /// Returns the explicit buffer planning policy identity.
    #[must_use]
    pub const fn policy_id(&self) -> &'static str {
        self.policy_id
    }
}

/// Validates caller annotations for optional buffer events without synthesizing audio.
///
/// # Errors
///
/// Returns a named error for invalid, duplicate, syllabic, or unsupported positions.
pub fn plan_realization(
    analysis: &WordAnalysis,
    buffer_after: &[usize],
) -> Result<RealizationPlan, RealizationError> {
    let original = analysis.original();
    let body_start = usize::from(original.starts_with('.'));
    let body_end = original
        .len()
        .saturating_sub(usize::from(original.ends_with('.')));
    let mut positions = buffer_after.to_vec();
    positions.sort_unstable();
    for pair in positions.windows(2) {
        if pair[0] == pair[1] {
            return Err(RealizationError::DuplicatePosition { position: pair[0] });
        }
    }
    let mut events = Vec::with_capacity(positions.len());
    for position in positions {
        if position < body_start || position >= body_end {
            return Err(RealizationError::InvalidPosition { position });
        }
        let byte = original.as_bytes()[position].to_ascii_lowercase();
        if !"bcdfgjklmnprstvxz".contains(byte as char) {
            return Err(RealizationError::InvalidPosition { position });
        }
        if analysis.nuclei().iter().any(|nucleus| {
            nucleus.start() == position
                && matches!(nucleus.kind(), NucleusKind::SyllabicConsonant(_))
        }) {
            return Err(RealizationError::SyllabicNucleusConflict { position });
        }
        let mut next = position + 1;
        while next < body_end && original.as_bytes()[next] == b',' {
            next += 1;
        }
        let followed_by_consonant = next < body_end
            && "bcdfgjklmnprstvxz".contains(original.as_bytes()[next].to_ascii_lowercase() as char);
        let final_name_consonant = analysis.word_kind() == WordKind::Cmevla
            && next == body_end
            && "bcdfgjklmnprstvxz".contains(byte as char);
        if !followed_by_consonant && !final_name_consonant {
            return Err(RealizationError::UnsupportedLocation { position });
        }
        events.push(BufferEvent {
            after_position: position,
            performed_nucleus_index: analysis
                .nuclei()
                .iter()
                .filter(|nucleus| nucleus.end() <= position)
                .count()
                + events.len(),
            level: StressLevel::Weak,
            source_sections: SEC38,
        });
    }
    let written_nucleus_count = analysis.nuclei().len();
    let countable_nucleus_count = analysis
        .nuclei()
        .iter()
        .filter(|nucleus| nucleus.stress_countable())
        .count();
    Ok(RealizationPlan {
        written_nucleus_count,
        performed_nucleus_count: written_nucleus_count + events.len(),
        countable_nucleus_count,
        events,
        diagnostics: analysis.diagnostics().to_vec(),
        source_id: analysis.source_id(),
        policy_id: REALIZATION_POLICY,
    })
}
