use std::num::NonZeroU16;

use crate::{
    Backness, CompareError, Features, Height, Manner, Place, PronunciationProfile, Rounding,
    Segment, SegmentClass, Voicing,
};

/// Positive costs for place, manner, and voicing mismatches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsonantWeights {
    place: NonZeroU16,
    manner: NonZeroU16,
    voicing: NonZeroU16,
}

impl ConsonantWeights {
    /// Creates positive weights for the three consonant dimensions.
    #[must_use]
    pub const fn new(place: NonZeroU16, manner: NonZeroU16, voicing: NonZeroU16) -> Self {
        Self {
            place,
            manner,
            voicing,
        }
    }
    /// Returns the place weight.
    #[must_use]
    pub const fn place(self) -> NonZeroU16 {
        self.place
    }
    /// Returns the manner weight.
    #[must_use]
    pub const fn manner(self) -> NonZeroU16 {
        self.manner
    }
    /// Returns the voicing weight.
    #[must_use]
    pub const fn voicing(self) -> NonZeroU16 {
        self.voicing
    }
}

impl Default for ConsonantWeights {
    fn default() -> Self {
        let one = NonZeroU16::new(1).expect("one is nonzero");
        Self::new(one, one, one)
    }
}

/// Positive costs for height, backness, and rounding mismatches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VowelWeights {
    height: NonZeroU16,
    backness: NonZeroU16,
    rounding: NonZeroU16,
}

impl VowelWeights {
    /// Creates positive weights for the three vowel dimensions.
    #[must_use]
    pub const fn new(height: NonZeroU16, backness: NonZeroU16, rounding: NonZeroU16) -> Self {
        Self {
            height,
            backness,
            rounding,
        }
    }
    /// Returns the height weight.
    #[must_use]
    pub const fn height(self) -> NonZeroU16 {
        self.height
    }
    /// Returns the backness weight.
    #[must_use]
    pub const fn backness(self) -> NonZeroU16 {
        self.backness
    }
    /// Returns the rounding weight.
    #[must_use]
    pub const fn rounding(self) -> NonZeroU16 {
        self.rounding
    }
}

impl Default for VowelWeights {
    fn default() -> Self {
        let one = NonZeroU16::new(1).expect("one is nonzero");
        Self::new(one, one, one)
    }
}

/// The typed feature dimension shown in one comparison contribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureDimension {
    /// Consonantal place.
    Place,
    /// Consonantal manner.
    Manner,
    /// Consonantal voicing.
    Voicing,
    /// Vowel height.
    Height,
    /// Vowel backness.
    Backness,
    /// Vowel rounding.
    Rounding,
}

impl FeatureDimension {
    /// Returns the stable lower-case output label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Place => "place",
            Self::Manner => "manner",
            Self::Voicing => "voicing",
            Self::Height => "height",
            Self::Backness => "backness",
            Self::Rounding => "rounding",
        }
    }
}

/// A typed categorical feature value used in a comparison explanation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FeatureValue {
    /// A consonantal place value.
    Place(Place),
    /// A consonantal manner value.
    Manner(Manner),
    /// A consonantal voicing value.
    Voicing(Voicing),
    /// A vowel height value.
    Height(Height),
    /// A vowel backness value.
    Backness(Backness),
    /// A vowel rounding value.
    Rounding(Rounding),
}

impl FeatureValue {
    /// Returns a stable label for display.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Place(v) => match v {
                Place::Bilabial => "bilabial",
                Place::Labiodental => "labiodental",
                Place::Alveolar => "alveolar",
                Place::Postalveolar => "postalveolar",
                Place::Velar => "velar",
                Place::Glottal => "glottal",
            },
            Self::Manner(v) => match v {
                Manner::Plosive => "plosive",
                Manner::Fricative => "fricative",
                Manner::Nasal => "nasal",
                Manner::LateralApproximant => "lateral-approximant",
                Manner::Trill => "trill",
            },
            Self::Voicing(v) => match v {
                Voicing::Voiceless => "voiceless",
                Voicing::Voiced => "voiced",
            },
            Self::Height(v) => match v {
                Height::Open => "open",
                Height::OpenMid => "open-mid",
                Height::Mid => "mid",
                Height::CloseMid => "close-mid",
                Height::Close => "close",
            },
            Self::Backness(v) => match v {
                Backness::Front => "front",
                Backness::Central => "central",
                Backness::Back => "back",
            },
            Self::Rounding(v) => match v {
                Rounding::Unrounded => "unrounded",
                Rounding::Rounded => "rounded",
            },
        }
    }
}

/// One coordinate and its positive mismatch weight in a comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureContribution {
    dimension: FeatureDimension,
    left: FeatureValue,
    right: FeatureValue,
    weight: NonZeroU16,
    cost: u32,
}

impl FeatureContribution {
    /// Returns the compared coordinate.
    #[must_use]
    pub const fn dimension(&self) -> FeatureDimension {
        self.dimension
    }
    /// Returns the left coordinate value.
    #[must_use]
    pub const fn left(&self) -> FeatureValue {
        self.left
    }
    /// Returns the right coordinate value.
    #[must_use]
    pub const fn right(&self) -> FeatureValue {
        self.right
    }
    /// Returns the configured positive weight.
    #[must_use]
    pub const fn weight(&self) -> NonZeroU16 {
        self.weight
    }
    /// Returns zero for equality or the configured weight for a mismatch.
    #[must_use]
    pub const fn cost(&self) -> u32 {
        self.cost
    }
}

/// A complete, immutable explanation of a successful comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistanceExplanation {
    left: Segment,
    right: Segment,
    class: SegmentClass,
    total: u32,
    contributions: [FeatureContribution; 3],
    profile: PronunciationProfile,
}

impl DistanceExplanation {
    /// Returns the left representative.
    #[must_use]
    pub const fn left(&self) -> Segment {
        self.left
    }
    /// Returns the right representative.
    #[must_use]
    pub const fn right(&self) -> Segment {
        self.right
    }
    /// Returns the shared comparison class.
    #[must_use]
    pub const fn class(&self) -> SegmentClass {
        self.class
    }
    /// Returns the sum of the three contribution costs.
    #[must_use]
    pub const fn total(&self) -> u32 {
        self.total
    }
    /// Returns the profile used to obtain the explained feature values.
    #[must_use]
    pub const fn profile(&self) -> PronunciationProfile {
        self.profile
    }
    /// Returns the stable profile identifier used for this comparison.
    #[must_use]
    pub const fn profile_id(&self) -> &'static str {
        self.profile.id()
    }
    /// Returns the acquired language-source identifier used for this comparison.
    #[must_use]
    pub const fn source_id(&self) -> &'static str {
        self.profile.source_id()
    }
    /// Returns all three contributions in fixed feature order.
    #[must_use]
    pub const fn contributions(&self) -> &[FeatureContribution] {
        &self.contributions
    }
}

/// A categorical weighted-mismatch model over separate consonant and vowel classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistanceModel {
    consonants: ConsonantWeights,
    vowels: VowelWeights,
    profile: PronunciationProfile,
}

impl DistanceModel {
    /// Creates a model from strictly positive class-specific weights.
    #[must_use]
    pub const fn new(consonants: ConsonantWeights, vowels: VowelWeights) -> Self {
        Self {
            consonants,
            vowels,
            profile: PronunciationProfile::RepresentativesV1,
        }
    }
    /// Returns a copy of this model using the identified representative profile.
    #[must_use]
    pub const fn with_profile(mut self, profile: PronunciationProfile) -> Self {
        self.profile = profile;
        self
    }
    /// Returns the representative profile used by this model.
    #[must_use]
    pub const fn profile(&self) -> PronunciationProfile {
        self.profile
    }
    /// Returns the profile identity used by this model.
    #[must_use]
    pub const fn profile_id(&self) -> &'static str {
        self.profile.id()
    }
    /// Returns the acquired language-source identity used by this model.
    #[must_use]
    pub const fn source_id(&self) -> &'static str {
        self.profile.source_id()
    }

    /// Compares two representatives in the same declared feature class.
    ///
    /// # Errors
    ///
    /// Returns [`CompareError::DifferentClasses`] when one segment is a consonant
    /// and the other is a vowel; this model defines no cross-class score.
    pub fn compare(
        &self,
        left: Segment,
        right: Segment,
    ) -> Result<DistanceExplanation, CompareError> {
        if left.class() != right.class() {
            return Err(CompareError::DifferentClasses {
                left: left.class(),
                right: right.class(),
            });
        }
        let contributions = match (self.profile.features(left), self.profile.features(right)) {
            (
                Features::Consonant {
                    place: lp,
                    manner: lm,
                    voicing: lv,
                },
                Features::Consonant {
                    place: rp,
                    manner: rm,
                    voicing: rv,
                },
            ) => [
                contribution(
                    FeatureDimension::Place,
                    FeatureValue::Place(lp),
                    FeatureValue::Place(rp),
                    self.consonants.place,
                ),
                contribution(
                    FeatureDimension::Manner,
                    FeatureValue::Manner(lm),
                    FeatureValue::Manner(rm),
                    self.consonants.manner,
                ),
                contribution(
                    FeatureDimension::Voicing,
                    FeatureValue::Voicing(lv),
                    FeatureValue::Voicing(rv),
                    self.consonants.voicing,
                ),
            ],
            (
                Features::Vowel {
                    height: lh,
                    backness: lb,
                    rounding: lr,
                },
                Features::Vowel {
                    height: rh,
                    backness: rb,
                    rounding: rr,
                },
            ) => [
                contribution(
                    FeatureDimension::Height,
                    FeatureValue::Height(lh),
                    FeatureValue::Height(rh),
                    self.vowels.height,
                ),
                contribution(
                    FeatureDimension::Backness,
                    FeatureValue::Backness(lb),
                    FeatureValue::Backness(rb),
                    self.vowels.backness,
                ),
                contribution(
                    FeatureDimension::Rounding,
                    FeatureValue::Rounding(lr),
                    FeatureValue::Rounding(rr),
                    self.vowels.rounding,
                ),
            ],
            _ => {
                return Err(CompareError::DifferentClasses {
                    left: left.class(),
                    right: right.class(),
                });
            }
        };
        let total = contributions.iter().map(|item| item.cost).sum();
        Ok(DistanceExplanation {
            left,
            right,
            class: left.class(),
            total,
            contributions,
            profile: self.profile,
        })
    }
}

fn contribution(
    dimension: FeatureDimension,
    left: FeatureValue,
    right: FeatureValue,
    weight: NonZeroU16,
) -> FeatureContribution {
    let equal = match (left, right) {
        (FeatureValue::Place(a), FeatureValue::Place(b)) => a == b,
        (FeatureValue::Manner(a), FeatureValue::Manner(b)) => a == b,
        (FeatureValue::Voicing(a), FeatureValue::Voicing(b)) => a == b,
        (FeatureValue::Height(a), FeatureValue::Height(b)) => a == b,
        (FeatureValue::Backness(a), FeatureValue::Backness(b)) => a == b,
        (FeatureValue::Rounding(a), FeatureValue::Rounding(b)) => a == b,
        _ => false,
    };
    FeatureContribution {
        dimension,
        left,
        right,
        weight,
        cost: if equal { 0 } else { u32::from(weight.get()) },
    }
}

impl Default for DistanceModel {
    fn default() -> Self {
        Self::new(ConsonantWeights::default(), VowelWeights::default())
    }
}
