//! Public-contract tests for contextual spelling and consonant clusters.

use bauske_phonology::{
    ClusterPosition, ClusterViolation, NucleusKind, WordKind, analyze_word, initial_pairs,
    initial_triples, inspect_cluster,
};

fn analyze(input: &str, kind: WordKind) -> bauske_phonology::WordAnalysis {
    analyze_word(input, kind, &[]).expect("source-policy witness parses")
}

fn contains_code(analysis: &bauske_phonology::WordAnalysis, code: &str) -> bool {
    analysis
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code() == code)
}

#[test]
fn ordered_cluster_sets_match_the_complete_source_lists() {
    assert_eq!(initial_pairs().len(), 48);
    assert_eq!(
        initial_pairs(),
        &[
            "bl", "br", "cf", "ck", "cl", "cm", "cn", "cp", "cr", "ct", "dj", "dr", "dz", "fl",
            "fr", "gl", "gr", "jb", "jd", "jg", "jm", "jv", "kl", "kr", "ml", "mr", "pl", "pr",
            "sf", "sk", "sl", "sm", "sn", "sp", "sr", "st", "tc", "tr", "ts", "vl", "vr", "xl",
            "xr", "zb", "zd", "zg", "zm", "zv",
        ]
    );
    assert_eq!(initial_triples().len(), 36);
    assert_eq!(
        initial_triples(),
        &[
            "cfr", "cfl", "sfr", "sfl", "jvr", "jvl", "zvr", "zvl", "cpr", "cpl", "spr", "spl",
            "jbr", "jbl", "zbr", "zbl", "ctr", "str", "jdr", "zdr", "ckr", "ckl", "skr", "skl",
            "jgr", "jgl", "zgr", "zgl", "cmr", "cml", "smr", "sml", "jmr", "jml", "zmr", "zml",
        ]
    );
}

fn direct_pair_reasons(a: char, b: char) -> Vec<ClusterViolation> {
    let mut expected = Vec::new();
    if a == b {
        expected.push(ClusterViolation::DuplicateNeighbors);
    }
    let sonorants = "lmnr";
    let unvoiced = "ptkfcsx";
    let voiced = "bdgvjz";
    if !sonorants.contains(a)
        && !sonorants.contains(b)
        && ((unvoiced.contains(a) && voiced.contains(b))
            || (voiced.contains(a) && unvoiced.contains(b)))
    {
        expected.push(ClusterViolation::VoicingMismatch);
    }
    if "cjsz".contains(a) && "cjsz".contains(b) {
        expected.push(ClusterViolation::MultipleSibilants);
    }
    if matches!((a, b), ('c', 'x') | ('k', 'x') | ('x', 'c') | ('x', 'k')) {
        expected.push(ClusterViolation::ForbiddenXPair);
    }
    expected
}

#[test]
fn all_289_ordered_consonant_pairs_match_an_independent_section_36_predicate() {
    let consonants = "bcdfgjklmnprstvxz";
    let mut population = 0;
    for left in consonants.chars() {
        for right in consonants.chars() {
            let pair = [left, right].iter().collect::<String>();
            let observed = inspect_cluster(&pair, ClusterPosition::Medial, WordKind::Cmevla)
                .unwrap()
                .violations()
                .to_vec();
            assert_eq!(observed, direct_pair_reasons(left, right), "{pair}");
            population += 1;
        }
    }
    assert_eq!(population, 17 * 17);
}

#[test]
fn cluster_contexts_keep_named_rules_and_the_four_letter_source_tension() {
    for rejected in ["pp", "bf", "sd", "cj", "cx", "kx", "xc", "xk"] {
        let item = inspect_cluster(rejected, ClusterPosition::Medial, WordKind::Cmevla).unwrap();
        assert!(!item.violations().is_empty(), "{rejected}");
    }
    for accepted in ["fl", "vl", "ls", "lz", "ng"] {
        assert!(
            inspect_cluster(accepted, ClusterPosition::Medial, WordKind::Cmevla)
                .unwrap()
                .violations()
                .is_empty(),
            "{accepted}"
        );
    }
    assert!(
        inspect_cluster("cn", ClusterPosition::Initial, WordKind::Brivla)
            .unwrap()
            .violations()
            .is_empty()
    );
    assert!(
        inspect_cluster("jn", ClusterPosition::Initial, WordKind::Brivla)
            .unwrap()
            .violations()
            .contains(&ClusterViolation::InitialPairUnlisted)
    );
    assert!(
        inspect_cluster("str", ClusterPosition::Initial, WordKind::Brivla)
            .unwrap()
            .violations()
            .is_empty()
    );
    for triple in ["ndj", "ndz", "ntc", "nts"] {
        assert!(
            inspect_cluster(triple, ClusterPosition::Medial, WordKind::Brivla)
                .unwrap()
                .violations()
                .contains(&ClusterViolation::ForbiddenMedialTriple)
        );
    }
    let tension = inspect_cluster("nstr", ClusterPosition::Medial, WordKind::Brivla).unwrap();
    assert!(tension.has_source_length_tension());
    let source_tension = tension.source_length_tension().unwrap();
    assert_eq!(source_tension.code(), "analysis.source-length-tension");
    assert_eq!(source_tension.source_sections(), &["3.6", "3.7"]);
    assert_eq!(source_tension.last_three_initial_triple(), Some("str"));
    assert!(
        !tension
            .violations()
            .contains(&ClusterViolation::UnsupportedLength)
    );
    assert!(
        inspect_cluster("nstr", ClusterPosition::Initial, WordKind::Cmevla)
            .unwrap()
            .violations()
            .is_empty()
    );
    assert!(
        inspect_cluster("bf", ClusterPosition::Medial, WordKind::Cmevla)
            .unwrap()
            .violations()
            .contains(&ClusterViolation::VoicingMismatch)
    );
}

#[test]
fn all_sixteen_registered_diphthongs_are_single_typed_nuclei() {
    for spelling in [
        "ai", "ei", "oi", "au", "ia", "ie", "ii", "io", "iu", "ua", "ue", "ui", "uo", "uu", "iy",
        "uy",
    ] {
        let result = analyze(spelling, WordKind::Cmavo);
        assert_eq!(result.nuclei().len(), 1, "{spelling}");
        assert!(matches!(
            result.nuclei()[0].kind(),
            NucleusKind::Diphthong(_)
        ));
        assert_eq!(result.nuclei()[0].spelling(), spelling);
        assert_eq!(
            result.nuclei()[0].stress_countable(),
            !spelling.ends_with('y')
        );
    }
    assert!(!analyze("iy", WordKind::Cmavo).nuclei()[0].stress_countable());
    assert!(!analyze("uy", WordKind::Cmavo).nuclei()[0].stress_countable());
    assert!(!analyze("y", WordKind::Cmavo).nuclei()[0].stress_countable());
}

#[test]
fn every_vowel_apostrophe_pair_has_two_nuclei() {
    let vowels = "aeiouy";
    let mut count = 0;
    for left in vowels.chars() {
        for right in vowels.chars() {
            let pair = format!("{left}'{right}");
            let result = analyze(&pair, WordKind::Cmavo);
            assert_eq!(result.nuclei().len(), 2, "{pair}");
            assert_eq!(result.nuclei()[0].spelling(), left.to_string());
            assert_eq!(result.nuclei()[1].spelling(), right.to_string());
            count += 1;
        }
    }
    assert_eq!(count, 36);
    let mut hinted_count = 0;
    for left in vowels.chars() {
        for right in vowels.chars() {
            for (spelling, right_start, comma_positions) in [
                (format!("{left},'{right}"), 3, vec![1]),
                (format!("{left}',{right}"), 3, vec![2]),
                (format!("{left},',{right}"), 4, vec![1, 3]),
            ] {
                let result = analyze(&spelling, WordKind::Cmavo);
                assert_eq!(result.canonical_spelling(), format!("{left}'{right}"));
                assert_eq!(result.comma_positions(), comma_positions);
                assert_eq!(result.nuclei().len(), 2, "{spelling}");
                assert_eq!(
                    (result.nuclei()[0].start(), result.nuclei()[0].end()),
                    (0, 1),
                    "{spelling}"
                );
                assert_eq!(
                    (result.nuclei()[1].start(), result.nuclei()[1].end()),
                    (right_start, right_start + 1),
                    "{spelling}"
                );
                assert_eq!(result.nuclei()[0].spelling(), left.to_string());
                assert_eq!(result.nuclei()[1].spelling(), right.to_string());
                assert!(matches!(result.nuclei()[0].kind(), NucleusKind::Vowel(_)));
                assert!(matches!(result.nuclei()[1].kind(), NucleusKind::Vowel(_)));
                assert_eq!(result.nuclei()[0].stress_countable(), left != 'y');
                assert_eq!(result.nuclei()[1].stress_countable(), right != 'y');
                hinted_count += 1;
            }
        }
    }
    assert_eq!(hinted_count, 36 * 3);
    for malformed in ["a,','i", "'a", "a'", "b,'a", "a',b"] {
        assert!(
            matches!(
                analyze_word(malformed, WordKind::Cmavo, &[]),
                Err(bauske_phonology::AnalysisError::ApostrophePlacement { .. })
            ),
            "{malformed:?}"
        );
    }
    assert!(matches!(
        analyze_word("a,','i", WordKind::Cmavo, &[]),
        Err(bauske_phonology::AnalysisError::ApostrophePlacement { offset: 2 })
    ));
    let diphthong = analyze("ai", WordKind::Cmavo);
    assert_eq!(diphthong.nuclei().len(), 1);
    assert_eq!(analyze("a'i", WordKind::Cmavo).nuclei().len(), 2);
}

#[test]
fn original_spans_hints_capitals_and_single_word_errors_are_explicit() {
    let meiin = analyze(".meiin.", WordKind::Cmevla);
    assert_eq!(
        meiin
            .nuclei()
            .iter()
            .map(|n| (n.start(), n.end(), n.spelling().to_owned()))
            .collect::<Vec<_>>(),
        [(2, 3, "e".to_owned()), (3, 5, "ii".to_owned())]
    );

    let song = analyze(".i,ia,ii,ia,ion.", WordKind::Cmevla);
    assert_eq!(
        song.nuclei()
            .iter()
            .map(|n| n.spelling())
            .collect::<Vec<_>>(),
        ["i", "ia", "ii", "ia", "io"]
    );
    assert_eq!(song.comma_positions(), &[2, 5, 8, 11]);
    assert!(
        !song
            .diagnostics()
            .iter()
            .any(|d| d.code() == "analysis.comma-inside-nucleus")
    );

    let name = analyze(".DJOsefin.", WordKind::Cmevla);
    assert_eq!(name.canonical_spelling(), "djosefin");
    assert_eq!(name.capital_positions(), &[1, 2, 3]);
    assert!(matches!(
        analyze("dikyjvo", WordKind::Brivla)
            .nuclei()
            .iter()
            .map(|n| n.stress_countable())
            .collect::<Vec<_>>()
            .as_slice(),
        [true, false, true]
    ));

    let comma_conflict = analyze("a,i", WordKind::Cmavo);
    assert_eq!(comma_conflict.nuclei().len(), 1);
    assert_eq!(comma_conflict.nuclei()[0].spelling(), "a,i");
    assert!(contains_code(
        &comma_conflict,
        "analysis.comma-inside-nucleus"
    ));
    for bad in ["", " ", "a.i", "’", "h", "a''i", "'ai", "a'"] {
        assert!(analyze_word(bad, WordKind::Cmavo, &[]).is_err(), "{bad:?}");
    }
    assert!(matches!(
        analyze_word("DJOsefin", WordKind::Brivla, &[]),
        Err(bauske_phonology::AnalysisError::UppercaseOutsideName { .. })
    ));
    assert!(matches!(
        analyze_word("ae", WordKind::Cmavo, &[]),
        Err(bauske_phonology::AnalysisError::UnsegmentableVowelRun { .. })
    ));
    assert!(matches!(
        analyze_word("bc", WordKind::Brivla, &[]),
        Err(bauske_phonology::AnalysisError::NoNuclei)
    ));
    let initial_comma_cluster = analyze("k,la", WordKind::Brivla);
    assert!(
        initial_comma_cluster
            .clusters()
            .iter()
            .any(|cluster| cluster.inspection().cluster() == "kl"
                && cluster.original_spelling() == "k,l"
                && cluster.inspection().position() == ClusterPosition::Initial)
    );
    assert!(analyze_word("a,", WordKind::Cmavo, &[]).is_err());
    assert!(analyze_word("a,,i", WordKind::Cmavo, &[]).is_err());
}

#[test]
fn caller_syllabicity_pauses_distribution_and_source_citations_are_preserved() {
    let syllabic = analyze_word(".rl.", WordKind::Cmevla, &[2]).unwrap();
    assert_eq!(syllabic.nuclei().len(), 1);
    assert_eq!(syllabic.nuclei()[0].spelling(), "l");
    assert!(!syllabic.nuclei()[0].stress_countable());
    assert!(matches!(
        syllabic.nuclei()[0].kind(),
        NucleusKind::SyllabicConsonant(_)
    ));
    assert!(analyze_word(".rl.", WordKind::Cmevla, &[0]).is_err());
    assert!(analyze_word(".rl.", WordKind::Cmevla, &[2, 2]).is_err());
    let optional = analyze_word("arla", WordKind::Brivla, &[1]).unwrap();
    assert!(contains_code(
        &optional,
        "analysis.optional-syllabic-sonorant"
    ));

    let i = analyze("i", WordKind::Cmavo);
    assert!(i.required_pause_before());
    assert!(!i.written_pause_before());
    assert!(analyze(".i", WordKind::Cmavo).written_pause_before());
    let name = analyze(".armstrong.", WordKind::Cmevla);
    assert!(name.required_pause_before());
    assert!(name.required_pause_after());
    assert!(name.written_pause_before() && name.written_pause_after());
    let ending = analyze("ban", WordKind::Brivla);
    assert!(ending.required_pause_before() && ending.required_pause_after());
    assert!(contains_code(&ending, "cluster.non-name-final-consonant"));

    assert!(contains_code(
        &analyze("ia", WordKind::GismuBasedLujvo),
        "analysis.restricted-on-glide"
    ));
    assert!(!contains_code(
        &analyze("ai", WordKind::GismuBasedLujvo),
        "analysis.restricted-on-glide"
    ));
    let bf = inspect_cluster("bf", ClusterPosition::Medial, WordKind::Cmevla).unwrap();
    assert!(bf.violations().iter().any(|reason| {
        reason.code() == "cluster.voicing-mismatch" && reason.source_sections() == ["3.6"]
    }));

    let across_comma = analyze("ab,fi", WordKind::Brivla);
    assert!(contains_code(&across_comma, "cluster.voicing-mismatch"));
    assert!(
        across_comma
            .clusters()
            .iter()
            .any(|cluster| cluster.inspection().cluster() == "bf"
                && cluster.original_spelling() == "b,f")
    );
}

#[test]
fn public_context_errors_are_send_sync_standard_errors() {
    fn assert_error_traits<T: std::error::Error + Send + Sync>() {}
    assert_error_traits::<bauske_phonology::AnalysisError>();
    assert_error_traits::<bauske_phonology::ClusterError>();
}
