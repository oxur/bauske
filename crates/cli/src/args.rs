use clap::{Parser, Subcommand, ValueEnum};

/// Command-line arguments for the Relojban phonology tool.
#[derive(Debug, Parser)]
#[command(
    name = "relojban",
    version,
    about = "Browse Relojban phonology data and representative distances"
)]
pub(crate) struct Cli {
    /// The operation to perform.
    #[command(subcommand)]
    pub command: Command,
}

/// Supported command families.
#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Print a source-ordered TSV inventory.
    Inventory {
        /// Select the alphabet, morphological consonants, vowels, or diphthongs.
        #[arg(long, value_enum, default_value_t = InventoryKind::Alphabet)]
        kind: InventoryKind,
    },
    /// Explain a same-class representative comparison.
    Compare {
        /// Left representative symbol.
        left: String,
        /// Right representative symbol.
        right: String,
        /// Identified pronunciation profile; defaults to the accepted baseline.
        #[arg(long, value_enum, default_value_t = ProfileArg::RepresentativesV1)]
        profile: ProfileArg,
    },
    /// List the identified pronunciation representatives.
    Profiles,
    /// Inspect spelling-level nuclei, pauses, and consonant contexts.
    Analyze {
        /// One written word to inspect.
        word: String,
        /// Caller-declared word category; this does not infer morphology.
        #[arg(long, value_enum)]
        kind: WordKindArg,
        /// Original zero-based ASCII byte position of a syllabic sonorant; repeatable.
        #[arg(long = "syllabic-at", value_name = "N", action = clap::ArgAction::Append)]
        syllabic_at: Vec<usize>,
    },
    /// Inspect one consonant run in a declared context.
    Cluster {
        /// Consonants to inspect.
        cluster: String,
        /// Declared word position.
        #[arg(long, value_enum)]
        position: ClusterPositionArg,
        /// Caller-declared word category.
        #[arg(long, value_enum)]
        kind: WordKindArg,
    },
    /// Inspect bounded syllables, stress choices, and optional buffer events.
    Prosody {
        /// One written word to inspect.
        word: String,
        /// Caller-declared word category; this does not infer morphology.
        #[arg(long, value_enum)]
        kind: WordKindArg,
        /// Original zero-based ASCII byte position of a syllabic sonorant; repeatable.
        #[arg(long = "syllabic-at", value_name = "BYTE", action = clap::ArgAction::Append)]
        syllabic_at: Vec<usize>,
        /// Original zero-based ASCII byte position of a consonant to buffer; repeatable.
        #[arg(long = "buffer-after", value_name = "BYTE", action = clap::ArgAction::Append)]
        buffer_after: Vec<usize>,
        /// Select conventional grouping or explicit comma hints.
        #[arg(long, value_enum, default_value_t = SyllablesArg::Canonical)]
        syllables: SyllablesArg,
        /// Request primary stress at this zero-based nucleus index (not a byte offset).
        #[arg(long, value_name = "NUCLEUS", conflicts_with = "unstressed")]
        primary: Option<usize>,
        /// Request no primary stress where the selected kind permits it.
        #[arg(long)]
        unstressed: bool,
        /// Supply the following-brivla pause condition for the stress guard.
        #[arg(long, value_enum, default_value_t = FollowingArg::Unspecified)]
        following: FollowingArg,
    },
}

/// Closed CLI spelling of a selected finite pronunciation profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ProfileArg {
    /// Original representative set.
    RepresentativesV1,
    /// Allowed close-mid `[e]` representative.
    #[value(name = "close-mid-e-v1")]
    CloseMidEV1,
}

impl From<ProfileArg> for bauske_phonology::PronunciationProfile {
    fn from(value: ProfileArg) -> Self {
        match value {
            ProfileArg::RepresentativesV1 => Self::RepresentativesV1,
            ProfileArg::CloseMidEV1 => Self::CloseMidEV1,
        }
    }
}

/// Closed CLI spelling of syllabification mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SyllablesArg {
    /// Use the bounded conventional grouping policy.
    Canonical,
    /// Honor comma hints between adjacent nuclei.
    Hints,
}

impl From<SyllablesArg> for bauske_phonology::SyllabificationMode {
    fn from(value: SyllablesArg) -> Self {
        match value {
            SyllablesArg::Canonical => Self::Canonical,
            SyllablesArg::Hints => Self::WrittenHints,
        }
    }
}

/// Closed CLI spelling of the following-word context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum FollowingArg {
    /// Leave the following-word condition explicitly unchecked.
    Unspecified,
    /// An unpaused brivla follows.
    BrivlaUnpaused,
    /// A pause precedes the following brivla.
    BrivlaPaused,
}

impl From<FollowingArg> for bauske_phonology::FollowingContext {
    fn from(value: FollowingArg) -> Self {
        match value {
            FollowingArg::Unspecified => Self::Unspecified,
            FollowingArg::BrivlaUnpaused => Self::BrivlaUnpaused,
            FollowingArg::BrivlaPaused => Self::BrivlaAfterPause,
        }
    }
}

/// Inventory view selected by `relojban inventory --kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum InventoryKind {
    /// All alphabet characters, including punctuation and apostrophe.
    Alphabet,
    /// The 17 morphological consonants.
    Consonant,
    /// The six vowels.
    Vowel,
    /// The 16 source-listed diphthongs.
    Diphthong,
}

/// Closed CLI spelling of the caller-declared word category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum WordKindArg {
    /// Content word.
    Brivla,
    /// Structural word.
    Cmavo,
    /// Proper name.
    Cmevla,
    /// Zi'evla subtype.
    Zihevla,
    /// Gismu-based lujvo subtype.
    #[value(name = "gismu-lujvo")]
    GismuLujvo,
}

impl From<WordKindArg> for bauske_phonology::WordKind {
    fn from(value: WordKindArg) -> Self {
        match value {
            WordKindArg::Brivla => Self::Brivla,
            WordKindArg::Cmavo => Self::Cmavo,
            WordKindArg::Cmevla => Self::Cmevla,
            WordKindArg::Zihevla => Self::Zihevla,
            WordKindArg::GismuLujvo => Self::GismuBasedLujvo,
        }
    }
}

/// Closed CLI spelling of the cluster's word position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ClusterPositionArg {
    /// Word-initial context.
    Initial,
    /// Word-medial context.
    Medial,
    /// Word-final context.
    Final,
}

impl From<ClusterPositionArg> for bauske_phonology::ClusterPosition {
    fn from(value: ClusterPositionArg) -> Self {
        match value {
            ClusterPositionArg::Initial => Self::Initial,
            ClusterPositionArg::Medial => Self::Medial,
            ClusterPositionArg::Final => Self::Final,
        }
    }
}
