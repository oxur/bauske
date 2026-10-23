use crate::{Backness, Features, Height, Manner, Place, Rounding, Voicing};

/// Language-source role of one alphabet character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    /// One of the 17 morphological consonants.
    Consonant,
    /// One of the six vowels.
    Vowel,
    /// Consonantal \[h\], excluded from morphology's consonant/vowel count.
    Apostrophe,
    /// Written pause indicator.
    Pause,
    /// Written syllable separator that does not mark a pause.
    SyllableSeparator,
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Consonant => "consonant",
            Self::Vowel => "vowel",
            Self::Apostrophe => "apostrophe",
            Self::Pause => "pause",
            Self::SyllableSeparator => "syllable-separator",
        })
    }
}

/// Open or variable pronunciation behavior retained as source metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpenPolicy {
    /// The listed and other rhotic realizations are equally acceptable.
    RhoticAlternativesEquallyAccepted,
    /// Vowel rounding may vary from the preferred profile.
    VowelRoundingMayVary,
    /// Consonants may be aspirated or unaspirated.
    AspirationMayVary,
    /// Sonorant consonants may be voiced or unvoiced.
    SonorantsMayBeUnvoiced,
    /// An unused voiceless fricative may substitute for apostrophe \[h\].
    ApostropheSubstituteIsOpenEnded,
    /// Non-Relojban vowels may be inserted as buffers where needed.
    BufferVowelIsOpenEnded,
    /// Diphthong variants are omitted from the source's table.
    DiphthongVariantsAreNotEnumerated,
    /// l, m, n, and r may have syllabic realizations in restricted contexts.
    SyllabicSonorantsHaveContextualVariants,
}

impl OpenPolicy {
    /// Returns a stable, concise label for tables and reports.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::RhoticAlternativesEquallyAccepted => "rhotic-alternatives-equally-accepted",
            Self::VowelRoundingMayVary => "vowel-rounding-may-vary",
            Self::AspirationMayVary => "aspiration-may-vary",
            Self::SonorantsMayBeUnvoiced => "sonorants-may-be-unvoiced",
            Self::ApostropheSubstituteIsOpenEnded => "apostrophe-substitute-open-ended",
            Self::BufferVowelIsOpenEnded => "buffer-vowel-open-ended",
            Self::DiphthongVariantsAreNotEnumerated => "diphthong-variants-not-enumerated",
            Self::SyllabicSonorantsHaveContextualVariants => {
                "syllabic-sonorant-contextual-variants"
            }
        }
    }

    /// Returns the grammar sections that state this open pronunciation policy.
    #[must_use]
    pub const fn source_sections(self) -> &'static [&'static str] {
        match self {
            Self::RhoticAlternativesEquallyAccepted
            | Self::VowelRoundingMayVary
            | Self::AspirationMayVary => &SEC32,
            Self::SonorantsMayBeUnvoiced => &SEC36,
            Self::ApostropheSubstituteIsOpenEnded => &SEC33,
            Self::BufferVowelIsOpenEnded => &SEC38,
            Self::DiphthongVariantsAreNotEnumerated => &SEC34,
            Self::SyllabicSonorantsHaveContextualVariants => &SEC34_39,
        }
    }
}

/// Immutable metadata for one alphabet entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolRecord {
    symbol: &'static str,
    kind: SymbolKind,
    representative_ipa: Option<&'static str>,
    alternatives: &'static [&'static str],
    features: Option<Features>,
    source_sections: &'static [&'static str],
    policies: &'static [OpenPolicy],
}

impl SymbolRecord {
    /// Returns the one-character orthographic spelling.
    #[must_use]
    pub const fn symbol(&self) -> &'static str {
        self.symbol
    }
    /// Returns the source-defined alphabet role.
    #[must_use]
    pub const fn kind(&self) -> SymbolKind {
        self.kind
    }
    /// Returns the selected profile's IPA representative, when scored.
    #[must_use]
    pub const fn representative_ipa(&self) -> Option<&'static str> {
        self.representative_ipa
    }
    /// Returns all IPA forms explicitly listed in the acquired pronunciation table.
    #[must_use]
    pub const fn alternatives(&self) -> &'static [&'static str] {
        self.alternatives
    }
    /// Returns the selected categorical feature encoding, if this entry is scored.
    #[must_use]
    pub const fn features(&self) -> Option<Features> {
        self.features
    }
    /// Returns source section IDs supporting this record.
    #[must_use]
    pub const fn source_sections(&self) -> &'static [&'static str] {
        self.source_sections
    }
    /// Returns open pronunciation policies attached to this symbol.
    #[must_use]
    pub const fn policies(&self) -> &'static [OpenPolicy] {
        self.policies
    }
}

const C12: &[&str] = &["3.1", "3.2"];
const C123: &[&str] = &["3.1", "3.2", "3.3"];
const SEC32: [&str; 1] = ["3.2"];
const SEC33: [&str; 1] = ["3.3"];
const SEC34: [&str; 1] = ["3.4"];
const SEC36: [&str; 1] = ["3.6"];
const SEC38: [&str; 1] = ["3.8"];
const SEC34_39: [&str; 2] = ["3.4", "3.9"];
const SONORANT_SOURCES: &[&str] = &["3.1", "3.2", "3.4", "3.6", "3.9"];
const VOWEL_POLICY: &[OpenPolicy] = &[OpenPolicy::VowelRoundingMayVary];
const VOWEL_Y_POLICY: &[OpenPolicy] = &[OpenPolicy::VowelRoundingMayVary];
const SONORANT_POLICY: &[OpenPolicy] = &[
    OpenPolicy::AspirationMayVary,
    OpenPolicy::SonorantsMayBeUnvoiced,
    OpenPolicy::SyllabicSonorantsHaveContextualVariants,
];
const R_POLICY: &[OpenPolicy] = &[
    OpenPolicy::AspirationMayVary,
    OpenPolicy::RhoticAlternativesEquallyAccepted,
    OpenPolicy::SonorantsMayBeUnvoiced,
    OpenPolicy::SyllabicSonorantsHaveContextualVariants,
];
const ASPIRATED: &[OpenPolicy] = &[OpenPolicy::AspirationMayVary];
const APOSTROPHE_POLICY: &[OpenPolicy] = &[
    OpenPolicy::AspirationMayVary,
    OpenPolicy::ApostropheSubstituteIsOpenEnded,
];
const NO_POLICY: &[OpenPolicy] = &[];

static ALPHABET: [SymbolRecord; 26] = [
    SymbolRecord {
        symbol: "'",
        kind: SymbolKind::Apostrophe,
        representative_ipa: Some("h"),
        alternatives: &["h"],
        features: Some(Features::Consonant {
            place: Place::Glottal,
            manner: Manner::Fricative,
            voicing: Voicing::Voiceless,
        }),
        source_sections: C123,
        policies: APOSTROPHE_POLICY,
    },
    SymbolRecord {
        symbol: ",",
        kind: SymbolKind::SyllableSeparator,
        representative_ipa: None,
        alternatives: &["."],
        features: None,
        source_sections: C12,
        policies: NO_POLICY,
    },
    SymbolRecord {
        symbol: ".",
        kind: SymbolKind::Pause,
        representative_ipa: None,
        alternatives: &["ʔ"],
        features: None,
        source_sections: C123,
        policies: NO_POLICY,
    },
    SymbolRecord {
        symbol: "a",
        kind: SymbolKind::Vowel,
        representative_ipa: Some("a"),
        alternatives: &["a", "ɑ"],
        features: Some(Features::Vowel {
            height: Height::Open,
            backness: Backness::Front,
            rounding: Rounding::Unrounded,
        }),
        source_sections: C12,
        policies: VOWEL_POLICY,
    },
    SymbolRecord {
        symbol: "b",
        kind: SymbolKind::Consonant,
        representative_ipa: Some("b"),
        alternatives: &["b"],
        features: Some(Features::Consonant {
            place: Place::Bilabial,
            manner: Manner::Plosive,
            voicing: Voicing::Voiced,
        }),
        source_sections: C12,
        policies: ASPIRATED,
    },
    SymbolRecord {
        symbol: "c",
        kind: SymbolKind::Consonant,
        representative_ipa: Some("ʃ"),
        alternatives: &["ʃ", "ʂ"],
        features: Some(Features::Consonant {
            place: Place::Postalveolar,
            manner: Manner::Fricative,
            voicing: Voicing::Voiceless,
        }),
        source_sections: C12,
        policies: ASPIRATED,
    },
    SymbolRecord {
        symbol: "d",
        kind: SymbolKind::Consonant,
        representative_ipa: Some("d"),
        alternatives: &["d"],
        features: Some(Features::Consonant {
            place: Place::Alveolar,
            manner: Manner::Plosive,
            voicing: Voicing::Voiced,
        }),
        source_sections: C12,
        policies: ASPIRATED,
    },
    SymbolRecord {
        symbol: "e",
        kind: SymbolKind::Vowel,
        representative_ipa: Some("ɛ"),
        alternatives: &["ɛ", "e"],
        features: Some(Features::Vowel {
            height: Height::OpenMid,
            backness: Backness::Front,
            rounding: Rounding::Unrounded,
        }),
        source_sections: C12,
        policies: VOWEL_POLICY,
    },
    SymbolRecord {
        symbol: "f",
        kind: SymbolKind::Consonant,
        representative_ipa: Some("f"),
        alternatives: &["f", "ɸ"],
        features: Some(Features::Consonant {
            place: Place::Labiodental,
            manner: Manner::Fricative,
            voicing: Voicing::Voiceless,
        }),
        source_sections: C12,
        policies: ASPIRATED,
    },
    SymbolRecord {
        symbol: "g",
        kind: SymbolKind::Consonant,
        representative_ipa: Some("ɡ"),
        alternatives: &["ɡ"],
        features: Some(Features::Consonant {
            place: Place::Velar,
            manner: Manner::Plosive,
            voicing: Voicing::Voiced,
        }),
        source_sections: C12,
        policies: ASPIRATED,
    },
    SymbolRecord {
        symbol: "i",
        kind: SymbolKind::Vowel,
        representative_ipa: Some("i"),
        alternatives: &["i"],
        features: Some(Features::Vowel {
            height: Height::Close,
            backness: Backness::Front,
            rounding: Rounding::Unrounded,
        }),
        source_sections: C12,
        policies: VOWEL_POLICY,
    },
    SymbolRecord {
        symbol: "j",
        kind: SymbolKind::Consonant,
        representative_ipa: Some("ʒ"),
        alternatives: &["ʒ", "ʐ"],
        features: Some(Features::Consonant {
            place: Place::Postalveolar,
            manner: Manner::Fricative,
            voicing: Voicing::Voiced,
        }),
        source_sections: C12,
        policies: ASPIRATED,
    },
    SymbolRecord {
        symbol: "k",
        kind: SymbolKind::Consonant,
        representative_ipa: Some("k"),
        alternatives: &["k"],
        features: Some(Features::Consonant {
            place: Place::Velar,
            manner: Manner::Plosive,
            voicing: Voicing::Voiceless,
        }),
        source_sections: C12,
        policies: ASPIRATED,
    },
    SymbolRecord {
        symbol: "l",
        kind: SymbolKind::Consonant,
        representative_ipa: Some("l"),
        alternatives: &["l", "l̩"],
        features: Some(Features::Consonant {
            place: Place::Alveolar,
            manner: Manner::LateralApproximant,
            voicing: Voicing::Voiced,
        }),
        source_sections: SONORANT_SOURCES,
        policies: SONORANT_POLICY,
    },
    SymbolRecord {
        symbol: "m",
        kind: SymbolKind::Consonant,
        representative_ipa: Some("m"),
        alternatives: &["m", "m̩"],
        features: Some(Features::Consonant {
            place: Place::Bilabial,
            manner: Manner::Nasal,
            voicing: Voicing::Voiced,
        }),
        source_sections: SONORANT_SOURCES,
        policies: SONORANT_POLICY,
    },
    SymbolRecord {
        symbol: "n",
        kind: SymbolKind::Consonant,
        representative_ipa: Some("n"),
        alternatives: &["n", "n̩", "ŋ", "ŋ̍"],
        features: Some(Features::Consonant {
            place: Place::Alveolar,
            manner: Manner::Nasal,
            voicing: Voicing::Voiced,
        }),
        source_sections: SONORANT_SOURCES,
        policies: SONORANT_POLICY,
    },
    SymbolRecord {
        symbol: "o",
        kind: SymbolKind::Vowel,
        representative_ipa: Some("o"),
        alternatives: &["o", "ɔ"],
        features: Some(Features::Vowel {
            height: Height::CloseMid,
            backness: Backness::Back,
            rounding: Rounding::Rounded,
        }),
        source_sections: C12,
        policies: VOWEL_POLICY,
    },
    SymbolRecord {
        symbol: "p",
        kind: SymbolKind::Consonant,
        representative_ipa: Some("p"),
        alternatives: &["p"],
        features: Some(Features::Consonant {
            place: Place::Bilabial,
            manner: Manner::Plosive,
            voicing: Voicing::Voiceless,
        }),
        source_sections: C12,
        policies: ASPIRATED,
    },
    SymbolRecord {
        symbol: "r",
        kind: SymbolKind::Consonant,
        representative_ipa: Some("r"),
        alternatives: &["r", "ɹ", "ɾ", "ʀ", "r̩", "ɹ̩", "ʀ̩"],
        features: Some(Features::Consonant {
            place: Place::Alveolar,
            manner: Manner::Trill,
            voicing: Voicing::Voiced,
        }),
        source_sections: SONORANT_SOURCES,
        policies: R_POLICY,
    },
    SymbolRecord {
        symbol: "s",
        kind: SymbolKind::Consonant,
        representative_ipa: Some("s"),
        alternatives: &["s"],
        features: Some(Features::Consonant {
            place: Place::Alveolar,
            manner: Manner::Fricative,
            voicing: Voicing::Voiceless,
        }),
        source_sections: C12,
        policies: ASPIRATED,
    },
    SymbolRecord {
        symbol: "t",
        kind: SymbolKind::Consonant,
        representative_ipa: Some("t"),
        alternatives: &["t"],
        features: Some(Features::Consonant {
            place: Place::Alveolar,
            manner: Manner::Plosive,
            voicing: Voicing::Voiceless,
        }),
        source_sections: C12,
        policies: ASPIRATED,
    },
    SymbolRecord {
        symbol: "u",
        kind: SymbolKind::Vowel,
        representative_ipa: Some("u"),
        alternatives: &["u"],
        features: Some(Features::Vowel {
            height: Height::Close,
            backness: Backness::Back,
            rounding: Rounding::Rounded,
        }),
        source_sections: C12,
        policies: VOWEL_POLICY,
    },
    SymbolRecord {
        symbol: "v",
        kind: SymbolKind::Consonant,
        representative_ipa: Some("v"),
        alternatives: &["v", "β"],
        features: Some(Features::Consonant {
            place: Place::Labiodental,
            manner: Manner::Fricative,
            voicing: Voicing::Voiced,
        }),
        source_sections: C12,
        policies: ASPIRATED,
    },
    SymbolRecord {
        symbol: "x",
        kind: SymbolKind::Consonant,
        representative_ipa: Some("x"),
        alternatives: &["x"],
        features: Some(Features::Consonant {
            place: Place::Velar,
            manner: Manner::Fricative,
            voicing: Voicing::Voiceless,
        }),
        source_sections: C12,
        policies: ASPIRATED,
    },
    SymbolRecord {
        symbol: "y",
        kind: SymbolKind::Vowel,
        representative_ipa: Some("ə"),
        alternatives: &["ə"],
        features: Some(Features::Vowel {
            height: Height::Mid,
            backness: Backness::Central,
            rounding: Rounding::Unrounded,
        }),
        source_sections: C12,
        policies: VOWEL_Y_POLICY,
    },
    SymbolRecord {
        symbol: "z",
        kind: SymbolKind::Consonant,
        representative_ipa: Some("z"),
        alternatives: &["z"],
        features: Some(Features::Consonant {
            place: Place::Alveolar,
            manner: Manner::Fricative,
            voicing: Voicing::Voiced,
        }),
        source_sections: C12,
        policies: ASPIRATED,
    },
];

/// Returns the complete alphabet registry in source ASCII order.
#[must_use]
pub fn alphabet() -> &'static [SymbolRecord] {
    &ALPHABET
}

/// Returns language-wide open realization policies not attached to one symbol.
#[must_use]
pub const fn open_policies() -> &'static [OpenPolicy] {
    &[
        OpenPolicy::BufferVowelIsOpenEnded,
        OpenPolicy::DiphthongVariantsAreNotEnumerated,
    ]
}

/// The glide identity in a source-listed diphthong.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Glide {
    /// Palatal glide \[j\].
    Palatal,
    /// Labial glide \[w\].
    Labial,
}

impl Glide {
    /// Returns the IPA symbol for this glide.
    #[must_use]
    pub const fn ipa(self) -> &'static str {
        match self {
            Self::Palatal => "j",
            Self::Labial => "w",
        }
    }
}

/// Whether the glide precedes or follows the vowel element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlideDirection {
    /// The glide follows the vowel (first four source entries).
    Off,
    /// The glide precedes the vowel (remaining twelve source entries).
    On,
}

/// Source distribution group for the diphthong spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiphthongDistribution {
    /// Freely used in most word types, like pure vowels.
    FreelyUsed,
    /// Absent from gismu-based lujvo; behaves like a CV syllable.
    RestrictedMorphology,
}

/// Immutable source metadata for one listed diphthong.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiphthongRecord {
    spelling: &'static str,
    ipa: &'static str,
    glide: Glide,
    direction: GlideDirection,
    distribution: DiphthongDistribution,
    source_sections: &'static [&'static str],
}

impl DiphthongRecord {
    /// Returns the two-letter spelling.
    #[must_use]
    pub const fn spelling(&self) -> &'static str {
        self.spelling
    }
    /// Returns the source IPA form without brackets.
    #[must_use]
    pub const fn ipa(&self) -> &'static str {
        self.ipa
    }
    /// Returns whether the glide is palatal or labial.
    #[must_use]
    pub const fn glide(&self) -> Glide {
        self.glide
    }
    /// Returns whether the glide is an on-glide or off-glide.
    #[must_use]
    pub const fn direction(&self) -> GlideDirection {
        self.direction
    }
    /// Returns the source distribution group.
    #[must_use]
    pub const fn distribution(&self) -> DiphthongDistribution {
        self.distribution
    }
    /// Returns source section IDs supporting this record.
    #[must_use]
    pub const fn source_sections(&self) -> &'static [&'static str] {
        self.source_sections
    }
}

const D34: &[&str] = &["3.4"];
static DIPHTHONGS: [DiphthongRecord; 16] = [
    DiphthongRecord {
        spelling: "ai",
        ipa: "aj",
        glide: Glide::Palatal,
        direction: GlideDirection::Off,
        distribution: DiphthongDistribution::FreelyUsed,
        source_sections: D34,
    },
    DiphthongRecord {
        spelling: "ei",
        ipa: "ɛj",
        glide: Glide::Palatal,
        direction: GlideDirection::Off,
        distribution: DiphthongDistribution::FreelyUsed,
        source_sections: D34,
    },
    DiphthongRecord {
        spelling: "oi",
        ipa: "oj",
        glide: Glide::Palatal,
        direction: GlideDirection::Off,
        distribution: DiphthongDistribution::FreelyUsed,
        source_sections: D34,
    },
    DiphthongRecord {
        spelling: "au",
        ipa: "aw",
        glide: Glide::Labial,
        direction: GlideDirection::Off,
        distribution: DiphthongDistribution::FreelyUsed,
        source_sections: D34,
    },
    DiphthongRecord {
        spelling: "ia",
        ipa: "ja",
        glide: Glide::Palatal,
        direction: GlideDirection::On,
        distribution: DiphthongDistribution::RestrictedMorphology,
        source_sections: D34,
    },
    DiphthongRecord {
        spelling: "ie",
        ipa: "jɛ",
        glide: Glide::Palatal,
        direction: GlideDirection::On,
        distribution: DiphthongDistribution::RestrictedMorphology,
        source_sections: D34,
    },
    DiphthongRecord {
        spelling: "ii",
        ipa: "ji",
        glide: Glide::Palatal,
        direction: GlideDirection::On,
        distribution: DiphthongDistribution::RestrictedMorphology,
        source_sections: D34,
    },
    DiphthongRecord {
        spelling: "io",
        ipa: "jo",
        glide: Glide::Palatal,
        direction: GlideDirection::On,
        distribution: DiphthongDistribution::RestrictedMorphology,
        source_sections: D34,
    },
    DiphthongRecord {
        spelling: "iu",
        ipa: "ju",
        glide: Glide::Palatal,
        direction: GlideDirection::On,
        distribution: DiphthongDistribution::RestrictedMorphology,
        source_sections: D34,
    },
    DiphthongRecord {
        spelling: "ua",
        ipa: "wa",
        glide: Glide::Labial,
        direction: GlideDirection::On,
        distribution: DiphthongDistribution::RestrictedMorphology,
        source_sections: D34,
    },
    DiphthongRecord {
        spelling: "ue",
        ipa: "wɛ",
        glide: Glide::Labial,
        direction: GlideDirection::On,
        distribution: DiphthongDistribution::RestrictedMorphology,
        source_sections: D34,
    },
    DiphthongRecord {
        spelling: "ui",
        ipa: "wi",
        glide: Glide::Labial,
        direction: GlideDirection::On,
        distribution: DiphthongDistribution::RestrictedMorphology,
        source_sections: D34,
    },
    DiphthongRecord {
        spelling: "uo",
        ipa: "wo",
        glide: Glide::Labial,
        direction: GlideDirection::On,
        distribution: DiphthongDistribution::RestrictedMorphology,
        source_sections: D34,
    },
    DiphthongRecord {
        spelling: "uu",
        ipa: "wu",
        glide: Glide::Labial,
        direction: GlideDirection::On,
        distribution: DiphthongDistribution::RestrictedMorphology,
        source_sections: D34,
    },
    DiphthongRecord {
        spelling: "iy",
        ipa: "jə",
        glide: Glide::Palatal,
        direction: GlideDirection::On,
        distribution: DiphthongDistribution::RestrictedMorphology,
        source_sections: D34,
    },
    DiphthongRecord {
        spelling: "uy",
        ipa: "wə",
        glide: Glide::Labial,
        direction: GlideDirection::On,
        distribution: DiphthongDistribution::RestrictedMorphology,
        source_sections: D34,
    },
];

/// Returns the complete diphthong table in source order.
#[must_use]
pub fn diphthongs() -> &'static [DiphthongRecord] {
    &DIPHTHONGS
}
