//! Identified finite pronunciation representatives for numeric comparisons.

use crate::{Backness, Features, Height, Rounding, Segment};

/// A finite project-selected representative profile, not an observed pronunciation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PronunciationProfile {
    /// The original categorical representatives, unchanged from the first increment.
    RepresentativesV1,
    /// The same representatives with the allowed close-mid `[e]` choice for `e`.
    CloseMidEV1,
}

impl PronunciationProfile {
    /// Returns the stable identifier for this profile.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::RepresentativesV1 => "representatives-v1",
            Self::CloseMidEV1 => "close-mid-e-v1",
        }
    }

    /// Returns the grammar acquisition that grounds the selected representatives.
    #[must_use]
    pub const fn source_id(self) -> &'static str {
        crate::SOURCE_ID
    }

    /// Returns the profile-specific categorical feature vector for a segment.
    #[must_use]
    pub fn features(self, segment: Segment) -> Features {
        if self == Self::CloseMidEV1 && segment.as_str() == "e" {
            Features::Vowel {
                height: Height::CloseMid,
                backness: Backness::Front,
                rounding: Rounding::Unrounded,
            }
        } else {
            segment.features()
        }
    }

    /// Returns the selected IPA representative, including brackets.
    #[must_use]
    pub fn ipa(self, segment: Segment) -> &'static str {
        if self == Self::CloseMidEV1 && segment.as_str() == "e" {
            "[e]"
        } else {
            match segment.as_str() {
                "'" => "[h]",
                "a" => "[a]",
                "b" => "[b]",
                "c" => "[ʃ]",
                "d" => "[d]",
                "e" => "[ɛ]",
                "f" => "[f]",
                "g" => "[ɡ]",
                "i" => "[i]",
                "j" => "[ʒ]",
                "k" => "[k]",
                "l" => "[l]",
                "m" => "[m]",
                "n" => "[n]",
                "o" => "[o]",
                "p" => "[p]",
                "r" => "[r]",
                "s" => "[s]",
                "t" => "[t]",
                "u" => "[u]",
                "v" => "[v]",
                "x" => "[x]",
                "y" => "[ə]",
                "z" => "[z]",
                _ => "[?]",
            }
        }
    }

    /// Returns the number of representatives with profile-specific features.
    #[must_use]
    pub const fn representative_count(self) -> usize {
        24
    }
}

/// Returns the immutable listing of supported, named profiles.
#[must_use]
pub const fn profiles() -> &'static [PronunciationProfile] {
    &[
        PronunciationProfile::RepresentativesV1,
        PronunciationProfile::CloseMidEV1,
    ]
}
