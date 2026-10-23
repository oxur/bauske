use std::io::{self, Write};

use bauske_phonology::{
    AnalysisError, ClusterError, CompareError, DiagnosticKind, DistanceModel, FeatureValue,
    FollowingContext, NucleusKind, ParseSegmentError, PronunciationProfile, RealizationError,
    SOURCE_ID, Segment, StressAnalysis, StressChoice, StressError, StressLevel, SymbolKind,
    WordAnalysis, alphabet, analyze_word, assign_stress, diphthongs, inspect_cluster,
    plan_realization, profiles, syllabify,
};

use crate::args::{ClusterPositionArg, FollowingArg, InventoryKind, SyllablesArg, WordKindArg};

pub(crate) enum RenderError {
    Io(io::Error),
    Parse(ParseSegmentError),
    Compare(CompareError),
    Analysis(AnalysisError),
    Cluster(ClusterError),
    Stress(StressError),
    Realization(RealizationError),
}

/// Writes one selected inventory as deterministic tab-separated values.
pub(crate) fn render_inventory<W: Write>(kind: InventoryKind, mut writer: W) -> io::Result<()> {
    if kind == InventoryKind::Diphthong {
        writeln!(
            writer,
            "spelling\tipa\tglide\tdirection\tdistribution\tsource_sections"
        )?;
        for record in diphthongs() {
            let glide = match record.glide() {
                bauske_phonology::Glide::Palatal => "palatal",
                bauske_phonology::Glide::Labial => "labial",
            };
            let direction = match record.direction() {
                bauske_phonology::GlideDirection::On => "on",
                bauske_phonology::GlideDirection::Off => "off",
            };
            let distribution = match record.distribution() {
                bauske_phonology::DiphthongDistribution::FreelyUsed => "freely-used",
                bauske_phonology::DiphthongDistribution::RestrictedMorphology => {
                    "restricted-morphology"
                }
            };
            writeln!(
                writer,
                "{}\t{}\t{}\t{}\t{}\t{}",
                record.spelling(),
                record.ipa(),
                glide,
                direction,
                distribution,
                record.source_sections().join(";")
            )?;
        }
        return Ok(());
    }
    writeln!(
        writer,
        "symbol\tkind\trepresentative_ipa\tsource_sections\tvariant_policy"
    )?;
    for record in alphabet() {
        let include = match kind {
            InventoryKind::Alphabet => true,
            InventoryKind::Consonant => record.kind() == SymbolKind::Consonant,
            InventoryKind::Vowel => record.kind() == SymbolKind::Vowel,
            InventoryKind::Diphthong => false,
        };
        if !include {
            continue;
        }
        let policies = if record.policies().is_empty() {
            "none".to_owned()
        } else {
            record
                .policies()
                .iter()
                .map(|policy| policy.label())
                .collect::<Vec<_>>()
                .join(";")
        };
        writeln!(
            writer,
            "{}\t{}\t{}\t{}\t{}",
            record.symbol(),
            record.kind(),
            record.representative_ipa().unwrap_or(""),
            record.source_sections().join(";"),
            policies
        )?;
    }
    Ok(())
}

/// Writes a comparison using the identified profile selected by the caller.
pub(crate) fn render_comparison_with_profile<W: Write>(
    left: &str,
    right: &str,
    profile: PronunciationProfile,
    mut writer: W,
) -> Result<(), RenderError> {
    let left_segment = left.parse::<Segment>().map_err(RenderError::Parse)?;
    let right_segment = right.parse::<Segment>().map_err(RenderError::Parse)?;
    let model = DistanceModel::default().with_profile(profile);
    let result = model
        .compare(left_segment, right_segment)
        .map_err(RenderError::Compare)?;
    writeln!(writer, "profile: {}", model.profile_id()).map_err(RenderError::Io)?;
    writeln!(writer, "source: {}", model.source_id()).map_err(RenderError::Io)?;
    writeln!(
        writer,
        "ipa: {} / {}",
        profile.ipa(left_segment),
        profile.ipa(right_segment)
    )
    .map_err(RenderError::Io)?;
    writeln!(
        writer,
        "articulatory baseline; perceptual status unvalidated"
    )
    .map_err(RenderError::Io)?;
    writeln!(
        writer,
        "inputs: {} / {}",
        result.left().as_str(),
        result.right().as_str()
    )
    .map_err(RenderError::Io)?;
    writeln!(writer, "total: {}", result.total()).map_err(RenderError::Io)?;
    writeln!(writer, "dimension\tleft\tright\tweight\tcost").map_err(RenderError::Io)?;
    for item in result.contributions() {
        writeln!(
            writer,
            "{}\t{}\t{}\t{}\t{}",
            item.dimension().label(),
            display_feature(item.left()),
            display_feature(item.right()),
            item.weight(),
            item.cost()
        )
        .map_err(RenderError::Io)?;
    }
    Ok(())
}

/// Writes the immutable identified pronunciation profiles.
pub(crate) fn render_profiles<W: Write>(mut writer: W) -> io::Result<()> {
    writeln!(
        writer,
        "profile\tsource\te_ipa\te_choice\tperceptual_status"
    )?;
    for profile in profiles() {
        let e: Segment = "e".parse().map_err(|error: ParseSegmentError| {
            io::Error::new(io::ErrorKind::InvalidData, error)
        })?;
        let choice = match profile {
            PronunciationProfile::RepresentativesV1 => "preferred-open-mid",
            PronunciationProfile::CloseMidEV1 => "allowed-close-mid",
            _ => "unspecified-profile-choice",
        };
        writeln!(
            writer,
            "{}\t{}\t{}\t{}\tunvalidated",
            profile.id(),
            profile.source_id(),
            profile.ipa(e),
            choice
        )?;
    }
    Ok(())
}

/// Complete caller request for a syllable/stress/buffer inspection.
pub(crate) struct ProsodyRequest<'a> {
    pub(crate) input: &'a str,
    pub(crate) kind: WordKindArg,
    pub(crate) syllabic_positions: &'a [usize],
    pub(crate) buffer_after: &'a [usize],
    pub(crate) mode: SyllablesArg,
    pub(crate) primary: Option<usize>,
    pub(crate) unstressed: bool,
    pub(crate) following: FollowingArg,
}

/// Writes a prevalidated syllable/stress/buffer inspection.
pub(crate) fn render_prosody<W: Write>(
    request: ProsodyRequest<'_>,
    mut writer: W,
) -> Result<(), RenderError> {
    let ProsodyRequest {
        input,
        kind,
        syllabic_positions,
        buffer_after,
        mode,
        primary,
        unstressed,
        following,
    } = request;
    let analysis =
        analyze_word(input, kind.into(), syllabic_positions).map_err(RenderError::Analysis)?;
    let choice = if let Some(index) = primary {
        StressChoice::Nucleus(index)
    } else if unstressed {
        StressChoice::Unstressed
    } else {
        StressChoice::Default
    };
    let stress = assign_stress(&analysis, choice, FollowingContext::from(following))
        .map_err(RenderError::Stress)?;
    let realization =
        plan_realization(&analysis, buffer_after).map_err(RenderError::Realization)?;
    let grouping = syllabify(&analysis, mode.into());

    write_word_analysis(&analysis, &mut writer).map_err(RenderError::Io)?;
    writeln!(writer, "prosody_policy\t{}", stress.policy_id()).map_err(RenderError::Io)?;
    writeln!(writer, "syllabification_policy\t{}", grouping.policy_id())
        .map_err(RenderError::Io)?;
    writeln!(writer, "syllable_mode\t{}", grouping.mode().id()).map_err(RenderError::Io)?;
    writeln!(
        writer,
        "syllabification_resolved\t{}",
        grouping.is_resolved()
    )
    .map_err(RenderError::Io)?;
    writeln!(
        writer,
        "syllable\tstart\tend\tspelling\tcanonical\tnucleus_index\tnucleus\tcountable"
    )
    .map_err(RenderError::Io)?;
    for syllable in grouping.syllables() {
        let nucleus = &analysis.nuclei()[syllable.nucleus_index()];
        writeln!(
            writer,
            "syllable\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            syllable.start(),
            syllable.end(),
            syllable.original_spelling(),
            syllable.canonical_spelling(),
            syllable.nucleus_index(),
            nucleus.spelling(),
            nucleus.stress_countable()
        )
        .map_err(RenderError::Io)?;
    }
    for limitation in grouping.limitations() {
        let span = limitation.span();
        writeln!(
            writer,
            "syllable_limitation\t{}\t{}\t{}..{}\t{}",
            limitation.code(),
            limitation.source_sections().join(";"),
            span.start,
            span.end,
            limitation.detail()
        )
        .map_err(RenderError::Io)?;
    }
    write_stress(&stress, &mut writer).map_err(RenderError::Io)?;
    writeln!(
        writer,
        "written_nuclei\t{}",
        realization.written_nucleus_count()
    )
    .map_err(RenderError::Io)?;
    writeln!(
        writer,
        "performed_nuclei\t{}",
        realization.performed_nucleus_count()
    )
    .map_err(RenderError::Io)?;
    writeln!(
        writer,
        "countable_nuclei\t{}",
        realization.countable_nucleus_count()
    )
    .map_err(RenderError::Io)?;
    for event in realization.events() {
        writeln!(
            writer,
            "buffer\t{}\t{}\tweak\t{}",
            event.after_position(),
            event.performed_nucleus_index(),
            event.source_sections().join(";")
        )
        .map_err(RenderError::Io)?;
    }
    writeln!(writer, "realization_policy\t{}", realization.policy_id()).map_err(RenderError::Io)?;
    writeln!(
        writer,
        "realization\tunmeasured caller-annotated plan; no audio inferred"
    )
    .map_err(RenderError::Io)?;
    Ok(())
}

fn write_stress<W: Write>(stress: &StressAnalysis, writer: &mut W) -> io::Result<()> {
    writeln!(
        writer,
        "primary_nucleus\t{}",
        stress
            .primary_nucleus()
            .map_or_else(|| "none".to_owned(), |index| index.to_string())
    )?;
    for (index, level) in stress.levels().iter().enumerate() {
        let label = match level {
            StressLevel::Primary => "primary",
            StressLevel::Weak => "weak",
            StressLevel::OptionalSecondary => "optional-secondary",
            _ => "unspecified-stress-level",
        };
        writeln!(writer, "stress\t{index}\t{label}")?;
    }
    for limitation in stress.limitations() {
        writeln!(writer, "stress_limitation\t{limitation}")?;
    }
    Ok(())
}

fn display_feature(value: FeatureValue) -> &'static str {
    value.label()
}

/// Writes a spelling-level word inspection after validating the complete request.
pub(crate) fn render_analysis<W: Write>(
    input: &str,
    kind: WordKindArg,
    syllabic_positions: &[usize],
    mut writer: W,
) -> Result<(), RenderError> {
    let analysis =
        analyze_word(input, kind.into(), syllabic_positions).map_err(RenderError::Analysis)?;
    write_word_analysis(&analysis, &mut writer).map_err(RenderError::Io)
}

/// Writes a consonant-cluster inspection after validating the complete request.
pub(crate) fn render_cluster<W: Write>(
    cluster: &str,
    position: ClusterPositionArg,
    kind: WordKindArg,
    mut writer: W,
) -> Result<(), RenderError> {
    let inspection =
        inspect_cluster(cluster, position.into(), kind.into()).map_err(RenderError::Cluster)?;
    writeln!(writer, "source	{SOURCE_ID}").map_err(RenderError::Io)?;
    writeln!(writer, "cluster	{}", inspection.cluster()).map_err(RenderError::Io)?;
    writeln!(writer, "position	{}", position_name(inspection.position()))
        .map_err(RenderError::Io)?;
    writeln!(writer, "kind	{}", inspection.word_kind().label()).map_err(RenderError::Io)?;
    writeln!(
        writer,
        "result	phonological inspection; morphology unverified"
    )
    .map_err(RenderError::Io)?;
    for reason in inspection.violations() {
        writeln!(
            writer,
            "diagnostic	{}	{}",
            reason.code(),
            reason.source_sections().join(";")
        )
        .map_err(RenderError::Io)?;
    }
    if let Some(tension) = inspection.source_length_tension() {
        writeln!(
            writer,
            "diagnostic\t{}\t{}\twitness={}",
            tension.code(),
            tension.source_sections().join(";"),
            tension.last_three_initial_triple().unwrap_or("none")
        )
        .map_err(RenderError::Io)?;
    }
    Ok(())
}

fn write_word_analysis<W: Write>(analysis: &WordAnalysis, writer: &mut W) -> io::Result<()> {
    writeln!(writer, "source	{}", analysis.source_id())?;
    writeln!(writer, "policy	{}", analysis.analysis_policy_id())?;
    writeln!(writer, "kind	{}", analysis.word_kind().label())?;
    writeln!(
        writer,
        "result	phonological inspection; morphology unverified"
    )?;
    writeln!(writer, "original	{}", analysis.original())?;
    writeln!(writer, "canonical	{}", analysis.canonical_spelling())?;
    writeln!(
        writer,
        "written_pause_before	{}",
        analysis.written_pause_before()
    )?;
    writeln!(
        writer,
        "written_pause_after	{}",
        analysis.written_pause_after()
    )?;
    writeln!(
        writer,
        "required_pause_before	{}",
        analysis.required_pause_before()
    )?;
    writeln!(
        writer,
        "required_pause_after	{}",
        analysis.required_pause_after()
    )?;
    writeln!(
        writer,
        "comma_positions	{}",
        join_positions(analysis.comma_positions())
    )?;
    writeln!(
        writer,
        "capital_positions	{}",
        join_positions(analysis.capital_positions())
    )?;
    writeln!(writer, "start	end	spelling	kind	stress_countable")?;
    for nucleus in analysis.nuclei() {
        writeln!(
            writer,
            "{}	{}	{}	{}	{}",
            nucleus.start(),
            nucleus.end(),
            nucleus.spelling(),
            nucleus_label(nucleus.kind()),
            nucleus.stress_countable()
        )?;
    }
    for diagnostic in analysis.diagnostics() {
        let span = diagnostic
            .span()
            .map(|range| format!("{}..{}", range.start, range.end))
            .unwrap_or_else(|| "-".to_owned());
        let detail = match diagnostic.kind() {
            DiagnosticKind::ClusterRule(_) => {
                let cluster = analysis
                    .clusters()
                    .iter()
                    .find(|cluster| Some(cluster.start()..cluster.end()) == diagnostic.span());
                if let Some(cluster) = cluster {
                    format!(
                        "spelling={} canonical={}",
                        cluster.original_spelling(),
                        cluster.inspection().cluster()
                    )
                } else {
                    format!("{:?}", diagnostic.kind())
                }
            }
            DiagnosticKind::SourceLengthTension { last_three_witness } => {
                format!(
                    "last-three-witness={}",
                    last_three_witness.unwrap_or("none")
                )
            }
            DiagnosticKind::RestrictedOnGlide { spelling } => {
                format!("diphthong={spelling}")
            }
            other => format!("{other:?}"),
        };
        writeln!(
            writer,
            "diagnostic	{}	{}	{}	{}",
            diagnostic.code(),
            diagnostic.source_sections().join(";"),
            span,
            detail
        )?;
    }
    Ok(())
}

fn nucleus_label(kind: NucleusKind) -> String {
    match kind {
        NucleusKind::Vowel(symbol) => format!("vowel:{}", symbol.as_str()),
        NucleusKind::Diphthong(record) => format!("diphthong:{}", record.spelling()),
        NucleusKind::SyllabicConsonant(symbol) => {
            format!("syllabic-consonant:{}", symbol.as_str())
        }
        _ => "unknown-nucleus".to_owned(),
    }
}

fn position_name(position: bauske_phonology::ClusterPosition) -> &'static str {
    match position {
        bauske_phonology::ClusterPosition::Initial => "initial",
        bauske_phonology::ClusterPosition::Medial => "medial",
        bauske_phonology::ClusterPosition::Final => "final",
        _ => "unknown",
    }
}

fn join_positions(positions: &[usize]) -> String {
    if positions.is_empty() {
        "none".to_owned()
    } else {
        positions
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
