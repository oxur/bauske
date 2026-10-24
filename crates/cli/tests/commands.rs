//! Real-process checks for CLI output, validation, and process exit behavior.

use std::process::Command;

use bauske_phonology::{ClusterPosition, SOURCE_ID, WordKind, inspect_cluster};

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bauske"))
        .args(args)
        .output()
        .expect("binary runs")
}

#[test]
fn inventory_commands_emit_expected_source_ordered_populations() {
    let alphabet = run(&["inventory"]);
    assert!(alphabet.status.success());
    let output = String::from_utf8(alphabet.stdout).unwrap();
    assert_eq!(output.lines().count(), 27);
    assert!(output.starts_with(
        "symbol\tkind\trepresentative_ipa\tsource_sections\tvariant_policy\n'\tapostrophe\th\t"
    ));
    let consonants = run(&["inventory", "--kind", "consonant"]);
    assert!(consonants.status.success());
    assert_eq!(
        String::from_utf8(consonants.stdout)
            .unwrap()
            .lines()
            .count(),
        18
    );
    let vowels = run(&["inventory", "--kind", "vowel"]);
    assert_eq!(String::from_utf8(vowels.stdout).unwrap().lines().count(), 7);
    let diphthongs = run(&["inventory", "--kind", "diphthong"]);
    let diphthong_output = String::from_utf8(diphthongs.stdout).unwrap();
    assert_eq!(diphthong_output.lines().count(), 17);
    assert!(diphthong_output.starts_with("spelling\tipa\tglide\tdirection\tdistribution\tsource_sections\nai\taj\tpalatal\toff\tfreely-used\t3.4\n"));
}

#[test]
fn compare_command_explains_all_coordinates_and_metadata() {
    let output = run(&["compare", "p", "b"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("profile: representatives-v1"));
    assert!(stdout.contains("source: P01-S20261022@0195f4b5b1f182ce8e21381210f766b326fb987f"));
    assert!(stdout.contains("articulatory baseline; perceptual status unvalidated"));
    assert!(stdout.contains("total: 1"));
    assert!(stdout.contains("place\tbilabial\tbilabial\t1\t0"));
    assert!(stdout.contains("manner\tplosive\tplosive\t1\t0"));
    assert!(stdout.contains("voicing\tvoiceless\tvoiced\t1\t1"));
}

#[test]
fn invalid_comparisons_and_arguments_exit_two_without_numeric_result() {
    for args in [
        &["compare", "p", "a"][..],
        &["compare", "ai", "au"][..],
        &["compare", "h", "p"][..],
        &["inventory", "--kind", "not-a-kind"][..],
    ] {
        let output = run(args);
        assert_eq!(output.status.code(), Some(2), "args={args:?}");
        assert!(output.stdout.is_empty(), "args={args:?}");
        assert!(!String::from_utf8_lossy(&output.stderr).contains("total:"));
    }
}

#[test]
fn help_succeeds_and_lists_value_enum_choices() {
    let output = run(&["inventory", "--help"]);
    assert!(output.status.success());
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("alphabet"));
    assert!(help.contains("diphthong"));
    assert!(help.contains("bauske inventory"));
    let version = run(&["--version"]);
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8_lossy(&version.stdout).trim(),
        format!("bauske {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn analyze_command_emits_exact_nucleus_context_and_diagnostic_fields() {
    let output = run(&["analyze", "a,i", "--kind", "cmavo"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("policy\tstandard-onsets-v1\n"));
    assert!(stdout.contains("kind\tcmavo\n"));
    assert!(stdout.contains("result\tphonological inspection; morphology unverified\n"));
    assert!(stdout.contains("required_pause_before\ttrue\n"));
    assert!(stdout.contains("start\tend\tspelling\tkind\tstress_countable\n"));
    assert!(stdout.contains("0\t3\ta,i\tdiphthong:ai\ttrue\n"));
    assert!(stdout.contains("analysis.comma-inside-nucleus\t3.3\t1..2\t"));

    let name = run(&["analyze", ".DJOsefin.", "--kind", "cmevla"]);
    assert!(name.status.success());
    let name_stdout = String::from_utf8(name.stdout).unwrap();
    assert!(name_stdout.contains("canonical\tdjosefin\n"));
    assert!(name_stdout.contains("capital_positions\t1,2,3\n"));
}

#[test]
fn cluster_command_reports_source_tension_and_malformed_inputs_are_empty_on_stdout() {
    let output = run(&[
        "cluster",
        "nstr",
        "--position",
        "medial",
        "--kind",
        "brivla",
    ]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("cluster\tnstr\n"));
    assert!(stdout.contains("position\tmedial\n"));
    assert!(stdout.contains(&format!("source\t{SOURCE_ID}\n")));
    let expected_tension = inspect_cluster("nstr", ClusterPosition::Medial, WordKind::Brivla)
        .unwrap()
        .source_length_tension()
        .unwrap();
    assert!(stdout.contains(&format!(
        "diagnostic\t{}\t{}\twitness={}",
        expected_tension.code(),
        expected_tension.source_sections().join(";"),
        expected_tension.last_three_initial_triple().unwrap_or("none")
    )));
    assert!(!stdout.contains("valid word"));

    for args in [
        &["analyze", "ae", "--kind", "cmavo"][..],
        &["analyze", "i", "--kind", "not-a-kind"][..],
        &["analyze", "a,','i", "--kind", "cmavo"][..],
        &["analyze", "b,'a", "--kind", "cmavo"][..],
        &["cluster", "h", "--position", "initial", "--kind", "brivla"][..],
        &["cluster", "bf", "--position", "wrong", "--kind", "brivla"][..],
    ] {
        let bad = run(args);
        assert_eq!(bad.status.code(), Some(2), "args={args:?}");
        assert!(bad.stdout.is_empty(), "args={args:?}");
        assert!(!bad.stderr.is_empty(), "args={args:?}");
    }
}

#[test]
fn real_process_runs_the_remaining_source_context_examples() {
    let cases: &[(&[&str], &[&str])] = &[
        (
            &["analyze", ".meiin.", "--kind", "cmevla"],
            &["2\t3\te\tvowel:e\ttrue", "3\t5\tii\tdiphthong:ii\ttrue"],
        ),
        (
            &["analyze", ".i,ia,ii,ia,ion.", "--kind", "cmevla"],
            &[
                "1\t2\ti\tvowel:i",
                "3\t5\tia\tdiphthong:ia",
                "12\t14\tio\tdiphthong:io",
            ],
        ),
        (
            &["analyze", "dikyjvo", "--kind", "brivla"],
            &[
                "1\t2\ti\tvowel:i\ttrue",
                "3\t4\ty\tvowel:y\tfalse",
                "6\t7\to\tvowel:o\ttrue",
            ],
        ),
        (
            &["analyze", ".rl.", "--kind", "cmevla", "--syllabic-at", "2"],
            &["2\t3\tl\tsyllabic-consonant:l\tfalse"],
        ),
        (
            &["analyze", ".armstrong.", "--kind", "cmevla"],
            &["required_pause_before\ttrue", "required_pause_after\ttrue"],
        ),
        (
            &["analyze", "ia", "--kind", "gismu-lujvo"],
            &["analysis.restricted-on-glide\t3.4", "diphthong=ia"],
        ),
        (
            &["analyze", "ab,fi", "--kind", "brivla"],
            &["cluster.voicing-mismatch\t3.6", "spelling=b,f canonical=bf"],
        ),
    ];
    for (args, expected) in cases {
        let output = run(args);
        assert!(
            output.status.success(),
            "args={args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        for field in *expected {
            assert!(
                stdout.contains(field),
                "args={args:?}, missing {field:?}: {stdout}"
            );
        }
    }
    let unrestricted = run(&["analyze", "ai", "--kind", "gismu-lujvo"]);
    assert!(unrestricted.status.success());
    assert!(
        !String::from_utf8_lossy(&unrestricted.stdout).contains("analysis.restricted-on-glide")
    );
}

#[test]
fn apostrophe_comma_hints_preserve_two_nuclei_and_original_offsets() {
    let cases = [("a,'i", 3, "1"), ("a',i", 3, "2"), ("a,',i", 4, "1,3")];
    for (word, second_start, commas) in cases {
        let output = run(&["analyze", word, "--kind", "cmavo"]);
        assert!(
            output.status.success(),
            "{word}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("canonical\ta'i\n"), "{word}: {stdout}");
        assert!(
            stdout.contains(&format!("comma_positions\t{commas}\n")),
            "{word}: {stdout}"
        );
        assert!(
            stdout.contains("0\t1\ta\tvowel:a\ttrue\n"),
            "{word}: {stdout}"
        );
        assert!(
            stdout.contains(&format!(
                "{second_start}\t{}\ti\tvowel:i\ttrue\n",
                second_start + 1
            )),
            "{word}: {stdout}"
        );
        assert!(
            !stdout.contains("analysis.comma-inside-nucleus"),
            "{word}: {stdout}"
        );
    }
}

#[test]
fn contextual_help_lists_closed_value_sets_and_repeatable_annotations() {
    let analyze = run(&["analyze", "--help"]);
    assert!(analyze.status.success());
    let help = String::from_utf8_lossy(&analyze.stdout);
    for choice in ["brivla", "cmavo", "cmevla", "zihevla", "gismu-lujvo"] {
        assert!(help.contains(choice), "missing {choice}: {help}");
    }
    assert!(help.contains("syllabic-at"));
    let cluster = run(&["cluster", "--help"]);
    assert!(cluster.status.success());
    let help = String::from_utf8_lossy(&cluster.stdout);
    for choice in ["initial", "medial", "final"] {
        assert!(help.contains(choice), "missing {choice}: {help}");
    }
}

#[test]
fn identified_profiles_are_listed_and_selectable_without_changing_default() {
    let listing = run(&["profiles"]);
    assert!(listing.status.success());
    let stdout = String::from_utf8(listing.stdout).unwrap();
    assert!(stdout.contains("representatives-v1"));
    assert!(stdout.contains("close-mid-e-v1"));
    assert!(stdout.contains("unvalidated"));

    let default = run(&["compare", "e", "o"]);
    let alternate = run(&["compare", "e", "o", "--profile", "close-mid-e-v1"]);
    assert!(default.status.success());
    assert!(alternate.status.success());
    let default = String::from_utf8(default.stdout).unwrap();
    let alternate = String::from_utf8(alternate.stdout).unwrap();
    assert!(default.contains("profile: representatives-v1"));
    assert!(default.contains("total: 3"));
    assert!(alternate.contains("profile: close-mid-e-v1"));
    assert!(alternate.contains("ipa: [e] / [o]"));
    assert!(alternate.contains("total: 2"));
}

#[test]
fn prosody_cli_shows_syllables_stress_and_buffer_distinctions() {
    let output = run(&["prosody", "klama", "--kind", "brivla"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("syllabification_resolved\ttrue"));
    assert!(stdout.contains("syllable\t0\t3\tkla\tkla\t0\ta\ttrue"));
    assert!(stdout.contains("primary_nucleus\t0"));
    assert!(stdout.contains("written_nuclei\t2"));
    assert!(stdout.contains("realization_policy\tbuffer-plan-v1"));
    assert!(stdout.contains("realization\tunmeasured caller-annotated plan"));

    let buffered = run(&[
        "prosody",
        "dikyjvo",
        "--kind",
        "gismu-lujvo",
        "--buffer-after",
        "4",
    ]);
    assert!(buffered.status.success());
    let stdout = String::from_utf8(buffered.stdout).unwrap();
    assert!(stdout.contains("written_nuclei\t3"));
    assert!(stdout.contains("performed_nuclei\t4"));
    assert!(stdout.contains("countable_nuclei\t2"));
    assert!(stdout.contains("buffer\t4\t2\tweak\t3.8;3.9"));
    assert!(stdout.contains("realization_policy\tbuffer-plan-v1"));
}

#[test]
fn source_limited_grouping_is_successful_and_keeps_independent_plans() {
    let canonical = run(&["prosody", "sairgoi", "--kind", "zihevla"]);
    assert!(canonical.status.success());
    let canonical = String::from_utf8(canonical.stdout).unwrap();
    assert!(canonical.contains("syllabification_resolved\ttrue"));
    assert!(canonical.contains("syllable\t0\t4\tsair\tsair\t0\tai\ttrue"));
    assert!(canonical.contains("syllable\t4\t7\tgoi\tgoi\t1\toi\ttrue"));

    for (word, annotation, span) in [("sairgoi", "3", "3..4"), ("sai,rgoi", "4", "4..5")] {
        let selected = run(&[
            "prosody",
            word,
            "--kind",
            "zihevla",
            "--syllabic-at",
            annotation,
        ]);
        assert!(
            selected.status.success(),
            "{word}: {}",
            String::from_utf8_lossy(&selected.stderr)
        );
        let stdout = String::from_utf8(selected.stdout).unwrap();
        assert!(
            stdout.contains("syllabification_resolved\tfalse"),
            "{word}: {stdout}"
        );
        assert!(stdout.contains(&format!(
            "syllable_limitation\tsyllables.syllabic-consonant-without-preceding-consonant\t3.9\t{span}"
        )), "{word}: {stdout}");
        assert!(stdout.contains("primary_nucleus\t0"), "{word}: {stdout}");
        assert!(stdout.contains("stress\t1\tweak"), "{word}: {stdout}");
        assert!(stdout.contains("written_nuclei\t3"), "{word}: {stdout}");
        assert!(stdout.contains("countable_nuclei\t2"), "{word}: {stdout}");
    }

    let output = run(&[
        "prosody",
        ".armstrong.",
        "--kind",
        "cmevla",
        "--buffer-after",
        "9",
    ]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("syllabification_resolved\tfalse"));
    assert!(stdout.contains("syllable_limitation\tsyllables.name-cluster-underspecified"));
    assert!(stdout.contains("primary_nucleus\t0"));
    assert!(stdout.contains("buffer\t9"));
}

#[test]
fn real_process_checks_grouping_hints_source_findings_and_syllabic_shape() {
    let cases: &[(&[&str], &[&str])] = &[
        (
            &[
                "prosody",
                "gas,tro",
                "--kind",
                "brivla",
                "--syllables",
                "hints",
            ],
            &[
                "syllabification_resolved\ttrue",
                "syllable\t0\t4\tgas,\tgas",
            ],
        ),
        (
            &["prosody", "a',i", "--kind", "cmavo", "--syllables", "hints"],
            &[
                "syllabification_resolved\tfalse",
                "syllables.hint-conflicts-with-apostrophe",
            ],
        ),
        (
            &["prosody", "a,i", "--kind", "cmavo"],
            &["syllabification_resolved\ttrue", "syllable\t0\t3\ta,i\tai"],
        ),
        (
            &[
                "prosody",
                "ab,fa",
                "--kind",
                "brivla",
                "--syllables",
                "hints",
            ],
            &[
                "syllabification_resolved\tfalse",
                "syllables.source-rule-finding\t3.6",
            ],
        ),
        (
            &[
                "prosody",
                "an,strana",
                "--kind",
                "brivla",
                "--syllables",
                "hints",
            ],
            &[
                "syllabification_resolved\tfalse",
                "syllables.source-length-tension\t3.6;3.7",
            ],
        ),
        (
            &["prosody", ".aBDAN.", "--kind", "cmevla"],
            &[
                "syllabification_resolved\ttrue",
                "syllable\t1\t3\taB\tab",
                "syllable\t3\t6\tDAN\tdan",
            ],
        ),
        (
            &[
                "prosody",
                "acmlana",
                "--kind",
                "zihevla",
                "--syllabic-at",
                "3",
            ],
            &[
                "syllabification_resolved\ttrue",
                "syllable\t0\t2\tac\tac",
                "syllable\t2\t4\tml\tml",
            ],
        ),
        (
            &[
                "prosody",
                "cmlana",
                "--kind",
                "zihevla",
                "--syllabic-at",
                "2",
            ],
            &[
                "syllabification_resolved\tfalse",
                "syllables.syllabic-consonant-unsupported-shape\t3.9",
            ],
        ),
    ];
    for (args, expected) in cases {
        let output = run(args);
        assert!(
            output.status.success(),
            "args={args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        for field in *expected {
            assert!(
                stdout.contains(field),
                "args={args:?}, missing {field:?}: {stdout}"
            );
        }
    }
}

#[test]
fn prosody_hard_errors_are_reported_without_partial_stdout() {
    for args in [
        vec!["prosody", "klama", "--kind", "brivla", "--primary", "99"],
        vec![
            "prosody",
            "e'u",
            "--kind",
            "cmavo",
            "--primary",
            "1",
            "--following",
            "brivla-unpaused",
        ],
        vec![
            "prosody",
            "klama",
            "--kind",
            "brivla",
            "--buffer-after",
            "1",
        ],
        vec!["prosody", "klama", "--kind", "brivla", "--unstressed"],
        vec![
            "prosody",
            "klama",
            "--kind",
            "brivla",
            "--primary",
            "0",
            "--unstressed",
        ],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2), "args: {args:?}");
        assert!(output.stdout.is_empty(), "args: {args:?}");
        assert!(!output.stderr.is_empty(), "args: {args:?}");
    }
}
