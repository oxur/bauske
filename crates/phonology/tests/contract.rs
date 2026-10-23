//! Integration checks for the public phonology registry and comparison contract.

use std::num::NonZeroU16;

use bauske_phonology::{
    AlphabetSymbol, Backness, CompareError, ConsonantWeights, DistanceModel, FeatureDimension,
    Features, GlideDirection, Height, Manner, OpenPolicy, PROFILE_ID, ParseSegmentError, Place,
    Rounding, SOURCE_ID, Segment, SegmentClass, SymbolKind, Voicing, VowelWeights, alphabet,
    diphthongs, open_policies,
};

fn segment(symbol: &str) -> Segment {
    symbol.parse().expect("test symbol is valid")
}
fn nz(value: u16) -> NonZeroU16 {
    NonZeroU16::new(value).expect("test weight is positive")
}
fn weighted(a: u16, b: u16, c: u16) -> DistanceModel {
    DistanceModel::new(
        ConsonantWeights::new(nz(a), nz(b), nz(c)),
        VowelWeights::new(nz(a), nz(b), nz(c)),
    )
}

#[test]
fn alphabet_registry_has_exact_source_order_and_population() {
    let expected = "',.abcdefgijklmnoprstuvxyz";
    let actual: String = alphabet().iter().map(|record| record.symbol()).collect();
    assert_eq!(actual, expected);
    assert_eq!(alphabet().len(), 26);
    assert_eq!(
        alphabet()
            .iter()
            .filter(|r| r.kind() == SymbolKind::Consonant)
            .count(),
        17
    );
    assert_eq!(
        alphabet()
            .iter()
            .filter(|r| r.kind() == SymbolKind::Vowel)
            .count(),
        6
    );
    assert_eq!(
        alphabet()
            .iter()
            .filter(|r| r.kind() == SymbolKind::Apostrophe)
            .count(),
        1
    );
    assert_eq!(
        alphabet()
            .iter()
            .filter(|r| r.kind() == SymbolKind::Pause)
            .count(),
        1
    );
    assert_eq!(
        alphabet()
            .iter()
            .filter(|r| r.kind() == SymbolKind::SyllableSeparator)
            .count(),
        1
    );
    assert_eq!(
        alphabet()
            .iter()
            .filter(|r| r
                .features()
                .is_some_and(|f| f.class() == SegmentClass::Consonant))
            .count(),
        18
    );
    assert_eq!(
        alphabet()
            .iter()
            .filter(|r| r
                .features()
                .is_some_and(|f| f.class() == SegmentClass::Vowel))
            .count(),
        6
    );
}

#[test]
fn source_alternatives_and_open_policies_are_preserved() {
    for (symbol, alternatives) in [
        ("'", &["h"][..]),
        (",", &["."][..]),
        (".", &["ʔ"][..]),
        ("a", &["a", "ɑ"][..]),
        ("b", &["b"][..]),
        ("c", &["ʃ", "ʂ"][..]),
        ("d", &["d"][..]),
        ("e", &["ɛ", "e"][..]),
        ("f", &["f", "ɸ"][..]),
        ("g", &["ɡ"][..]),
        ("i", &["i"][..]),
        ("j", &["ʒ", "ʐ"][..]),
        ("k", &["k"][..]),
        ("l", &["l", "l̩"][..]),
        ("m", &["m", "m̩"][..]),
        ("n", &["n", "n̩", "ŋ", "ŋ̍"][..]),
        ("o", &["o", "ɔ"][..]),
        ("p", &["p"][..]),
        ("r", &["r", "ɹ", "ɾ", "ʀ", "r̩", "ɹ̩", "ʀ̩"][..]),
        ("s", &["s"][..]),
        ("t", &["t"][..]),
        ("u", &["u"][..]),
        ("v", &["v", "β"][..]),
        ("x", &["x"][..]),
        ("y", &["ə"][..]),
        ("z", &["z"][..]),
    ] {
        assert_eq!(
            symbol
                .parse::<AlphabetSymbol>()
                .unwrap()
                .record()
                .alternatives(),
            alternatives,
            "{symbol}"
        );
    }
    let consonants: Vec<_> = alphabet()
        .iter()
        .filter(|record| record.kind() == SymbolKind::Consonant)
        .collect();
    assert_eq!(consonants.len(), 17);
    for record in consonants {
        assert!(
            record.policies().contains(&OpenPolicy::AspirationMayVary),
            "{} must expose the general aspiration policy",
            record.symbol()
        );
    }

    let r = "r".parse::<AlphabetSymbol>().unwrap();
    let policies = r.record().policies();
    assert!(policies.contains(&OpenPolicy::AspirationMayVary));
    assert!(policies.contains(&bauske_phonology::OpenPolicy::RhoticAlternativesEquallyAccepted));
    assert!(policies.contains(&bauske_phonology::OpenPolicy::SonorantsMayBeUnvoiced));
    assert!(
        policies.contains(&bauske_phonology::OpenPolicy::SyllabicSonorantsHaveContextualVariants)
    );
    assert!(segment("'").features().class() == SegmentClass::Consonant);
    assert!(open_policies().contains(&bauske_phonology::OpenPolicy::BufferVowelIsOpenEnded));
    assert!(
        open_policies().contains(&bauske_phonology::OpenPolicy::DiphthongVariantsAreNotEnumerated)
    );
    for symbol in ["a", "e", "i", "o", "u", "y"] {
        assert!(
            symbol
                .parse::<AlphabetSymbol>()
                .unwrap()
                .record()
                .policies()
                .contains(&bauske_phonology::OpenPolicy::VowelRoundingMayVary)
        );
    }
}

#[test]
fn open_policy_source_sections_are_precise_and_public() {
    for (policy, expected) in [
        (OpenPolicy::RhoticAlternativesEquallyAccepted, &["3.2"][..]),
        (OpenPolicy::VowelRoundingMayVary, &["3.2"][..]),
        (OpenPolicy::AspirationMayVary, &["3.2"][..]),
        (OpenPolicy::SonorantsMayBeUnvoiced, &["3.6"][..]),
        (OpenPolicy::ApostropheSubstituteIsOpenEnded, &["3.3"][..]),
        (OpenPolicy::BufferVowelIsOpenEnded, &["3.8"][..]),
        (OpenPolicy::DiphthongVariantsAreNotEnumerated, &["3.4"][..]),
        (
            OpenPolicy::SyllabicSonorantsHaveContextualVariants,
            &["3.4", "3.9"][..],
        ),
    ] {
        assert_eq!(policy.source_sections(), expected, "{policy:?}");
    }

    for record in alphabet()
        .iter()
        .filter(|record| matches!(record.symbol(), "l" | "m" | "n" | "r"))
    {
        assert_eq!(
            record.source_sections(),
            &["3.1", "3.2", "3.4", "3.6", "3.9"],
            "{}",
            record.symbol()
        );
    }
}

#[test]
fn source_diphthong_inventory_is_complete_and_classified() {
    let expected = [
        "ai", "ei", "oi", "au", "ia", "ie", "ii", "io", "iu", "ua", "ue", "ui", "uo", "uu", "iy",
        "uy",
    ];
    assert_eq!(diphthongs().len(), 16);
    assert_eq!(
        diphthongs()
            .iter()
            .map(|r| r.spelling())
            .collect::<Vec<_>>(),
        expected
    );
    assert!(
        diphthongs()[..4]
            .iter()
            .all(|r| r.direction() == GlideDirection::Off)
    );
    assert!(
        diphthongs()[4..]
            .iter()
            .all(|r| r.direction() == GlideDirection::On)
    );
    let expected_ipa = [
        "aj", "ɛj", "oj", "aw", "ja", "jɛ", "ji", "jo", "ju", "wa", "wɛ", "wi", "wo", "wu", "jə",
        "wə",
    ];
    assert_eq!(
        diphthongs().iter().map(|r| r.ipa()).collect::<Vec<_>>(),
        expected_ipa
    );
}

#[test]
fn source_and_profile_metadata_and_feature_witnesses_are_explicit() {
    let model = DistanceModel::default();
    assert_eq!(model.profile_id(), PROFILE_ID);
    assert_eq!(model.source_id(), SOURCE_ID);
    assert!(matches!(
        segment("r").features(),
        Features::Consonant {
            place: Place::Alveolar,
            manner: Manner::Trill,
            voicing: Voicing::Voiced
        }
    ));
    assert!(matches!(
        segment("y").features(),
        Features::Vowel {
            height: Height::Mid,
            backness: Backness::Central,
            rounding: Rounding::Unrounded
        }
    ));
    assert_eq!(
        model.compare(segment("p"), segment("p")).unwrap().total(),
        0
    );
    assert_eq!(
        model.compare(segment("p"), segment("b")).unwrap().total(),
        1
    );
    assert_eq!(
        model.compare(segment("p"), segment("t")).unwrap().total(),
        1
    );
    let px = model.compare(segment("p"), segment("x")).unwrap();
    assert_eq!(px.total(), 2);
    assert_eq!(
        px.contributions()
            .iter()
            .map(|c| c.dimension())
            .collect::<Vec<_>>(),
        [
            FeatureDimension::Place,
            FeatureDimension::Manner,
            FeatureDimension::Voicing
        ]
    );
    assert_eq!(
        px.contributions()
            .iter()
            .map(|c| c.cost())
            .collect::<Vec<_>>(),
        [1, 1, 0]
    );
    assert_eq!(
        model.compare(segment("i"), segment("u")).unwrap().total(),
        2
    );
    assert_eq!(
        model.compare(segment("i"), segment("e")).unwrap().total(),
        1
    );
    assert_eq!(
        model.compare(segment("e"), segment("o")).unwrap().total(),
        3
    );
    assert_eq!(
        model.compare(segment("'"), segment("x")).unwrap().total(),
        1
    );
}

#[test]
fn every_selected_representative_matches_the_foundation_feature_table() {
    use Backness::{Back, Central, Front};
    use Features::{Consonant as C, Vowel as V};
    use Height::{Close, CloseMid, Mid, Open, OpenMid};
    use Manner::{Fricative, LateralApproximant, Nasal, Plosive, Trill};
    use Place::{Alveolar, Bilabial, Glottal, Labiodental, Postalveolar, Velar};
    use Rounding::{Rounded, Unrounded};
    use Voicing::{Voiced, Voiceless};

    let expected = [
        (
            "'",
            C {
                place: Glottal,
                manner: Fricative,
                voicing: Voiceless,
            },
        ),
        (
            "b",
            C {
                place: Bilabial,
                manner: Plosive,
                voicing: Voiced,
            },
        ),
        (
            "c",
            C {
                place: Postalveolar,
                manner: Fricative,
                voicing: Voiceless,
            },
        ),
        (
            "d",
            C {
                place: Alveolar,
                manner: Plosive,
                voicing: Voiced,
            },
        ),
        (
            "f",
            C {
                place: Labiodental,
                manner: Fricative,
                voicing: Voiceless,
            },
        ),
        (
            "g",
            C {
                place: Velar,
                manner: Plosive,
                voicing: Voiced,
            },
        ),
        (
            "j",
            C {
                place: Postalveolar,
                manner: Fricative,
                voicing: Voiced,
            },
        ),
        (
            "k",
            C {
                place: Velar,
                manner: Plosive,
                voicing: Voiceless,
            },
        ),
        (
            "l",
            C {
                place: Alveolar,
                manner: LateralApproximant,
                voicing: Voiced,
            },
        ),
        (
            "m",
            C {
                place: Bilabial,
                manner: Nasal,
                voicing: Voiced,
            },
        ),
        (
            "n",
            C {
                place: Alveolar,
                manner: Nasal,
                voicing: Voiced,
            },
        ),
        (
            "p",
            C {
                place: Bilabial,
                manner: Plosive,
                voicing: Voiceless,
            },
        ),
        (
            "r",
            C {
                place: Alveolar,
                manner: Trill,
                voicing: Voiced,
            },
        ),
        (
            "s",
            C {
                place: Alveolar,
                manner: Fricative,
                voicing: Voiceless,
            },
        ),
        (
            "t",
            C {
                place: Alveolar,
                manner: Plosive,
                voicing: Voiceless,
            },
        ),
        (
            "v",
            C {
                place: Labiodental,
                manner: Fricative,
                voicing: Voiced,
            },
        ),
        (
            "x",
            C {
                place: Velar,
                manner: Fricative,
                voicing: Voiceless,
            },
        ),
        (
            "z",
            C {
                place: Alveolar,
                manner: Fricative,
                voicing: Voiced,
            },
        ),
        (
            "a",
            V {
                height: Open,
                backness: Front,
                rounding: Unrounded,
            },
        ),
        (
            "e",
            V {
                height: OpenMid,
                backness: Front,
                rounding: Unrounded,
            },
        ),
        (
            "i",
            V {
                height: Close,
                backness: Front,
                rounding: Unrounded,
            },
        ),
        (
            "o",
            V {
                height: CloseMid,
                backness: Back,
                rounding: Rounded,
            },
        ),
        (
            "u",
            V {
                height: Close,
                backness: Back,
                rounding: Rounded,
            },
        ),
        (
            "y",
            V {
                height: Mid,
                backness: Central,
                rounding: Unrounded,
            },
        ),
    ];
    for (symbol, features) in expected {
        assert_eq!(segment(symbol).features(), features, "{symbol}");
    }
}

#[test]
fn configurable_positive_weights_affect_all_dimensions_without_overflow() {
    let model = weighted(2, 3, 5);
    assert_eq!(
        model.compare(segment("p"), segment("b")).unwrap().total(),
        5
    );
    assert_eq!(
        model.compare(segment("p"), segment("t")).unwrap().total(),
        2
    );
    assert_eq!(
        model.compare(segment("p"), segment("x")).unwrap().total(),
        5
    );
    let max = weighted(u16::MAX, u16::MAX, u16::MAX);
    let result = max.compare(segment("e"), segment("o")).unwrap();
    assert_eq!(result.total(), 196_605);
    assert_eq!(
        result.contributions().iter().map(|c| c.cost()).sum::<u32>(),
        result.total()
    );
    assert!(
        result
            .contributions()
            .iter()
            .all(|c| c.weight().get() == u16::MAX)
    );
}

#[test]
fn input_parsing_is_exact_and_boundaries_remain_in_alphabet_only() {
    for (input, expected) in [
        ("", ParseSegmentError::Empty),
        ("ai", ParseSegmentError::MultipleSymbols),
        ("word", ParseSegmentError::MultipleSymbols),
        ("h", ParseSegmentError::UnknownSymbol),
        ("P", ParseSegmentError::UnknownSymbol),
        ("’", ParseSegmentError::UnknownSymbol),
        (".", ParseSegmentError::Boundary(SymbolKind::Pause)),
        (
            ",",
            ParseSegmentError::Boundary(SymbolKind::SyllableSeparator),
        ),
    ] {
        assert_eq!(input.parse::<Segment>().unwrap_err(), expected, "{input:?}");
    }
    for input in ["'", "a", "p"] {
        assert!(input.parse::<Segment>().is_ok());
    }
    for input in ["'", ",", "."] {
        assert!(input.parse::<AlphabetSymbol>().is_ok());
    }
    assert!(matches!(
        DistanceModel::default().compare(segment("p"), segment("a")),
        Err(CompareError::DifferentClasses { .. })
    ));
    assert!("ai".parse::<Segment>().is_err());
}

#[test]
fn every_subspace_has_injective_vectors_and_exhaustive_metric_properties() {
    let consonants: Vec<_> = alphabet()
        .iter()
        .filter(|r| {
            r.features()
                .is_some_and(|f| f.class() == SegmentClass::Consonant)
        })
        .map(|r| segment(r.symbol()))
        .collect();
    let vowels: Vec<_> = alphabet()
        .iter()
        .filter(|r| {
            r.features()
                .is_some_and(|f| f.class() == SegmentClass::Vowel)
        })
        .map(|r| segment(r.symbol()))
        .collect();
    assert_eq!(consonants.len(), 18);
    assert_eq!(vowels.len(), 6);
    for group in [&consonants, &vowels] {
        for i in 0..group.len() {
            for j in i + 1..group.len() {
                assert_ne!(
                    group[i].features(),
                    group[j].features(),
                    "{} and {} share an encoding",
                    group[i].as_str(),
                    group[j].as_str()
                );
            }
        }
    }
    for model in [DistanceModel::default(), weighted(2, 3, 5)] {
        let mut pairs = 0;
        let mut triples = 0;
        for group in [&consonants, &vowels] {
            for &a in group {
                for &b in group {
                    let explanation = model.compare(a, b).unwrap();
                    let ab = explanation.total();
                    assert_eq!(ab == 0, a == b, "{} / {}", a.as_str(), b.as_str());
                    let expected_dimensions = match a.class() {
                        SegmentClass::Consonant => [
                            FeatureDimension::Place,
                            FeatureDimension::Manner,
                            FeatureDimension::Voicing,
                        ],
                        SegmentClass::Vowel => [
                            FeatureDimension::Height,
                            FeatureDimension::Backness,
                            FeatureDimension::Rounding,
                        ],
                    };
                    assert_eq!(explanation.contributions().len(), 3);
                    for (contribution, expected) in
                        explanation.contributions().iter().zip(expected_dimensions)
                    {
                        assert_eq!(contribution.dimension(), expected);
                    }
                    assert_eq!(
                        explanation
                            .contributions()
                            .iter()
                            .map(|item| item.cost())
                            .sum::<u32>(),
                        ab
                    );
                    let ba = model.compare(b, a).unwrap().total();
                    assert_eq!(ab, ba);
                    pairs += 1;
                    for &c in group {
                        let bc = model.compare(b, c).unwrap().total();
                        let ac = model.compare(a, c).unwrap().total();
                        assert!(
                            ac <= ab + bc,
                            "{} -> {} -> {}",
                            a.as_str(),
                            b.as_str(),
                            c.as_str()
                        );
                        triples += 1;
                    }
                }
            }
        }
        assert_eq!(pairs, 360);
        assert_eq!(triples, 6_048);
    }
}

#[test]
fn vowel_coordinates_are_typed_and_ordered() {
    let io = DistanceModel::default()
        .compare(segment("i"), segment("o"))
        .unwrap();
    assert!(matches!(
        segment("i").features(),
        Features::Vowel {
            height: Height::Close,
            backness: Backness::Front,
            rounding: Rounding::Unrounded
        }
    ));
    assert_eq!(
        io.contributions()
            .iter()
            .map(|c| c.dimension())
            .collect::<Vec<_>>(),
        [
            FeatureDimension::Height,
            FeatureDimension::Backness,
            FeatureDimension::Rounding
        ]
    );
}
