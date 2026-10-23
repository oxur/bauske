//! Regression tests for bounded syllable, stress, and buffer planning.

use bauske_phonology::{
    FollowingContext, RealizationError, StressChoice, StressError, StressLevel,
    SyllabificationMode, WordKind, analyze_word, assign_stress, plan_realization, syllabify,
};

fn syllables(word: &str, kind: WordKind) -> Vec<String> {
    let analysis = analyze_word(word, kind, &[]).unwrap();
    let result = syllabify(&analysis, SyllabificationMode::Canonical);
    assert!(result.is_resolved(), "{word}: {:?}", result.limitations());
    result
        .syllables()
        .iter()
        .map(|item| item.canonical_spelling().to_owned())
        .collect()
}

#[test]
fn canonical_source_examples_group_with_original_spans() {
    assert_eq!(syllables("ninmu", WordKind::Brivla), ["nin", "mu"]);
    assert_eq!(syllables("gastro", WordKind::Brivla), ["ga", "stro"]);
    assert_eq!(syllables("fitpri", WordKind::Brivla), ["fit", "pri"]);
    assert_eq!(syllables("klezba", WordKind::Brivla), ["kle", "zba"]);
    assert_eq!(syllables("a'i", WordKind::Cmavo), ["a", "'i"]);
    assert_eq!(syllables(".meiin.", WordKind::Cmevla), ["me", "iin"]);

    let analysis = analyze_word("gas,tro", WordKind::Brivla, &[]).unwrap();
    let hinted = syllabify(&analysis, SyllabificationMode::WrittenHints);
    assert!(hinted.is_resolved(), "{:?}", hinted.limitations());
    assert_eq!(
        hinted
            .syllables()
            .iter()
            .map(|item| item.canonical_spelling())
            .collect::<Vec<_>>(),
        ["gas", "tro"]
    );
    assert_eq!(hinted.syllables()[0].original_spelling(), "gas,");
    assert_eq!(hinted.syllables()[0].start(), 0);
    assert_eq!(hinted.syllables()[0].end(), 4);
}

#[test]
fn source_limited_name_grouping_is_empty_but_stress_remains_available() {
    let analysis = analyze_word(".armstrong.", WordKind::Cmevla, &[]).unwrap();
    let grouped = syllabify(&analysis, SyllabificationMode::Canonical);
    assert!(!grouped.is_resolved());
    assert!(grouped.syllables().is_empty());
    assert!(
        grouped
            .limitations()
            .iter()
            .any(|item| item.code() == "syllables.name-cluster-underspecified")
    );
    assert_eq!(
        assign_stress(
            &analysis,
            StressChoice::Default,
            FollowingContext::Unspecified
        )
        .unwrap()
        .primary_nucleus(),
        Some(0)
    );
}

#[test]
fn selected_syllabic_nuclei_are_retained_when_grouping_is_source_limited() {
    let analysis = analyze_word("sairgoi", WordKind::Zihevla, &[3]).unwrap();
    assert_eq!(analysis.nuclei().len(), 3);
    assert_eq!(
        analysis
            .nuclei()
            .iter()
            .map(|item| item.stress_countable())
            .collect::<Vec<_>>(),
        [true, false, true]
    );
    let result = syllabify(&analysis, SyllabificationMode::Canonical);
    assert!(!result.is_resolved());
    assert!(result.syllables().is_empty());
    assert_eq!(
        result.limitations()[0].code(),
        "syllables.syllabic-consonant-without-preceding-consonant"
    );
    assert_eq!(result.limitations()[0].source_sections(), ["3.9"]);
    assert_eq!(result.limitations()[0].span(), 3..4);
    let stress = assign_stress(
        &analysis,
        StressChoice::Default,
        FollowingContext::Unspecified,
    )
    .unwrap();
    assert_eq!(stress.primary_nucleus(), Some(0));
    assert_eq!(
        stress.levels(),
        [
            StressLevel::Primary,
            StressLevel::Weak,
            StressLevel::OptionalSecondary
        ]
    );

    let comma_variant = analyze_word("sai,rgoi", WordKind::Zihevla, &[4]).unwrap();
    assert_eq!(comma_variant.nuclei().len(), 3);
    assert_eq!(
        comma_variant
            .nuclei()
            .iter()
            .map(|item| item.stress_countable())
            .collect::<Vec<_>>(),
        [true, false, true]
    );
    let comma_result = syllabify(&comma_variant, SyllabificationMode::Canonical);
    assert!(!comma_result.is_resolved());
    assert_eq!(comma_result.syllables().len(), 0);
    assert_eq!(
        comma_result.limitations()[0].code(),
        result.limitations()[0].code()
    );
    assert_eq!(comma_result.limitations()[0].span(), 4..5);
    let comma_stress = assign_stress(
        &comma_variant,
        StressChoice::Default,
        FollowingContext::Unspecified,
    )
    .unwrap();
    assert_eq!(comma_stress.primary_nucleus(), stress.primary_nucleus());
    assert_eq!(comma_stress.levels(), stress.levels());

    let supported = analyze_word(".rl.", WordKind::Cmevla, &[2]).unwrap();
    let supported_result = syllabify(&supported, SyllabificationMode::Canonical);
    assert!(
        supported_result.is_resolved(),
        "{:?}",
        supported_result.limitations()
    );
    assert_eq!(supported_result.syllables()[0].canonical_spelling(), "rl");
}

#[test]
fn selected_syllabic_groupings_obey_the_two_consonant_shape() {
    let analysis = analyze_word("acmlana", WordKind::Zihevla, &[3]).unwrap();
    let result = syllabify(&analysis, SyllabificationMode::Canonical);
    assert!(result.is_resolved(), "{:?}", result.limitations());
    assert_eq!(
        result
            .syllables()
            .iter()
            .map(|syllable| syllable.canonical_spelling())
            .collect::<Vec<_>>(),
        ["ac", "ml", "a", "na"]
    );
    assert_eq!(
        result
            .syllables()
            .iter()
            .map(|syllable| (syllable.start(), syllable.end()))
            .collect::<Vec<_>>(),
        [(0, 2), (2, 4), (4, 5), (5, 7)]
    );

    let initial = analyze_word("cmlana", WordKind::Zihevla, &[2]).unwrap();
    let unresolved = syllabify(&initial, SyllabificationMode::Canonical);
    assert!(!unresolved.is_resolved());
    assert!(unresolved.syllables().is_empty());
    assert_eq!(
        unresolved.limitations()[0].code(),
        "syllables.syllabic-consonant-unsupported-shape"
    );
    assert_eq!(unresolved.limitations()[0].source_sections(), ["3.9"]);
    assert_eq!(initial.nuclei().len(), 3);
    assert_eq!(
        initial
            .nuclei()
            .iter()
            .map(|nucleus| nucleus.stress_countable())
            .collect::<Vec<_>>(),
        [false, true, true]
    );
    assert_eq!(
        assign_stress(
            &initial,
            StressChoice::Default,
            FollowingContext::Unspecified
        )
        .unwrap()
        .primary_nucleus(),
        Some(1)
    );

    let hinted = analyze_word("ac,mlana", WordKind::Zihevla, &[4]).unwrap();
    let hinted_result = syllabify(&hinted, SyllabificationMode::WrittenHints);
    assert!(
        hinted_result.is_resolved(),
        "{:?}",
        hinted_result.limitations()
    );
    assert_eq!(
        hinted_result
            .syllables()
            .iter()
            .map(|syllable| syllable.canonical_spelling())
            .collect::<Vec<_>>(),
        ["ac", "ml", "a", "na"]
    );
    assert_eq!(hinted_result.syllables()[0].original_spelling(), "ac,");
    assert_eq!(hinted_result.syllables()[1].start(), 3);
    assert_eq!(hinted_result.syllables()[1].end(), 5);

    let incompatible = analyze_word("a,cmlana", WordKind::Zihevla, &[4]).unwrap();
    let hinted_unresolved = syllabify(&incompatible, SyllabificationMode::WrittenHints);
    assert!(!hinted_unresolved.is_resolved());
    assert!(hinted_unresolved.syllables().is_empty());
    assert_eq!(
        hinted_unresolved.limitations()[0].code(),
        "syllables.syllabic-consonant-unsupported-shape"
    );
    let canonical = syllabify(&incompatible, SyllabificationMode::Canonical);
    assert!(canonical.is_resolved(), "{:?}", canonical.limitations());
    assert_eq!(
        canonical
            .syllables()
            .iter()
            .map(|syllable| syllable.canonical_spelling())
            .collect::<Vec<_>>(),
        ["ac", "ml", "a", "na"]
    );
}

#[test]
fn syllabic_grouping_normalizes_case_without_weakening_name_stress_marks() {
    let uppercase = analyze_word(".rL.", WordKind::Cmevla, &[2]).unwrap();
    for mode in [
        SyllabificationMode::Canonical,
        SyllabificationMode::WrittenHints,
    ] {
        let result = syllabify(&uppercase, mode);
        assert!(result.is_resolved(), "{:?}", result.limitations());
        assert_eq!(result.syllables().len(), 1);
        let syllable = &result.syllables()[0];
        assert_eq!((syllable.start(), syllable.end()), (1, 3));
        assert_eq!(syllable.original_spelling(), "rL");
        assert_eq!(syllable.canonical_spelling(), "rl");
    }

    let lowercase = analyze_word(".rl.", WordKind::Cmevla, &[2]).unwrap();
    let lowercase_stress = assign_stress(
        &lowercase,
        StressChoice::Default,
        FollowingContext::Unspecified,
    )
    .unwrap();
    assert_eq!(lowercase_stress.primary_nucleus(), None);
    assert_eq!(lowercase_stress.levels(), [StressLevel::Weak]);

    assert_eq!(
        assign_stress(
            &uppercase,
            StressChoice::Default,
            FollowingContext::Unspecified,
        )
        .unwrap_err(),
        StressError::NameStressOnWeakNucleus { index: 0 }
    );
}

#[test]
fn stress_plans_follow_counting_defaults_and_name_marks() {
    let klama = analyze_word("klama", WordKind::Brivla, &[]).unwrap();
    assert_eq!(
        assign_stress(&klama, StressChoice::Default, FollowingContext::Unspecified)
            .unwrap()
            .primary_nucleus(),
        Some(0)
    );
    let dikyjvo = analyze_word("dikyjvo", WordKind::GismuBasedLujvo, &[]).unwrap();
    let stress = assign_stress(
        &dikyjvo,
        StressChoice::Default,
        FollowingContext::Unspecified,
    )
    .unwrap();
    assert_eq!(stress.primary_nucleus(), Some(0));
    assert_eq!(
        stress.levels(),
        [
            StressLevel::Primary,
            StressLevel::Weak,
            StressLevel::OptionalSecondary
        ]
    );
    let marked = analyze_word(".DJOsefin.", WordKind::Cmevla, &[]).unwrap();
    assert_eq!(
        assign_stress(
            &marked,
            StressChoice::Default,
            FollowingContext::Unspecified
        )
        .unwrap()
        .primary_nucleus(),
        Some(0)
    );
    let unmarked = analyze_word(".djosefin.", WordKind::Cmevla, &[]).unwrap();
    assert_eq!(
        assign_stress(
            &unmarked,
            StressChoice::Default,
            FollowingContext::Unspecified
        )
        .unwrap()
        .primary_nucleus(),
        Some(1)
    );
}

#[test]
fn cmavo_pause_guard_is_contextual_and_visible_when_unchecked() {
    let analysis = analyze_word("e'u", WordKind::Cmavo, &[]).unwrap();
    assert_eq!(
        assign_stress(
            &analysis,
            StressChoice::Default,
            FollowingContext::Unspecified
        )
        .unwrap()
        .primary_nucleus(),
        None
    );
    assert_eq!(
        assign_stress(
            &analysis,
            StressChoice::Nucleus(1),
            FollowingContext::BrivlaUnpaused
        )
        .unwrap_err(),
        StressError::PauseRequiredBeforeBrivla
    );
    assert_eq!(
        assign_stress(
            &analysis,
            StressChoice::Nucleus(1),
            FollowingContext::BrivlaAfterPause
        )
        .unwrap()
        .primary_nucleus(),
        Some(1)
    );
    assert!(
        assign_stress(
            &analysis,
            StressChoice::Nucleus(1),
            FollowingContext::Unspecified
        )
        .unwrap()
        .limitations()
        .contains(&"stress.following-brivla-unchecked")
    );

    // The last countable nucleus is not necessarily the last actual nucleus.
    // A trailing selected syllabic sonorant means nucleus 1 is not final.
    let trailing_weak = analyze_word("e'ul", WordKind::Cmavo, &[3]).unwrap();
    assert_eq!(trailing_weak.nuclei().len(), 3);
    assert_eq!(
        assign_stress(
            &trailing_weak,
            StressChoice::Nucleus(1),
            FollowingContext::BrivlaUnpaused,
        )
        .unwrap()
        .primary_nucleus(),
        Some(1)
    );
}

#[test]
fn stress_policy_handles_counting_marks_and_explicit_rejections() {
    for (word, kind, expected) in [
        ("bisydja", WordKind::Brivla, 0),
        ("da'udja", WordKind::Brivla, 1),
    ] {
        let analysis = analyze_word(word, kind, &[]).unwrap();
        assert_eq!(
            assign_stress(
                &analysis,
                StressChoice::Default,
                FollowingContext::Unspecified
            )
            .unwrap()
            .primary_nucleus(),
            Some(expected),
            "{word}"
        );
    }

    let all_weak = analyze_word("e'u", WordKind::Cmavo, &[]).unwrap();
    let unstressed = assign_stress(
        &all_weak,
        StressChoice::Unstressed,
        FollowingContext::Unspecified,
    )
    .unwrap();
    assert_eq!(unstressed.primary_nucleus(), None);
    assert!(
        unstressed
            .levels()
            .iter()
            .all(|level| *level == StressLevel::Weak)
    );
    assert_eq!(
        assign_stress(
            &all_weak,
            StressChoice::Nucleus(0),
            FollowingContext::BrivlaUnpaused,
        )
        .unwrap()
        .primary_nucleus(),
        Some(0),
        "a non-final cmavo nucleus may be stressed without a pause"
    );

    let syllabic = analyze_word(".rl.", WordKind::Cmevla, &[2]).unwrap();
    let weak_name = assign_stress(
        &syllabic,
        StressChoice::Default,
        FollowingContext::Unspecified,
    )
    .unwrap();
    assert_eq!(weak_name.primary_nucleus(), None);
    assert_eq!(weak_name.levels(), [StressLevel::Weak]);
    assert!(
        weak_name
            .limitations()
            .contains(&"stress.name-no-countable-nuclei")
    );

    let content = analyze_word("klama", WordKind::Brivla, &[]).unwrap();
    assert_eq!(
        assign_stress(
            &content,
            StressChoice::Nucleus(99),
            FollowingContext::Unspecified
        )
        .unwrap_err(),
        StressError::IneligibleNucleus { index: 99 }
    );
    assert_eq!(
        assign_stress(
            &content,
            StressChoice::Nucleus(1),
            FollowingContext::Unspecified
        )
        .unwrap_err(),
        StressError::RequiredPenultimate {
            expected: 0,
            found: 1
        }
    );
    assert_eq!(
        assign_stress(
            &content,
            StressChoice::Unstressed,
            FollowingContext::Unspecified
        )
        .unwrap_err(),
        StressError::ContentWordUnstressed
    );
    let too_few = analyze_word("byda", WordKind::Brivla, &[]).unwrap();
    assert_eq!(
        assign_stress(
            &too_few,
            StressChoice::Default,
            FollowingContext::Unspecified
        )
        .unwrap_err(),
        StressError::InsufficientCountableNuclei
    );
    let y = analyze_word("y'a", WordKind::Cmavo, &[]).unwrap();
    assert_eq!(
        assign_stress(&y, StressChoice::Nucleus(0), FollowingContext::Unspecified).unwrap_err(),
        StressError::IneligibleNucleus { index: 0 }
    );
    assert_eq!(
        assign_stress(
            &syllabic,
            StressChoice::Nucleus(0),
            FollowingContext::Unspecified
        )
        .unwrap_err(),
        StressError::IneligibleNucleus { index: 0 }
    );

    let multiple_marks = analyze_word(".dJOsEfin.", WordKind::Cmevla, &[]).unwrap();
    assert_eq!(
        assign_stress(
            &multiple_marks,
            StressChoice::Default,
            FollowingContext::Unspecified
        )
        .unwrap_err(),
        StressError::AmbiguousNameStress
    );
    let consonant_marks = analyze_word(".DJosefin.", WordKind::Cmevla, &[]).unwrap();
    assert_eq!(
        assign_stress(
            &consonant_marks,
            StressChoice::Default,
            FollowingContext::Unspecified
        )
        .unwrap_err(),
        StressError::NameCapitalWithoutVowel
    );
    let marked_weak = analyze_word(".rL.", WordKind::Cmevla, &[2]).unwrap();
    assert_eq!(
        assign_stress(
            &marked_weak,
            StressChoice::Default,
            FollowingContext::Unspecified
        )
        .unwrap_err(),
        StressError::NameStressOnWeakNucleus { index: 0 }
    );
    let unmarked = analyze_word(".djosefin.", WordKind::Cmevla, &[]).unwrap();
    assert_eq!(
        assign_stress(
            &unmarked,
            StressChoice::Nucleus(0),
            FollowingContext::Unspecified
        )
        .unwrap_err(),
        StressError::NameNotationRequired
    );
    assert_eq!(
        assign_stress(
            &unmarked,
            StressChoice::Unstressed,
            FollowingContext::Unspecified
        )
        .unwrap_err(),
        StressError::UnstressedNameUnsupported
    );
}

#[test]
fn public_stress_and_realization_errors_are_thread_safe_errors() {
    fn assert_error_traits<T: std::error::Error + Send + Sync>() {}
    assert_error_traits::<StressError>();
    assert_error_traits::<RealizationError>();
}

#[test]
fn buffer_plans_keep_written_and_countable_nuclei_stable() {
    let analysis = analyze_word("dikyjvo", WordKind::GismuBasedLujvo, &[]).unwrap();
    let plan = plan_realization(&analysis, &[4]).unwrap();
    assert_eq!(plan.written_nucleus_count(), 3);
    assert_eq!(plan.performed_nucleus_count(), 4);
    assert_eq!(plan.countable_nucleus_count(), 2);
    assert_eq!(plan.events()[0].after_position(), 4);
    assert_eq!(plan.events()[0].performed_nucleus_index(), 2);
    assert_eq!(plan.events()[0].level(), StressLevel::Weak);

    let conflict = analyze_word("abfa", WordKind::Cmavo, &[]).unwrap();
    let plan = plan_realization(&conflict, &[1]).unwrap();
    assert!(
        plan.diagnostics()
            .iter()
            .any(|item| item.code() == "cluster.voicing-mismatch")
    );
    assert_eq!(plan.countable_nucleus_count(), 2);
    assert!(matches!(
        plan_realization(&analysis, &[1]),
        Err(RealizationError::InvalidPosition { position: 1 })
    ));
    assert!(matches!(
        plan_realization(&analysis, &[4, 4]),
        Err(RealizationError::DuplicatePosition { position: 4 })
    ));
}

#[test]
fn buffer_indices_are_insertion_points_in_the_performed_sequence() {
    for (word, kind, positions, expected) in [
        ("klama", WordKind::Brivla, vec![0], vec![0]),
        ("dikyjvo", WordKind::GismuBasedLujvo, vec![4], vec![2]),
        (".armstrong.", WordKind::Cmevla, vec![9], vec![2]),
        ("klamprona", WordKind::Brivla, vec![0, 3, 4], vec![0, 2, 3]),
        (
            "kla,mpro,na",
            WordKind::Brivla,
            vec![0, 4, 5],
            vec![0, 2, 3],
        ),
    ] {
        let analysis = analyze_word(word, kind, &[]).unwrap();
        let plan = plan_realization(&analysis, &positions).unwrap();
        assert_eq!(
            plan.events()
                .iter()
                .map(|event| event.performed_nucleus_index())
                .collect::<Vec<_>>(),
            expected,
            "{word}"
        );
        assert_eq!(
            plan.events()
                .iter()
                .map(|event| event.after_position())
                .collect::<Vec<_>>(),
            positions,
            "{word}"
        );
        assert_eq!(
            plan.performed_nucleus_count(),
            plan.written_nucleus_count() + positions.len()
        );
    }
}

#[test]
fn buffer_annotations_reject_selected_nuclei_and_unsupported_locations() {
    let syllabic = analyze_word(".rl.", WordKind::Cmevla, &[2]).unwrap();
    assert_eq!(
        plan_realization(&syllabic, &[2]).unwrap_err(),
        RealizationError::SyllabicNucleusConflict { position: 2 }
    );
    let unsupported = analyze_word("klama", WordKind::Brivla, &[]).unwrap();
    assert_eq!(
        plan_realization(&unsupported, &[3]).unwrap_err(),
        RealizationError::UnsupportedLocation { position: 3 }
    );
    let apostrophe = analyze_word("akla'i", WordKind::Cmavo, &[]).unwrap();
    assert_eq!(
        plan_realization(&apostrophe, &[2]).unwrap_err(),
        RealizationError::UnsupportedLocation { position: 2 }
    );
}

#[test]
fn name_final_buffer_is_supported_while_name_grouping_stays_limited() {
    let analysis = analyze_word(".armstrong.", WordKind::Cmevla, &[]).unwrap();
    let plan = plan_realization(&analysis, &[9]).unwrap();
    assert_eq!(
        plan.performed_nucleus_count(),
        plan.written_nucleus_count() + 1
    );
    assert!(matches!(
        syllabify(&analysis, SyllabificationMode::Canonical).limitations()[0].code(),
        "syllables.name-cluster-underspecified"
    ));
}

#[test]
fn malformed_hint_does_not_return_partial_syllables() {
    let analysis = analyze_word("a,i,a", WordKind::Cmavo, &[]).unwrap();
    let result = syllabify(&analysis, SyllabificationMode::WrittenHints);
    assert!(!result.is_resolved());
    assert!(result.syllables().is_empty());
    assert!(
        result
            .limitations()
            .iter()
            .any(|item| item.code() == "syllables.hint-inside-nucleus")
    );
}

#[test]
fn comma_hints_are_neutral_to_canonical_nuclei_and_cannot_hide_clusters() {
    let canonical = analyze_word("a,i", WordKind::Cmavo, &[]).unwrap();
    let result = syllabify(&canonical, SyllabificationMode::Canonical);
    assert!(result.is_resolved(), "{:?}", result.limitations());
    assert_eq!(result.syllables().len(), 1);
    assert_eq!(result.syllables()[0].canonical_spelling(), "ai");
    assert_eq!(result.syllables()[0].start(), 0);
    assert_eq!(result.syllables()[0].end(), 3);
    let hinted = syllabify(&canonical, SyllabificationMode::WrittenHints);
    assert!(!hinted.is_resolved());
    assert_eq!(
        hinted.limitations()[0].code(),
        "syllables.hint-inside-nucleus"
    );

    for (word, expected_span) in [("abfa", 1..3), ("ab,fa", 1..4)] {
        let analysis = analyze_word(word, WordKind::Brivla, &[]).unwrap();
        for mode in [
            SyllabificationMode::Canonical,
            SyllabificationMode::WrittenHints,
        ] {
            let result = syllabify(&analysis, mode);
            assert!(!result.is_resolved(), "{word} {mode:?}");
            assert!(result.syllables().is_empty());
            let finding = result
                .limitations()
                .iter()
                .find(|item| item.code() == "syllables.source-rule-finding")
                .expect("mixed-voicing cluster must remain visible");
            assert_eq!(finding.span(), expected_span);
            assert_eq!(finding.source_sections(), ["3.6"]);
        }
    }

    for (word, expected_span) in [("anstrana", 1..5), ("an,strana", 1..6)] {
        let analysis = analyze_word(word, WordKind::Brivla, &[]).unwrap();
        let result = syllabify(&analysis, SyllabificationMode::WrittenHints);
        assert!(!result.is_resolved(), "{word}");
        let tension = result
            .limitations()
            .iter()
            .find(|item| {
                item.code() == "syllables.source-length-tension" && item.span() == expected_span
            })
            .expect("the public cluster inspection declares the nstr tension");
        assert_eq!(tension.source_sections(), ["3.6", "3.7"]);
    }

    for (word, expected_span) in [("kkala", 0..2), ("k,kala", 0..3)] {
        let analysis = analyze_word(word, WordKind::Brivla, &[]).unwrap();
        let result = syllabify(&analysis, SyllabificationMode::WrittenHints);
        assert!(!result.is_resolved(), "{word}");
        assert!(result.limitations().iter().any(|item| {
            item.code() == "syllables.source-rule-finding" && item.span() == expected_span
        }));
    }
    let final_finding = analyze_word("ban", WordKind::Brivla, &[]).unwrap();
    let final_result = syllabify(&final_finding, SyllabificationMode::Canonical);
    assert!(!final_result.is_resolved());
    assert!(
        final_result
            .limitations()
            .iter()
            .any(|item| item.code() == "syllables.source-rule-finding")
    );

    for (word, expected) in [("gas,tro", ["gas", "tro"]), ("klez,ba", ["klez", "ba"])] {
        let analysis = analyze_word(word, WordKind::Brivla, &[]).unwrap();
        let result = syllabify(&analysis, SyllabificationMode::WrittenHints);
        assert!(result.is_resolved(), "{word}: {:?}", result.limitations());
        assert_eq!(
            result
                .syllables()
                .iter()
                .map(|item| item.canonical_spelling())
                .collect::<Vec<_>>(),
            expected
        );
    }

    for (word, expected) in [("gastro", ["ga", "stro"]), ("klezba", ["kle", "zba"])] {
        assert_eq!(syllables(word, WordKind::Brivla), expected);
    }
}

#[test]
fn apostrophe_hint_placement_preserves_vowel_attachment() {
    let incompatible = analyze_word("a',i", WordKind::Cmavo, &[]).unwrap();
    let result = syllabify(&incompatible, SyllabificationMode::WrittenHints);
    assert!(!result.is_resolved());
    assert!(result.syllables().is_empty());
    assert_eq!(
        result.limitations()[0].code(),
        "syllables.hint-conflicts-with-apostrophe"
    );
    assert_eq!(result.limitations()[0].span(), 1..3);

    let compatible = analyze_word("a,'i", WordKind::Cmavo, &[]).unwrap();
    let result = syllabify(&compatible, SyllabificationMode::WrittenHints);
    assert!(result.is_resolved(), "{:?}", result.limitations());
    assert_eq!(
        result
            .syllables()
            .iter()
            .map(|item| item.canonical_spelling())
            .collect::<Vec<_>>(),
        ["a", "'i"]
    );
}

#[test]
fn uppercase_name_clusters_preserve_original_spans_and_long_names_are_named() {
    for word in [".aBDAN.", ".abdan."] {
        let analysis = analyze_word(word, WordKind::Cmevla, &[]).unwrap();
        let result = syllabify(&analysis, SyllabificationMode::Canonical);
        assert!(result.is_resolved(), "{word}: {:?}", result.limitations());
        assert_eq!(
            result
                .syllables()
                .iter()
                .map(|item| (item.canonical_spelling(), item.start(), item.end()))
                .collect::<Vec<_>>(),
            [("ab", 1, 3), ("dan", 3, 6)]
        );
    }
    let listed_uppercase = analyze_word(".aKLama.", WordKind::Cmevla, &[]).unwrap();
    let listed_uppercase_result = syllabify(&listed_uppercase, SyllabificationMode::Canonical);
    assert!(listed_uppercase_result.is_resolved());
    assert_eq!(
        listed_uppercase_result
            .syllables()
            .iter()
            .map(|item| item.canonical_spelling())
            .collect::<Vec<_>>(),
        ["a", "kla", "ma"]
    );
    let long_name = analyze_word(".anstraf.", WordKind::Cmevla, &[]).unwrap();
    let result = syllabify(&long_name, SyllabificationMode::Canonical);
    assert!(!result.is_resolved());
    assert!(result.limitations().iter().any(|item| {
        item.code() == "syllables.name-cluster-underspecified" && item.span() == (2..6)
    }));
    assert!(
        !result
            .limitations()
            .iter()
            .any(|item| item.code() == "syllables.source-length-tension")
    );
    for word in [".nstrala.", ".abnstr."] {
        let edge_name = analyze_word(word, WordKind::Cmevla, &[]).unwrap();
        let edge_result = syllabify(&edge_name, SyllabificationMode::Canonical);
        assert!(!edge_result.is_resolved(), "{word}");
        assert!(
            edge_result
                .limitations()
                .iter()
                .any(|item| item.code() == "syllables.name-cluster-underspecified")
        );
        assert!(
            !edge_result
                .limitations()
                .iter()
                .any(|item| item.code() == "syllables.source-length-tension")
        );
    }
}
