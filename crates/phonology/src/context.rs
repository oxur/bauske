//! Source-grounded inspection of written word contexts and nuclei.

use std::{error::Error, fmt, ops::Range};

use crate::{
    AlphabetSymbol, ClusterError, ClusterInspection, ClusterPosition, ClusterViolation,
    DiphthongDistribution, DiphthongRecord, GlideDirection, SOURCE_ID, diphthongs, inspect_cluster,
};

const VOWELS: &str = "aeiouy";
const SONORANTS: &str = "lmnr";
const SEC33: &[&str] = &["3.3"];
const SEC34: &[&str] = &["3.4"];
const SEC36_37: &[&str] = &["3.6", "3.7"];

/// A caller-declared word category for the contextual source rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum WordKind {
    /// A declared content word.
    Brivla,
    /// A declared structural word.
    Cmavo,
    /// A declared proper name with stress capitalization.
    Cmevla,
    /// A declared zi'evla subtype with canonical syllabic sonorants.
    Zihevla,
    /// A declared gismu-based lujvo subtype.
    GismuBasedLujvo,
}

impl WordKind {
    /// Returns the stable command/API label for this declared category.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Brivla => "brivla",
            Self::Cmavo => "cmavo",
            Self::Cmevla => "cmevla",
            Self::Zihevla => "zihevla",
            Self::GismuBasedLujvo => "gismu-lujvo",
        }
    }
}

/// The typed source identity of one written nucleus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum NucleusKind {
    /// A single written vowel, including pure y.
    Vowel(AlphabetSymbol),
    /// A source-listed two-letter diphthong.
    Diphthong(&'static DiphthongRecord),
    /// A caller-selected syllabic l, m, n, or r.
    SyllabicConsonant(AlphabetSymbol),
}

/// A nucleus with an exact original-input byte span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nucleus {
    start: usize,
    end: usize,
    spelling: String,
    kind: NucleusKind,
    stress_countable: bool,
}

impl Nucleus {
    /// Returns the inclusive start byte offset in the original input.
    #[must_use]
    pub const fn start(&self) -> usize {
        self.start
    }

    /// Returns the exclusive end byte offset in the original input.
    #[must_use]
    pub const fn end(&self) -> usize {
        self.end
    }

    /// Returns the exact original substring covered by this nucleus span.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the typed vowel, diphthong, or caller-selected consonant.
    #[must_use]
    pub const fn kind(&self) -> NucleusKind {
        self.kind
    }

    /// Reports whether this written nucleus counts for stress purposes.
    #[must_use]
    pub const fn stress_countable(&self) -> bool {
        self.stress_countable
    }
}

/// A word-level cluster with canonical adjacency and original text offsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordCluster {
    start: usize,
    end: usize,
    original_spelling: String,
    inspection: ClusterInspection,
}

impl WordCluster {
    /// Returns the inclusive start byte offset in the original input.
    #[must_use]
    pub const fn start(&self) -> usize {
        self.start
    }

    /// Returns the exclusive end byte offset in the original input.
    #[must_use]
    pub const fn end(&self) -> usize {
        self.end
    }

    /// Returns original text across the cluster span, including comma hints.
    #[must_use]
    pub fn original_spelling(&self) -> &str {
        &self.original_spelling
    }

    /// Returns the shared public consonant-cluster inspection.
    #[must_use]
    pub const fn inspection(&self) -> &ClusterInspection {
        &self.inspection
    }
}

/// A named diagnostic produced by non-fatal source-policy inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DiagnosticKind {
    /// A source consonant-cluster rule finding.
    ClusterRule(ClusterViolation),
    /// A four-letter non-name medial run lies in the §3.6/§3.7 tension.
    SourceLengthTension {
        /// Listed initial triple in the run's final three consonants, if any.
        last_three_witness: Option<&'static str>,
    },
    /// A selected sonorant syllabicity is not canonical for this declared kind.
    OptionalSyllabicConsonant,
    /// A comma lies inside the deterministic spelling-based nucleus.
    CommaInsideNucleus,
    /// A source-restricted on-glide occurs in a declared gismu-based lujvo.
    RestrictedOnGlide {
        /// Exact source-listed diphthong spelling.
        spelling: &'static str,
    },
}

impl DiagnosticKind {
    /// Returns the stable diagnostic code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::ClusterRule(reason) => reason.code(),
            Self::SourceLengthTension { .. } => "analysis.source-length-tension",
            Self::OptionalSyllabicConsonant => "analysis.optional-syllabic-sonorant",
            Self::CommaInsideNucleus => "analysis.comma-inside-nucleus",
            Self::RestrictedOnGlide { .. } => "analysis.restricted-on-glide",
        }
    }

    /// Returns the grammar sections that support the diagnostic.
    #[must_use]
    pub const fn source_sections(&self) -> &'static [&'static str] {
        match self {
            Self::ClusterRule(reason) => reason.source_sections(),
            Self::SourceLengthTension { .. } => SEC36_37,
            Self::OptionalSyllabicConsonant => SEC34,
            Self::CommaInsideNucleus => SEC33,
            Self::RestrictedOnGlide { .. } => SEC34,
        }
    }
}

/// A diagnostic and optional original-input byte span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisDiagnostic {
    kind: DiagnosticKind,
    span: Option<Range<usize>>,
}

impl AnalysisDiagnostic {
    /// Returns the typed finding.
    #[must_use]
    pub const fn kind(&self) -> &DiagnosticKind {
        &self.kind
    }

    /// Returns the stable code for this finding.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.kind.code()
    }

    /// Returns the source sections supporting this finding.
    #[must_use]
    pub const fn source_sections(&self) -> &'static [&'static str] {
        self.kind.source_sections()
    }

    /// Returns the original-input byte span, when the finding is localized.
    #[must_use]
    pub fn span(&self) -> Option<Range<usize>> {
        self.span.clone()
    }
}

/// A complete spelling-level inspection without a grammatical-validity verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordAnalysis {
    original: String,
    canonical_spelling: String,
    kind: WordKind,
    nuclei: Vec<Nucleus>,
    clusters: Vec<WordCluster>,
    comma_positions: Vec<usize>,
    capital_positions: Vec<usize>,
    written_pause_before: bool,
    written_pause_after: bool,
    required_pause_before: bool,
    required_pause_after: bool,
    diagnostics: Vec<AnalysisDiagnostic>,
}

impl WordAnalysis {
    /// Returns the exact spelling supplied by the caller.
    #[must_use]
    pub fn original(&self) -> &str {
        &self.original
    }

    /// Returns the case-folded body without edge periods or comma hints.
    #[must_use]
    pub fn canonical_spelling(&self) -> &str {
        &self.canonical_spelling
    }

    /// Returns the caller-declared word category.
    #[must_use]
    pub const fn word_kind(&self) -> WordKind {
        self.kind
    }

    /// Returns the acquired grammar snapshot identity.
    #[must_use]
    pub const fn source_id(&self) -> &'static str {
        SOURCE_ID
    }

    /// Returns the spelling-analysis policy identity.
    #[must_use]
    pub const fn analysis_policy_id(&self) -> &'static str {
        "standard-onsets-v1"
    }

    /// Returns nuclei in original-input order.
    #[must_use]
    pub fn nuclei(&self) -> &[Nucleus] {
        &self.nuclei
    }

    /// Returns consonant runs in original-input order.
    #[must_use]
    pub fn clusters(&self) -> &[WordCluster] {
        &self.clusters
    }

    /// Returns zero-based original byte offsets of comma hints.
    #[must_use]
    pub fn comma_positions(&self) -> &[usize] {
        &self.comma_positions
    }

    /// Returns zero-based original byte offsets of capital letters.
    #[must_use]
    pub fn capital_positions(&self) -> &[usize] {
        &self.capital_positions
    }

    /// Reports whether a leading period was written as a pause reminder.
    #[must_use]
    pub const fn written_pause_before(&self) -> bool {
        self.written_pause_before
    }

    /// Reports whether a trailing period was written as a pause reminder.
    #[must_use]
    pub const fn written_pause_after(&self) -> bool {
        self.written_pause_after
    }

    /// Reports whether the source rules require a preceding pause.
    #[must_use]
    pub const fn required_pause_before(&self) -> bool {
        self.required_pause_before
    }

    /// Reports whether the source rules require a following pause.
    #[must_use]
    pub const fn required_pause_after(&self) -> bool {
        self.required_pause_after
    }

    /// Returns named source-policy observations without judging word legality.
    #[must_use]
    pub fn diagnostics(&self) -> &[AnalysisDiagnostic] {
        &self.diagnostics
    }
}

/// A malformed word-inspection request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnalysisError {
    /// No spelling was supplied.
    Empty,
    /// The character is outside the supported written alphabet.
    UnknownCharacter {
        /// Original UTF-8 byte offset.
        offset: usize,
        /// The unaccepted character.
        found: char,
    },
    /// Whitespace separates multiple words or appears inside the request.
    Whitespace {
        /// Original UTF-8 byte offset.
        offset: usize,
    },
    /// An embedded period denotes multiple written words.
    MultipleWords {
        /// Original UTF-8 byte offset of the embedded period.
        offset: usize,
    },
    /// Uppercase letters are supported only for declared cmevla.
    UppercaseOutsideName {
        /// Original UTF-8 byte offset.
        offset: usize,
        /// The uppercase letter.
        found: char,
    },
    /// An edge marker or comma placement is malformed.
    MalformedBoundary {
        /// Original UTF-8 byte offset when identifiable.
        offset: usize,
    },
    /// Apostrophe is not between two written vowels.
    ApostrophePlacement {
        /// Original UTF-8 byte offset.
        offset: usize,
    },
    /// A caller annotation does not identify l, m, n, or r in the body.
    InvalidSyllabicPosition {
        /// Requested original byte position.
        position: usize,
    },
    /// A caller repeated an original byte position.
    DuplicateSyllabicPosition {
        /// Repeated original byte position.
        position: usize,
    },
    /// A vowel run cannot be partitioned using the assigned policy.
    UnsegmentableVowelRun {
        /// Inclusive original byte start.
        start: usize,
        /// Exclusive original byte end.
        end: usize,
    },
    /// No written vowel or explicitly selected sonorant forms a nucleus.
    NoNuclei,
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty word spelling"),
            Self::UnknownCharacter { offset, found } => {
                write!(f, "unknown character {found:?} at byte {offset}")
            }
            Self::Whitespace { offset } => write!(f, "whitespace at byte {offset}"),
            Self::MultipleWords { offset } => {
                write!(
                    f,
                    "embedded period at byte {offset} separates multiple words"
                )
            }
            Self::UppercaseOutsideName { offset, found } => {
                write!(f, "uppercase {found:?} at byte {offset} outside cmevla")
            }
            Self::MalformedBoundary { offset } => {
                write!(f, "malformed edge or comma boundary at byte {offset}")
            }
            Self::ApostrophePlacement { offset } => {
                write!(f, "apostrophe at byte {offset} must separate vowels")
            }
            Self::InvalidSyllabicPosition { position } => {
                write!(f, "byte {position} is not a sonorant nucleus position")
            }
            Self::DuplicateSyllabicPosition { position } => {
                write!(
                    f,
                    "syllabic position {position} was supplied more than once"
                )
            }
            Self::UnsegmentableVowelRun { start, end } => {
                write!(f, "vowel run at bytes {start}..{end} is not segmentable")
            }
            Self::NoNuclei => f.write_str("spelling contains no nucleus"),
        }
    }
}
impl Error for AnalysisError {}

/// Inspects one caller-declared word without inferring morphology or validity.
///
/// Nucleus grouping uses the deterministic `standard-onsets-v1` spelling
/// policy. Caller-provided syllabic positions use original ASCII byte offsets.
///
/// # Errors
///
/// Returns a named [`AnalysisError`] for empty, malformed, unsupported, or
/// unsegmentable input and for invalid caller-supplied syllabic positions.
pub fn analyze_word(
    input: &str,
    kind: WordKind,
    syllabic_positions: &[usize],
) -> Result<WordAnalysis, AnalysisError> {
    if input.is_empty() {
        return Err(AnalysisError::Empty);
    }
    for (offset, ch) in input.char_indices() {
        if ch.is_whitespace() {
            return Err(AnalysisError::Whitespace { offset });
        }
    }

    let bytes = input.as_bytes();
    let written_pause_before = bytes.first() == Some(&b'.');
    let written_pause_after = bytes.last() == Some(&b'.');
    let body_start = usize::from(written_pause_before);
    let body_end = input.len().saturating_sub(usize::from(written_pause_after));
    if body_start >= body_end {
        return Err(AnalysisError::MalformedBoundary { offset: body_start });
    }
    if let Some(offset) = input[body_start..body_end].find('.') {
        return Err(AnalysisError::MultipleWords {
            offset: body_start + offset,
        });
    }
    let body = &input[body_start..body_end];
    if body.starts_with(',') {
        return Err(AnalysisError::MalformedBoundary { offset: body_start });
    }
    if body.ends_with(',') {
        return Err(AnalysisError::MalformedBoundary {
            offset: body_end - 1,
        });
    }
    if let Some(offset) = body.find(",,") {
        return Err(AnalysisError::MalformedBoundary {
            offset: body_start + offset + 1,
        });
    }

    let mut comma_positions = Vec::new();
    let mut capital_positions = Vec::new();
    let mut lower = vec![0_u8; body.len()];
    for (relative, ch) in body.char_indices() {
        if ch.is_ascii_uppercase() {
            if kind != WordKind::Cmevla {
                return Err(AnalysisError::UppercaseOutsideName {
                    offset: body_start + relative,
                    found: ch,
                });
            }
            capital_positions.push(body_start + relative);
        }
        if ch == ',' {
            comma_positions.push(body_start + relative);
            lower[relative] = b',';
            continue;
        }
        if ch == '\'' {
            lower[relative] = b'\'';
            continue;
        }
        if ch.is_ascii() && (ch.is_ascii_lowercase() || ch.is_ascii_uppercase()) {
            lower[relative] = ch.to_ascii_lowercase() as u8;
            if lower[relative] != b'\''
                && (!(lower[relative] as char).is_ascii_lowercase()
                    || (!VOWELS.contains(lower[relative] as char)
                        && !is_morphological_consonant(lower[relative] as char)))
            {
                return Err(AnalysisError::UnknownCharacter {
                    offset: body_start + relative,
                    found: ch,
                });
            }
        } else {
            return Err(AnalysisError::UnknownCharacter {
                offset: body_start + relative,
                found: ch,
            });
        }
    }
    let lower_body = std::str::from_utf8(&lower).map_err(|_| AnalysisError::UnknownCharacter {
        offset: body_start,
        found: '\u{fffd}',
    })?;

    validate_apostrophes(&lower, body_start)?;
    let selected =
        validate_syllabic_positions(input, body_start, body_end, &lower, syllabic_positions)?;

    let mut nuclei = Vec::new();
    let mut diagnostics = Vec::new();
    scan_vowel_runs(
        input,
        body_start,
        body_end,
        &lower,
        &comma_positions,
        &mut nuclei,
        &mut diagnostics,
    )?;
    for &position in &selected {
        let ch = lower[position - body_start] as char;
        let symbol = parse_symbol(ch).ok_or(AnalysisError::InvalidSyllabicPosition { position })?;
        nuclei.push(Nucleus {
            start: position,
            end: position + 1,
            spelling: input[position..position + 1].to_owned(),
            kind: NucleusKind::SyllabicConsonant(symbol),
            stress_countable: false,
        });
        if !matches!(kind, WordKind::Zihevla | WordKind::Cmevla) {
            diagnostics.push(AnalysisDiagnostic {
                kind: DiagnosticKind::OptionalSyllabicConsonant,
                span: Some(position..position + 1),
            });
        }
    }
    nuclei.sort_by_key(Nucleus::start);
    if nuclei.is_empty() {
        return Err(AnalysisError::NoNuclei);
    }

    let (clusters, mut cluster_diagnostics) =
        scan_clusters(input, body_start, body_end, &lower, kind)?;
    diagnostics.append(&mut cluster_diagnostics);

    for nucleus in &nuclei {
        if let NucleusKind::Diphthong(record) = nucleus.kind
            && kind == WordKind::GismuBasedLujvo
            && record.direction() == GlideDirection::On
            && record.distribution() == DiphthongDistribution::RestrictedMorphology
        {
            diagnostics.push(AnalysisDiagnostic {
                kind: DiagnosticKind::RestrictedOnGlide {
                    spelling: record.spelling(),
                },
                span: Some(nucleus.start..nucleus.end),
            });
        }
    }

    let body_chars: Vec<(usize, u8)> = lower
        .iter()
        .enumerate()
        .filter(|(_, byte)| **byte != b',')
        .map(|(i, byte)| (body_start + i, *byte))
        .collect();
    let first = body_chars.first().map(|(_, b)| *b as char);
    let last = body_chars.last().map(|(_, b)| *b as char);
    let ends_in_consonant = last.is_some_and(is_morphological_consonant);
    let non_name_final_consonant = ends_in_consonant && kind != WordKind::Cmevla;
    if non_name_final_consonant
        && !diagnostics
            .iter()
            .any(|item| item.code() == ClusterViolation::NonNameFinalConsonant.code())
    {
        let (offset, _) = body_chars.last().ok_or(AnalysisError::NoNuclei)?;
        diagnostics.push(AnalysisDiagnostic {
            kind: DiagnosticKind::ClusterRule(ClusterViolation::NonNameFinalConsonant),
            span: Some(*offset..*offset + 1),
        });
    }
    let required_pause_before = first.is_some_and(|ch| VOWELS.contains(ch)) || ends_in_consonant;
    let required_pause_after = ends_in_consonant;

    diagnostics.sort_by_key(|diagnostic| {
        (
            diagnostic
                .span
                .as_ref()
                .map_or(usize::MAX, |span| span.start),
            diagnostic.code(),
        )
    });
    let canonical_spelling = lower_body.replace(',', "");
    Ok(WordAnalysis {
        original: input.to_owned(),
        canonical_spelling,
        kind,
        nuclei,
        clusters,
        comma_positions,
        capital_positions,
        written_pause_before,
        written_pause_after,
        required_pause_before,
        required_pause_after,
        diagnostics,
    })
}

fn validate_apostrophes(body: &[u8], body_start: usize) -> Result<(), AnalysisError> {
    let without_hints: Vec<(usize, u8)> = body
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, byte)| *byte != b',')
        .collect();
    for (index, &(original_offset, byte)) in without_hints.iter().enumerate() {
        if byte == b'\'' {
            let valid = index > 0
                && index + 1 < without_hints.len()
                && VOWELS.contains(without_hints[index - 1].1 as char)
                && VOWELS.contains(without_hints[index + 1].1 as char);
            if !valid {
                return Err(AnalysisError::ApostrophePlacement {
                    offset: body_start + original_offset,
                });
            }
        }
    }
    Ok(())
}

fn validate_syllabic_positions(
    input: &str,
    body_start: usize,
    body_end: usize,
    lower: &[u8],
    positions: &[usize],
) -> Result<Vec<usize>, AnalysisError> {
    let mut selected = Vec::with_capacity(positions.len());
    for &position in positions {
        if selected.contains(&position) {
            return Err(AnalysisError::DuplicateSyllabicPosition { position });
        }
        if position < body_start || position >= body_end {
            return Err(AnalysisError::InvalidSyllabicPosition { position });
        }
        let Some(byte) = input.as_bytes().get(position) else {
            return Err(AnalysisError::InvalidSyllabicPosition { position });
        };
        let relative = position - body_start;
        if lower.get(relative).copied() != Some(byte.to_ascii_lowercase())
            || !SONORANTS.contains(lower[relative] as char)
        {
            return Err(AnalysisError::InvalidSyllabicPosition { position });
        }
        selected.push(position);
    }
    Ok(selected)
}

fn scan_vowel_runs(
    original: &str,
    body_start: usize,
    body_end: usize,
    lower: &[u8],
    comma_positions: &[usize],
    nuclei: &mut Vec<Nucleus>,
    diagnostics: &mut Vec<AnalysisDiagnostic>,
) -> Result<(), AnalysisError> {
    let mut cursor = body_start;
    while cursor < body_end {
        let relative = cursor - body_start;
        let byte = lower[relative];
        if !VOWELS.contains(byte as char) {
            cursor += 1;
            continue;
        }
        let run_start = cursor;
        let mut vowel_positions = Vec::new();
        while cursor < body_end {
            let current = lower[cursor - body_start];
            if VOWELS.contains(current as char) {
                vowel_positions.push(cursor);
                cursor += 1;
            } else if current == b',' {
                cursor += 1;
            } else {
                break;
            }
        }
        let run_end = cursor;
        let mut offset = 0;
        let first_width = if vowel_positions.len() % 2 == 1 { 1 } else { 2 };
        let mut first = true;
        while offset < vowel_positions.len() {
            let width = if first { first_width } else { 2 };
            if offset + width > vowel_positions.len() {
                return Err(AnalysisError::UnsegmentableVowelRun {
                    start: run_start,
                    end: run_end,
                });
            }
            let start = vowel_positions[offset];
            let end = vowel_positions[offset + width - 1] + 1;
            let spelling = vowel_positions[offset..offset + width]
                .iter()
                .map(|position| lower[*position - body_start] as char)
                .collect::<String>();
            let (kind, stress_countable) = if width == 1 {
                let vowel = spelling.as_bytes()[0] as char;
                let Some(symbol) = parse_symbol(vowel) else {
                    return Err(AnalysisError::UnsegmentableVowelRun { start, end });
                };
                (NucleusKind::Vowel(symbol), vowel != 'y')
            } else {
                let Some(record) = diphthongs()
                    .iter()
                    .find(|record| record.spelling() == spelling)
                else {
                    return Err(AnalysisError::UnsegmentableVowelRun {
                        start: run_start,
                        end: run_end,
                    });
                };
                if !first && record.direction() != GlideDirection::On {
                    return Err(AnalysisError::UnsegmentableVowelRun {
                        start: run_start,
                        end: run_end,
                    });
                }
                (
                    NucleusKind::Diphthong(record),
                    !record.spelling().ends_with('y'),
                )
            };
            nuclei.push(Nucleus {
                start,
                end,
                spelling: original[start..end].to_owned(),
                kind,
                stress_countable,
            });
            for &comma in comma_positions {
                if comma > start && comma < end {
                    diagnostics.push(AnalysisDiagnostic {
                        kind: DiagnosticKind::CommaInsideNucleus,
                        span: Some(comma..comma + 1),
                    });
                }
            }
            offset += width;
            first = false;
        }
    }
    Ok(())
}

fn scan_clusters(
    original: &str,
    body_start: usize,
    body_end: usize,
    lower: &[u8],
    kind: WordKind,
) -> Result<(Vec<WordCluster>, Vec<AnalysisDiagnostic>), AnalysisError> {
    let mut clusters = Vec::new();
    let mut diagnostics = Vec::new();
    let mut cursor = 0;
    while cursor < lower.len() {
        if !is_morphological_consonant(lower[cursor] as char) {
            cursor += 1;
            continue;
        }
        let run_start = cursor;
        let mut canonical = String::new();
        while cursor < lower.len() && is_morphological_consonant(lower[cursor] as char) {
            canonical.push(lower[cursor] as char);
            cursor += 1;
            let after_consonant = cursor;
            while cursor < lower.len() && lower[cursor] == b',' {
                cursor += 1;
            }
            if cursor < lower.len() && is_morphological_consonant(lower[cursor] as char) {
                continue;
            }
            cursor = after_consonant;
            break;
        }
        let run_end = cursor;
        let original_start = body_start + run_start;
        let original_end = body_start + run_end;
        let position = if !lower[..run_start].iter().any(|byte| *byte != b',') {
            ClusterPosition::Initial
        } else if !lower[run_end..].iter().any(|byte| *byte != b',') {
            ClusterPosition::Final
        } else {
            ClusterPosition::Medial
        };
        let inspection =
            inspect_cluster(&canonical, position, kind).map_err(|error| match error {
                ClusterError::Empty => AnalysisError::NoNuclei,
                ClusterError::InvalidSymbol { offset, found } => AnalysisError::UnknownCharacter {
                    offset: original_start + offset,
                    found,
                },
            })?;
        for reason in inspection.violations() {
            diagnostics.push(AnalysisDiagnostic {
                kind: DiagnosticKind::ClusterRule(*reason),
                span: Some(original_start..original_end),
            });
        }
        if inspection.has_source_length_tension() {
            diagnostics.push(AnalysisDiagnostic {
                kind: DiagnosticKind::SourceLengthTension {
                    last_three_witness: inspection.last_three_initial_triple(),
                },
                span: Some(original_start..original_end),
            });
        }
        clusters.push(WordCluster {
            start: original_start,
            end: original_end,
            original_spelling: original[original_start..original_end].to_owned(),
            inspection,
        });
    }
    if body_start > body_end {
        return Err(AnalysisError::MalformedBoundary { offset: body_start });
    }
    Ok((clusters, diagnostics))
}

fn parse_symbol(ch: char) -> Option<AlphabetSymbol> {
    ch.to_string().parse().ok()
}

fn is_morphological_consonant(ch: char) -> bool {
    "bcdfgjklmnprstvxz".contains(ch)
}
