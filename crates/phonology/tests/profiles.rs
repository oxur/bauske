//! Exhaustive finite metric checks for identified pronunciation profiles.

use std::num::NonZeroU16;

use bauske_phonology::{
    Backness, ConsonantWeights, DistanceModel, FeatureDimension, FeatureValue, Features, Height,
    PronunciationProfile, Rounding, Segment, SegmentClass, VowelWeights, alphabet, profiles,
};

fn representatives(class: SegmentClass) -> Vec<Segment> {
    alphabet()
        .iter()
        .filter(|record| {
            record
                .features()
                .is_some_and(|feature| feature.class() == class)
        })
        .map(|record| record.symbol().parse().unwrap())
        .collect()
}

fn model(profile: PronunciationProfile, weights: [u16; 3]) -> DistanceModel {
    let [first, second, third] = weights.map(|weight| NonZeroU16::new(weight).unwrap());
    DistanceModel::new(
        ConsonantWeights::new(first, second, third),
        VowelWeights::new(first, second, third),
    )
    .with_profile(profile)
}

#[test]
fn registry_contains_exactly_the_two_immutable_profiles() {
    assert_eq!(
        profiles(),
        [
            PronunciationProfile::RepresentativesV1,
            PronunciationProfile::CloseMidEV1
        ]
    );
    assert_eq!(
        PronunciationProfile::RepresentativesV1.id(),
        "representatives-v1"
    );
    assert_eq!(PronunciationProfile::CloseMidEV1.id(), "close-mid-e-v1");
    let e: Segment = "e".parse().unwrap();
    let o: Segment = "o".parse().unwrap();
    assert_eq!(PronunciationProfile::RepresentativesV1.ipa(e), "[ɛ]");
    assert_eq!(PronunciationProfile::CloseMidEV1.ipa(e), "[e]");
    assert_eq!(
        PronunciationProfile::CloseMidEV1.features(e),
        Features::Vowel {
            height: Height::CloseMid,
            backness: Backness::Front,
            rounding: Rounding::Unrounded,
        }
    );
    assert_eq!(PronunciationProfile::CloseMidEV1.features(o), o.features());
    for record in alphabet() {
        let Ok(segment) = record.symbol().parse::<Segment>() else {
            continue;
        };
        if segment != e {
            assert_eq!(
                PronunciationProfile::CloseMidEV1.features(segment),
                PronunciationProfile::RepresentativesV1.features(segment),
                "{} features",
                record.symbol()
            );
            assert_eq!(
                PronunciationProfile::CloseMidEV1.ipa(segment),
                PronunciationProfile::RepresentativesV1.ipa(segment),
                "{} IPA",
                record.symbol()
            );
        }
    }
    let d_default = DistanceModel::default().compare(e, o).unwrap();
    let d_alt = DistanceModel::default()
        .with_profile(PronunciationProfile::CloseMidEV1)
        .compare(e, o)
        .unwrap();
    assert_eq!(d_default.total(), 3);
    assert_eq!(d_alt.total(), 2);
    assert_eq!(d_alt.contributions()[0].cost(), 0);
    assert_eq!(d_alt.contributions()[1].cost(), 1);
    assert_eq!(d_alt.contributions()[2].cost(), 1);
    assert_eq!(d_alt.profile_id(), "close-mid-e-v1");
    assert_eq!(d_alt.source_id(), d_default.source_id());
    assert_eq!(
        DistanceModel::default().profile(),
        PronunciationProfile::RepresentativesV1
    );
}

#[test]
fn each_profile_preserves_injective_finite_metric_subspaces_for_required_weights() {
    let consonants = representatives(SegmentClass::Consonant);
    let vowels = representatives(SegmentClass::Vowel);
    assert_eq!(consonants.len().pow(2) + vowels.len().pow(2), 360);
    assert_eq!(consonants.len().pow(3) + vowels.len().pow(3), 6048);
    for profile in profiles() {
        for weights in [[1, 1, 1], [2, 3, 5]] {
            let metric = model(*profile, weights);
            assert_eq!(metric.profile_id(), profile.id());
            for values in [&consonants, &vowels] {
                for &a in values {
                    for &b in values {
                        let ab = metric.compare(a, b).unwrap();
                        let ba = metric.compare(b, a).unwrap();
                        assert_eq!(ab.total(), ba.total());
                        assert_eq!(ab.profile_id(), profile.id());
                        assert_eq!(ab.source_id(), profile.source_id());
                        assert_eq!(ab.contributions().len(), 3);
                        let dimensions = match a.class() {
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
                        let expected_values = |segment| match profile.features(segment) {
                            Features::Consonant {
                                place,
                                manner,
                                voicing,
                            } => [
                                FeatureValue::Place(place),
                                FeatureValue::Manner(manner),
                                FeatureValue::Voicing(voicing),
                            ],
                            Features::Vowel {
                                height,
                                backness,
                                rounding,
                            } => [
                                FeatureValue::Height(height),
                                FeatureValue::Backness(backness),
                                FeatureValue::Rounding(rounding),
                            ],
                            _ => unreachable!("a segment feature class has exactly three axes"),
                        };
                        let left_features = expected_values(a);
                        let right_features = expected_values(b);
                        for (index, part) in ab.contributions().iter().enumerate() {
                            assert_eq!(part.dimension(), dimensions[index]);
                            assert_eq!(part.left(), left_features[index]);
                            assert_eq!(part.right(), right_features[index]);
                            let expected_weight = weights[index];
                            assert_eq!(part.weight().get(), expected_weight);
                            assert_eq!(
                                part.cost(),
                                if left_features[index] == right_features[index] {
                                    0
                                } else {
                                    u32::from(expected_weight)
                                }
                            );
                        }
                        assert_eq!(
                            ab.total(),
                            ab.contributions()
                                .iter()
                                .map(|part| part.cost())
                                .sum::<u32>()
                        );
                        assert_eq!(ab.total() == 0, a == b);
                        for &c in values {
                            let ac = metric.compare(a, c).unwrap().total();
                            let bc = metric.compare(b, c).unwrap().total();
                            assert!(ac <= ab.total() + bc);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn maximum_weights_and_cross_class_rejection_are_profile_independent() {
    let maximum = NonZeroU16::new(u16::MAX).unwrap();
    let metric = DistanceModel::new(
        ConsonantWeights::new(maximum, maximum, maximum),
        VowelWeights::new(maximum, maximum, maximum),
    );
    let e: Segment = "e".parse().unwrap();
    let o: Segment = "o".parse().unwrap();
    let i: Segment = "i".parse().unwrap();
    assert_eq!(metric.compare(e, o).unwrap().total(), 196_605);
    assert_eq!(metric.compare(i, o).unwrap().total(), 196_605);
    let alt = metric.with_profile(PronunciationProfile::CloseMidEV1);
    assert_eq!(alt.compare(e, o).unwrap().total(), 131_070);
    assert_eq!(alt.compare(i, o).unwrap().total(), 196_605);
    assert!(metric.compare(e, "b".parse().unwrap()).is_err());
    assert!(alt.compare(e, "b".parse().unwrap()).is_err());
}
