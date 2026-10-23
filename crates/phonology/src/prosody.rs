//! Bounded syllable and stress plans over an already inspected word.

use std::{error::Error, fmt, ops::Range};

use crate::{
    AnalysisDiagnostic, DiagnosticKind, Nucleus, NucleusKind, WordAnalysis, WordKind,
    initial_pairs, initial_triples,
};

const SYLLABLES_POLICY: &str = "syllables-v1";
const STRESS_POLICY: &str = "stress-v1";
const SEC34_39: &[&str] = &["3.4", "3.9"];

/// Selects the deterministic conventional grouping or explicit written hints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SyllabificationMode {
    /// Apply the project's bounded implementation of the source convention.
    Canonical,
    /// Honor a comma hint between nuclei as the caller-selected boundary.
    WrittenHints,
}

impl SyllabificationMode {
    /// Returns the stable output spelling for this mode.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Canonical => "canonical",
            Self::WrittenHints => "hints",
        }
    }
}

/// A named reason conventional grouping is unresolved or limited.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProsodySourceLimitation {
    /// A selected nucleus is separated by an unsupported ordinary cluster.
    UnsupportedCluster {
        /// Original-input span containing the unsupported cluster.
        span: Range<usize>,
        /// Canonical consonant-run spelling.
        spelling: String,
    },
    /// A four-consonant gap is covered by the §3.6/§3.7 source tension.
    SourceLengthTension {
        /// Original-input span within the source-length conflict.
        span: Range<usize>,
        /// Canonical four-consonant spelling.
        spelling: String,
        /// Sections carried by the public cluster source-tension declaration.
        source_sections: Vec<&'static str>,
    },
    /// The source gives no definitive canonical grouping for a long name cluster.
    NameClusterUnderspecified {
        /// Original-input span of the long name cluster.
        span: Range<usize>,
        /// Canonical consonant-run spelling.
        spelling: String,
    },
    /// A selected syllabic consonant lacks the preceding consonant required
    /// by the source's two-consonant syllabic-consonant description.
    SelectedSyllabicWithoutPrecedingConsonant {
        /// Original-input span of the selected syllabic consonant.
        span: Range<usize>,
    },
    /// The selected syllabic consonant cannot be placed in a supported
    /// two-consonant syllable without losing or inventing written material.
    SelectedSyllabicUnsupportedShape {
        /// Original-input span of the unsupported syllable shape.
        span: Range<usize>,
        /// Lowercase spelling with comma hints removed.
        spelling: String,
    },
    /// A source cluster rule finding prevents assigning a conventional split.
    SourceRuleFinding {
        /// Original-input span associated with the source finding.
        span: Range<usize>,
        /// Canonical consonant-run spelling.
        spelling: String,
        /// Source locators from the actual cluster-rule findings.
        source_sections: Vec<&'static str>,
    },
    /// A written comma falls inside a selected vowel/diphthong nucleus.
    HintInsideNucleus {
        /// Nucleus span containing the written hint.
        span: Range<usize>,
    },
    /// Multiple comma hints in one nucleus gap would create empty syllables.
    MultipleHintsBetweenNuclei {
        /// Original span covering the adjacent hints.
        span: Range<usize>,
    },
    /// A hint leaves an invalid coda or unlisted onset in the selected boundary.
    HintViolatesClusterBoundary {
        /// Original span around the requested boundary.
        span: Range<usize>,
        /// Canonical onset left after the explicit boundary.
        onset: String,
    },
    /// A written hint after an apostrophe would detach it from its vowel.
    HintConflictsWithApostrophe {
        /// Original-input span of the apostrophe and incompatible hint.
        span: Range<usize>,
    },
}

impl ProsodySourceLimitation {
    /// Returns the source sections relevant to this limitation.
    #[must_use]
    pub fn source_sections(&self) -> &[&'static str] {
        match self {
            Self::UnsupportedCluster { .. } => SEC34_39,
            Self::SourceRuleFinding {
                source_sections, ..
            } => source_sections,
            Self::SourceLengthTension {
                source_sections, ..
            } => source_sections,
            Self::NameClusterUnderspecified { .. } => &["3.6", "3.7", "3.9"],
            Self::SelectedSyllabicWithoutPrecedingConsonant { .. }
            | Self::SelectedSyllabicUnsupportedShape { .. } => &["3.9"],
            Self::HintInsideNucleus { .. }
            | Self::MultipleHintsBetweenNuclei { .. }
            | Self::HintViolatesClusterBoundary { .. }
            | Self::HintConflictsWithApostrophe { .. } => &["3.3", "3.9"],
        }
    }

    /// Returns the exact original-input byte span when one is known.
    #[must_use]
    pub fn span(&self) -> Range<usize> {
        match self {
            Self::UnsupportedCluster { span, .. }
            | Self::SourceLengthTension { span, .. }
            | Self::NameClusterUnderspecified { span, .. }
            | Self::SelectedSyllabicWithoutPrecedingConsonant { span }
            | Self::SelectedSyllabicUnsupportedShape { span, .. }
            | Self::SourceRuleFinding { span, .. }
            | Self::HintInsideNucleus { span }
            | Self::MultipleHintsBetweenNuclei { span }
            | Self::HintViolatesClusterBoundary { span, .. }
            | Self::HintConflictsWithApostrophe { span } => span.clone(),
        }
    }

    /// Returns a stable machine-readable limitation code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedCluster { .. } => "syllables.unsupported-cluster",
            Self::SourceLengthTension { .. } => "syllables.source-length-tension",
            Self::NameClusterUnderspecified { .. } => "syllables.name-cluster-underspecified",
            Self::SelectedSyllabicWithoutPrecedingConsonant { .. } => {
                "syllables.syllabic-consonant-without-preceding-consonant"
            }
            Self::SelectedSyllabicUnsupportedShape { .. } => {
                "syllables.syllabic-consonant-unsupported-shape"
            }
            Self::SourceRuleFinding { .. } => "syllables.source-rule-finding",
            Self::HintInsideNucleus { .. } => "syllables.hint-inside-nucleus",
            Self::MultipleHintsBetweenNuclei { .. } => "syllables.multiple-hints-between-nuclei",
            Self::HintViolatesClusterBoundary { .. } => "syllables.hint-violates-cluster-boundary",
            Self::HintConflictsWithApostrophe { .. } => "syllables.hint-conflicts-with-apostrophe",
        }
    }
}

/// A source limitation attached to a syllabification result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProsodyLimitation {
    code: &'static str,
    source_sections: Vec<&'static str>,
    span: Range<usize>,
    detail: String,
}

impl ProsodyLimitation {
    fn from_source(value: ProsodySourceLimitation) -> Self {
        let detail = match &value {
            ProsodySourceLimitation::UnsupportedCluster { spelling, .. } => {
                format!("unsupported cluster {spelling}")
            }
            ProsodySourceLimitation::SourceLengthTension { spelling, .. } => {
                format!("source-length tension in {spelling}")
            }
            ProsodySourceLimitation::NameClusterUnderspecified { spelling, .. } => {
                format!("canonical name grouping is unspecified for {spelling}")
            }
            ProsodySourceLimitation::SelectedSyllabicWithoutPrecedingConsonant { .. } => {
                "source does not determine a syllable layout for this selected syllabic consonant without a preceding consonant".to_owned()
            }
            ProsodySourceLimitation::SelectedSyllabicUnsupportedShape { spelling, .. } => {
                format!("source does not determine a supported two-consonant syllable for {spelling}")
            }
            ProsodySourceLimitation::SourceRuleFinding { spelling, .. } => {
                format!("source cluster finding prevents grouping {spelling}")
            }
            ProsodySourceLimitation::HintInsideNucleus { .. } => {
                "comma hint lies inside a nucleus".to_owned()
            }
            ProsodySourceLimitation::MultipleHintsBetweenNuclei { .. } => {
                "multiple comma hints occur between adjacent nuclei".to_owned()
            }
            ProsodySourceLimitation::HintViolatesClusterBoundary { onset, .. } => {
                format!("written hint leaves an unsupported onset {onset}")
            }
            ProsodySourceLimitation::HintConflictsWithApostrophe { .. } => {
                "written hint after apostrophe conflicts with apostrophe-to-vowel boundary ownership".to_owned()
            }
        };
        let source_sections = value.source_sections().to_vec();
        Self {
            code: value.code(),
            source_sections,
            span: value.span(),
            detail,
        }
    }

    /// Returns a stable machine-readable code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }
    /// Returns grammar sections supporting the limitation.
    #[must_use]
    pub fn source_sections(&self) -> &[&'static str] {
        &self.source_sections
    }
    /// Returns the original-input byte span.
    #[must_use]
    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }
    /// Returns a concise explanation of the limitation.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// One conventionally or explicitly grouped syllable with original offsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Syllable {
    start: usize,
    end: usize,
    original_spelling: String,
    canonical_spelling: String,
    nucleus_index: usize,
}

impl Syllable {
    /// Returns the inclusive original-input start byte.
    #[must_use]
    pub const fn start(&self) -> usize {
        self.start
    }
    /// Returns the exclusive original-input end byte.
    #[must_use]
    pub const fn end(&self) -> usize {
        self.end
    }
    /// Returns the original substring, retaining case and comma hints.
    #[must_use]
    pub fn original_spelling(&self) -> &str {
        &self.original_spelling
    }
    /// Returns lowercase spelling with written comma hints removed.
    #[must_use]
    pub fn canonical_spelling(&self) -> &str {
        &self.canonical_spelling
    }
    /// Returns the zero-based index of the selected nucleus in `WordAnalysis`.
    #[must_use]
    pub const fn nucleus_index(&self) -> usize {
        self.nucleus_index
    }
}

/// A complete grouping or a limitation-only unresolved result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Syllabification {
    syllables: Vec<Syllable>,
    limitations: Vec<ProsodyLimitation>,
    resolved: bool,
    source_id: &'static str,
    policy_id: &'static str,
    mode: SyllabificationMode,
}

impl Syllabification {
    /// Returns the resolved syllables; unresolved results contain no partial list.
    #[must_use]
    pub fn syllables(&self) -> &[Syllable] {
        &self.syllables
    }
    /// Returns all named source or policy limitations.
    #[must_use]
    pub fn limitations(&self) -> &[ProsodyLimitation] {
        &self.limitations
    }
    /// Reports whether every selected nucleus has one resolved syllable.
    #[must_use]
    pub const fn is_resolved(&self) -> bool {
        self.resolved
    }
    /// Returns the acquired grammar snapshot identity.
    #[must_use]
    pub const fn source_id(&self) -> &'static str {
        self.source_id
    }
    /// Returns the deterministic syllabification policy identity.
    #[must_use]
    pub const fn policy_id(&self) -> &'static str {
        self.policy_id
    }
    /// Returns the selected grouping mode.
    #[must_use]
    pub const fn mode(&self) -> SyllabificationMode {
        self.mode
    }
}

/// Groups selected word nuclei using a bounded project convention.
#[must_use]
pub fn syllabify(analysis: &WordAnalysis, mode: SyllabificationMode) -> Syllabification {
    let original = analysis.original();
    let body_start = usize::from(original.starts_with('.'));
    let body_end = original
        .len()
        .saturating_sub(usize::from(original.ends_with('.')));
    let nuclei = analysis.nuclei();
    let mut limitations = Vec::new();
    for nucleus in nuclei {
        if mode == SyllabificationMode::WrittenHints
            && analysis
                .comma_positions()
                .iter()
                .any(|comma| *comma > nucleus.start() && *comma < nucleus.end())
        {
            limitations.push(ProsodyLimitation::from_source(
                ProsodySourceLimitation::HintInsideNucleus {
                    span: nucleus.start()..nucleus.end(),
                },
            ));
        }
    }
    limitations.extend(source_cluster_limitations(analysis));
    if !limitations.is_empty() {
        return unresolved(analysis, mode, limitations);
    }

    let mut boundaries = Vec::with_capacity(nuclei.len().saturating_sub(1));
    // §3.9 describes a syllabic consonant as the final consonant of a
    // two-consonant syllable. If its syllable has no preceding non-nucleus
    // consonant available, preserve the selected nucleus but do not invent a
    // standalone-consonant syllable (the sairgoi source tension).
    for (index, nucleus) in nuclei.iter().enumerate() {
        if !matches!(nucleus.kind(), NucleusKind::SyllabicConsonant(_)) {
            continue;
        }
        let preceding_start = if index == 0 {
            body_start
        } else {
            nuclei[index - 1].end()
        };
        let gap = letters_in(original, preceding_start..nucleus.start(), b',');
        if !gap.chars().any(|ch| "bcdfgjklmnprstvxz".contains(ch)) {
            limitations.push(ProsodyLimitation::from_source(
                ProsodySourceLimitation::SelectedSyllabicWithoutPrecedingConsonant {
                    span: nucleus.start()..nucleus.end(),
                },
            ));
            break;
        }
    }
    if !limitations.is_empty() {
        return unresolved(analysis, mode, limitations);
    }
    for pair in nuclei.windows(2) {
        let previous = &pair[0];
        let next = &pair[1];
        let gap_start = previous.end();
        let gap_end = next.start();
        let commas: Vec<usize> = analysis
            .comma_positions()
            .iter()
            .copied()
            .filter(|position| *position >= gap_start && *position < gap_end)
            .collect();
        if mode == SyllabificationMode::WrittenHints && !commas.is_empty() {
            if commas.len() != 1 {
                limitations.push(ProsodyLimitation::from_source(
                    ProsodySourceLimitation::MultipleHintsBetweenNuclei {
                        span: commas[0]..commas[commas.len() - 1] + 1,
                    },
                ));
                break;
            }
            if original.as_bytes().get(commas[0].wrapping_sub(1)) == Some(&b'\'') {
                limitations.push(ProsodyLimitation::from_source(
                    ProsodySourceLimitation::HintConflictsWithApostrophe {
                        span: commas[0] - 1..commas[0] + 1,
                    },
                ));
                break;
            }
            let boundary = commas[0] + 1;
            let before = letters_in(original, gap_start..commas[0], b',');
            let after = letters_in(original, boundary..gap_end, b',');
            if before.chars().count() > 1 || !valid_onset(&after) {
                limitations.push(ProsodyLimitation::from_source(
                    ProsodySourceLimitation::HintViolatesClusterBoundary {
                        span: gap_start..gap_end,
                        onset: after,
                    },
                ));
                break;
            }
            boundaries.push(boundary);
            continue;
        }
        if matches!(next.kind(), NucleusKind::SyllabicConsonant(_)) {
            // A selected syllabic consonant is specifically the second
            // consonant in its syllable. Ordinary onset-pair preference must
            // not pull two consonants into that syllable (for example,
            // `acmlana` must not become `a/cml/a/na`). Earlier consonants
            // remain with the preceding nucleus when one exists.
            if let Some(boundary) = last_consonant_offset(original, gap_start..gap_end) {
                boundaries.push(boundary);
            } else {
                limitations.push(ProsodyLimitation::from_source(
                    ProsodySourceLimitation::SelectedSyllabicWithoutPrecedingConsonant {
                        span: next.start()..next.end(),
                    },
                ));
                break;
            }
            continue;
        }
        match conventional_boundary(analysis, previous, next, gap_start, gap_end) {
            Ok(boundary) => boundaries.push(boundary),
            Err(limitation) => {
                limitations.push(ProsodyLimitation::from_source(limitation));
                break;
            }
        }
    }
    if !limitations.is_empty() {
        return unresolved(analysis, mode, limitations);
    }
    let mut syllables = Vec::with_capacity(nuclei.len());
    for (index, nucleus) in nuclei.iter().enumerate() {
        let start = if index == 0 {
            body_start
        } else {
            boundaries[index - 1]
        };
        let end = boundaries.get(index).copied().unwrap_or(body_end);
        if start > nucleus.start() || end < nucleus.end() || start >= end {
            limitations.push(ProsodyLimitation::from_source(
                ProsodySourceLimitation::UnsupportedCluster {
                    span: start.min(end)..start.max(end),
                    spelling: original[start.min(end)..start.max(end)].to_owned(),
                },
            ));
            return unresolved(analysis, mode, limitations);
        }
        let original_spelling = original[start..end].to_owned();
        let canonical_spelling: String = original_spelling
            .chars()
            .filter(|ch| *ch != ',')
            .flat_map(char::to_lowercase)
            .collect();
        if matches!(nucleus.kind(), NucleusKind::SyllabicConsonant(_))
            && !is_two_consonant_syllabic_shape(&canonical_spelling, nucleus.spelling())
        {
            limitations.push(ProsodyLimitation::from_source(
                ProsodySourceLimitation::SelectedSyllabicUnsupportedShape {
                    span: start..end,
                    spelling: canonical_spelling,
                },
            ));
            return unresolved(analysis, mode, limitations);
        }
        syllables.push(Syllable {
            start,
            end,
            original_spelling,
            canonical_spelling,
            nucleus_index: index,
        });
        debug_assert!(matches!(
            nucleus.kind(),
            NucleusKind::Vowel(_) | NucleusKind::Diphthong(_) | NucleusKind::SyllabicConsonant(_)
        ));
    }
    Syllabification {
        syllables,
        limitations,
        resolved: true,
        source_id: analysis.source_id(),
        policy_id: SYLLABLES_POLICY,
        mode,
    }
}

fn unresolved(
    analysis: &WordAnalysis,
    mode: SyllabificationMode,
    limitations: Vec<ProsodyLimitation>,
) -> Syllabification {
    Syllabification {
        syllables: Vec::new(),
        limitations,
        resolved: false,
        source_id: analysis.source_id(),
        policy_id: SYLLABLES_POLICY,
        mode,
    }
}

fn source_cluster_limitations(analysis: &WordAnalysis) -> Vec<ProsodyLimitation> {
    let mut limitations = Vec::new();
    for cluster in analysis.clusters() {
        let inspection = cluster.inspection();
        let spelling = inspection.cluster().to_owned();
        let span = cluster.start()..cluster.end();
        let is_long_name = analysis.word_kind() == WordKind::Cmevla && spelling.len() > 3;
        let has_length_tension = inspection.source_length_tension().is_some();
        if is_long_name {
            limitations.push(ProsodyLimitation::from_source(
                ProsodySourceLimitation::NameClusterUnderspecified {
                    span: span.clone(),
                    spelling: spelling.clone(),
                },
            ));
        } else if has_length_tension {
            limitations.push(ProsodyLimitation::from_source(
                ProsodySourceLimitation::SourceLengthTension {
                    span: span.clone(),
                    spelling: spelling.clone(),
                    source_sections: inspection
                        .source_length_tension()
                        .map_or_else(Vec::new, |tension| tension.source_sections().to_vec()),
                },
            ));
        }
        let violations = inspection.violations();
        if !violations.is_empty() {
            let mut source_sections = Vec::new();
            for violation in violations {
                for section in violation.source_sections() {
                    if !source_sections.contains(section) {
                        source_sections.push(*section);
                    }
                }
            }
            limitations.push(ProsodyLimitation::from_source(
                ProsodySourceLimitation::SourceRuleFinding {
                    span,
                    spelling,
                    source_sections,
                },
            ));
        }
    }
    limitations
}

fn conventional_boundary(
    analysis: &WordAnalysis,
    previous: &Nucleus,
    next: &Nucleus,
    gap_start: usize,
    gap_end: usize,
) -> Result<usize, ProsodySourceLimitation> {
    let original = analysis.original();
    let span = gap_start..gap_end;
    let gap = letters_in(original, span.clone(), b',');
    if gap.contains('\'') {
        let position = original[gap_start..gap_end]
            .find('\'')
            .map_or(next.start(), |offset| gap_start + offset);
        return Ok(position);
    }
    let consonants: String = gap
        .chars()
        .filter(|ch| "bcdfgjklmnprstvxz".contains(*ch))
        .collect();
    if consonants.is_empty() {
        return Ok(next.start());
    }
    if let Some(cluster) = analysis
        .clusters()
        .iter()
        .find(|cluster| cluster.start() < gap_end && cluster.end() > gap_start)
        && !cluster.inspection().violations().is_empty()
    {
        return Err(ProsodySourceLimitation::SourceRuleFinding {
            span: cluster.start()..cluster.end(),
            spelling: cluster.inspection().cluster().to_owned(),
            source_sections: cluster
                .inspection()
                .violations()
                .iter()
                .flat_map(|finding| finding.source_sections().iter().copied())
                .collect(),
        });
    }
    if analysis.word_kind() == WordKind::Cmevla && consonants.len() > 3 {
        return Err(ProsodySourceLimitation::NameClusterUnderspecified {
            span,
            spelling: consonants,
        });
    }
    if consonants.len() == 4
        && let Some(tension) = analysis
            .clusters()
            .iter()
            .find(|cluster| cluster.start() <= gap_start && cluster.end() >= gap_end)
            .and_then(|cluster| cluster.inspection().source_length_tension())
    {
        return Err(ProsodySourceLimitation::SourceLengthTension {
            span,
            spelling: consonants,
            source_sections: tension.source_sections().to_vec(),
        });
    }
    if consonants.len() > 3 {
        let kind = if analysis.word_kind() == WordKind::Cmevla {
            ProsodySourceLimitation::NameClusterUnderspecified {
                span,
                spelling: consonants,
            }
        } else {
            ProsodySourceLimitation::UnsupportedCluster {
                span,
                spelling: consonants,
            }
        };
        return Err(kind);
    }
    if analysis.word_kind() == WordKind::Cmevla && consonants.len() == 3 {
        // The grammar allows pronunciation variation in names; this is only the
        // project's conventional candidate and must still satisfy source lists.
    }
    let boundary = match consonants.len() {
        1 => gap_start,
        2 if initial_pairs().contains(&consonants.as_str()) => {
            first_consonant_offset(original, &span, 2)
        }
        2 => second_consonant_offset(original, &span),
        3 if initial_triples().contains(&consonants.as_str()) => {
            first_consonant_offset(original, &span, 3)
        }
        3 if initial_pairs().contains(&&consonants[1..]) => {
            second_consonant_offset(original, &span)
        }
        _ => {
            return Err(ProsodySourceLimitation::UnsupportedCluster {
                span,
                spelling: consonants,
            });
        }
    };
    if previous.start() > boundary || next.end() < boundary {
        return Err(ProsodySourceLimitation::UnsupportedCluster {
            span,
            spelling: gap,
        });
    }
    Ok(boundary)
}

fn first_consonant_offset(original: &str, span: &Range<usize>, _count: usize) -> usize {
    (span.start..span.end)
        .find(|offset| {
            "bcdfgjklmnprstvxz".contains(original.as_bytes()[*offset].to_ascii_lowercase() as char)
        })
        .unwrap_or(span.start)
}

fn second_consonant_offset(original: &str, span: &Range<usize>) -> usize {
    let mut seen = 0;
    let end = span.end;
    for offset in span.clone() {
        if "bcdfgjklmnprstvxz".contains(original.as_bytes()[offset].to_ascii_lowercase() as char) {
            seen += 1;
            if seen == 2 {
                return offset;
            }
        }
    }
    end
}

fn last_consonant_offset(original: &str, span: Range<usize>) -> Option<usize> {
    span.clone().rev().find(|offset| {
        "bcdfgjklmnprstvxz".contains(original.as_bytes()[*offset].to_ascii_lowercase() as char)
    })
}

fn is_two_consonant_syllabic_shape(spelling: &str, nucleus: &str) -> bool {
    let consonants: Vec<char> = spelling
        .chars()
        .filter(|ch| "bcdfgjklmnprstvxz".contains(*ch))
        .collect();
    consonants.len() == 2 && spelling.ends_with(&nucleus.to_ascii_lowercase())
}

fn letters_in(original: &str, span: Range<usize>, ignore: u8) -> String {
    original.as_bytes()[span]
        .iter()
        .copied()
        .filter(|byte| *byte != ignore)
        .map(char::from)
        .flat_map(char::to_lowercase)
        .collect()
}

fn valid_onset(onset: &str) -> bool {
    match onset.len() {
        0 | 1 => true,
        2 => initial_pairs().contains(&onset),
        3 => initial_triples().contains(&onset),
        _ => false,
    }
}

/// Selects how a nucleus is assigned a primary/weak/optional-weak level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StressChoice {
    /// Apply the caller-declared word-kind default.
    Default,
    /// Request no primary stress.
    Unstressed,
    /// Request the zero-based index in `WordAnalysis::nuclei()`.
    Nucleus(usize),
}

/// Declares whether a following word is an unpaused brivla for the final-stress guard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FollowingContext {
    /// The following-word context was not supplied.
    Unspecified,
    /// The following word is a brivla and no pause separates it.
    BrivlaUnpaused,
    /// A pause separates the word from a following brivla.
    BrivlaAfterPause,
}

/// One selected written nucleus's planned stress category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StressLevel {
    /// The one planned primary-stressed nucleus.
    Primary,
    /// Required weak stress, with no emphasis assigned.
    Weak,
    /// Eligible for speaker-selected secondary stress; not a measured level.
    OptionalSecondary,
}

/// A named stress-planning error.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StressError {
    /// Content-word stress requires two countable nuclei.
    InsufficientCountableNuclei,
    /// Requested nucleus is absent or excluded from stress.
    IneligibleNucleus {
        /// Requested zero-based nucleus index.
        index: usize,
    },
    /// Content-word primary stress must stay penultimate.
    RequiredPenultimate {
        /// Required zero-based penultimate countable nucleus.
        expected: usize,
        /// Requested zero-based nucleus.
        found: usize,
    },
    /// Content word cannot be explicitly marked unstressed.
    ContentWordUnstressed,
    /// A final cmavo primary before an unpaused brivla requires a pause.
    PauseRequiredBeforeBrivla,
    /// Capitalized name nuclei do not identify one vowel stress target.
    AmbiguousNameStress,
    /// A name's capital mark is on a consonant with no capitalized vowel.
    NameCapitalWithoutVowel,
    /// A capitalized name nucleus is not countable.
    NameStressOnWeakNucleus {
        /// Capitalized zero-based nucleus index.
        index: usize,
    },
    /// Requested nonstandard name stress lacks the source-required vowel mark.
    NameNotationRequired,
    /// This policy does not assign an unstressed name request.
    UnstressedNameUnsupported,
}

impl fmt::Display for StressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientCountableNuclei => {
                f.write_str("content-word stress needs two countable nuclei")
            }
            Self::IneligibleNucleus { index } => {
                write!(f, "nucleus {index} is not eligible for stress")
            }
            Self::RequiredPenultimate { expected, found } => write!(
                f,
                "content-word stress must select nucleus {expected}, not {found}"
            ),
            Self::ContentWordUnstressed => {
                f.write_str("content words require penultimate primary stress")
            }
            Self::PauseRequiredBeforeBrivla => {
                f.write_str("final cmavo stress requires a pause before a brivla")
            }
            Self::AmbiguousNameStress => {
                f.write_str("name has more than one capitalized vowel nucleus")
            }
            Self::NameCapitalWithoutVowel => {
                f.write_str("name capitals do not mark a vowel nucleus")
            }
            Self::NameStressOnWeakNucleus { index } => {
                write!(f, "name stress mark selects weak nucleus {index}")
            }
            Self::NameNotationRequired => {
                f.write_str("nonstandard name stress requires a capitalized vowel")
            }
            Self::UnstressedNameUnsupported => {
                f.write_str("unstressed name planning is unsupported by this policy")
            }
        }
    }
}
impl Error for StressError {}

/// A stress plan with explicit primary selection and context limitations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StressAnalysis {
    primary_nucleus: Option<usize>,
    levels: Vec<StressLevel>,
    limitations: Vec<&'static str>,
    source_id: &'static str,
    policy_id: &'static str,
}

impl StressAnalysis {
    /// Returns the selected primary nucleus, if the policy assigns one.
    #[must_use]
    pub const fn primary_nucleus(&self) -> Option<usize> {
        self.primary_nucleus
    }
    /// Returns one stress category for each selected written nucleus.
    #[must_use]
    pub fn levels(&self) -> &[StressLevel] {
        &self.levels
    }
    /// Returns applicable unchecked or policy-limitation identifiers.
    #[must_use]
    pub fn limitations(&self) -> &[&'static str] {
        &self.limitations
    }
    /// Returns the acquired grammar snapshot identity.
    #[must_use]
    pub const fn source_id(&self) -> &'static str {
        self.source_id
    }
    /// Returns the selected stress policy identity.
    #[must_use]
    pub const fn policy_id(&self) -> &'static str {
        self.policy_id
    }
}

/// Plans stress over the nuclei already selected by `analyze_word`.
///
/// # Errors
///
/// Returns a named error for unsupported requests, missing content-word nuclei,
/// incompatible name capitalization, or the final-cmavo pause guard.
pub fn assign_stress(
    analysis: &WordAnalysis,
    choice: StressChoice,
    following: FollowingContext,
) -> Result<StressAnalysis, StressError> {
    let nuclei = analysis.nuclei();
    let eligible: Vec<usize> = nuclei
        .iter()
        .enumerate()
        .filter_map(|(index, nucleus)| nucleus.stress_countable().then_some(index))
        .collect();
    let name_marks: Vec<usize> = nuclei
        .iter()
        .enumerate()
        .filter_map(|(index, nucleus)| {
            analysis
                .capital_positions()
                .iter()
                .any(|position| *position >= nucleus.start() && *position < nucleus.end())
                .then_some(index)
        })
        .collect();
    let is_content = matches!(
        analysis.word_kind(),
        WordKind::Brivla | WordKind::Zihevla | WordKind::GismuBasedLujvo
    );
    let mut limitations = Vec::new();
    let primary = if is_content {
        if eligible.len() < 2 {
            return Err(StressError::InsufficientCountableNuclei);
        }
        let expected = eligible[eligible.len() - 2];
        match choice {
            StressChoice::Default => Some(expected),
            StressChoice::Unstressed => return Err(StressError::ContentWordUnstressed),
            StressChoice::Nucleus(index) if !eligible.contains(&index) => {
                return Err(StressError::IneligibleNucleus { index });
            }
            StressChoice::Nucleus(index) if index != expected => {
                return Err(StressError::RequiredPenultimate {
                    expected,
                    found: index,
                });
            }
            StressChoice::Nucleus(index) => Some(index),
        }
    } else if analysis.word_kind() == WordKind::Cmevla {
        if name_marks.len() > 1 {
            return Err(StressError::AmbiguousNameStress);
        }
        let marked = name_marks.first().copied();
        if !analysis.capital_positions().is_empty() && marked.is_none() {
            return Err(StressError::NameCapitalWithoutVowel);
        }
        if marked.is_some_and(|index| !eligible.contains(&index)) {
            return Err(StressError::NameStressOnWeakNucleus {
                index: marked.unwrap_or(0),
            });
        }
        match choice {
            StressChoice::Unstressed => return Err(StressError::UnstressedNameUnsupported),
            StressChoice::Default => marked.or_else(|| match eligible.len() {
                0 => None,
                1 => eligible.first().copied(),
                n => Some(eligible[n - 2]),
            }),
            StressChoice::Nucleus(index) if !eligible.contains(&index) => {
                return Err(StressError::IneligibleNucleus { index });
            }
            StressChoice::Nucleus(index) if marked.is_some_and(|mark| mark != index) => {
                return Err(StressError::NameNotationRequired);
            }
            StressChoice::Nucleus(index) => {
                if marked.is_none()
                    && eligible.get(eligible.len().saturating_sub(2)) != Some(&index)
                {
                    return Err(StressError::NameNotationRequired);
                }
                Some(index)
            }
        }
    } else {
        match choice {
            StressChoice::Default | StressChoice::Unstressed => None,
            StressChoice::Nucleus(index) if !eligible.contains(&index) => {
                return Err(StressError::IneligibleNucleus { index });
            }
            StressChoice::Nucleus(index) => Some(index),
        }
    };

    // The source pause guard is about the last *actual* nucleus, not merely
    // the last countable nucleus. A trailing weak y or selected syllabic
    // consonant therefore cannot make an earlier nucleus "final".
    if primary.is_some_and(|index| Some(index) == nuclei.len().checked_sub(1)) {
        match (analysis.word_kind(), following) {
            (WordKind::Cmavo, FollowingContext::BrivlaUnpaused) => {
                return Err(StressError::PauseRequiredBeforeBrivla);
            }
            (WordKind::Cmavo, FollowingContext::Unspecified) => {
                limitations.push("stress.following-brivla-unchecked");
            }
            _ => {}
        }
    }
    let levels = nuclei
        .iter()
        .enumerate()
        .map(|(index, nucleus)| {
            if !nucleus.stress_countable() {
                StressLevel::Weak
            } else if Some(index) == primary {
                StressLevel::Primary
            } else if choice == StressChoice::Unstressed {
                StressLevel::Weak
            } else {
                StressLevel::OptionalSecondary
            }
        })
        .collect();
    if analysis.word_kind() == WordKind::Cmevla && eligible.is_empty() {
        limitations.push("stress.name-no-countable-nuclei");
    }
    Ok(StressAnalysis {
        primary_nucleus: primary,
        levels,
        limitations,
        source_id: analysis.source_id(),
        policy_id: STRESS_POLICY,
    })
}

// Keep the diagnostic type imported and checked against the accepted analysis API.
fn _diagnostic_sections(diagnostic: &AnalysisDiagnostic) -> &'static [&'static str] {
    match diagnostic.kind() {
        DiagnosticKind::ClusterRule(reason) => reason.source_sections(),
        _ => diagnostic.source_sections(),
    }
}
