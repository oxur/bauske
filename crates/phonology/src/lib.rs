//! Source-grounded Relojban alphabet data and representative sound distances.
//!
//! The registry describes the acquired grammar snapshot `P01-S20261022`.
//! `representatives-v1` is a project-selected categorical articulatory profile;
//! it is not a measurement of perceived similarity. Comparisons are defined
//! only within the consonant and vowel classes.
//!
//! # Examples
//!
//! ```
//! use bauske_phonology::{DistanceModel, Segment};
//!
//! let result = DistanceModel::default().compare(
//!     "p".parse::<Segment>()?,
//!     "b".parse::<Segment>()?,
//! )?;
//! assert_eq!(result.total(), 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod clusters;
mod context;
mod distance;
mod error;
mod inventory;
mod profiles;
mod prosody;
mod realization;
mod types;

pub use clusters::{
    ClusterError, ClusterInspection, ClusterPosition, ClusterSourceTension, ClusterViolation,
    initial_pairs, initial_triples, inspect_cluster,
};
pub use context::{
    AnalysisDiagnostic, AnalysisError, DiagnosticKind, Nucleus, NucleusKind, WordAnalysis,
    WordCluster, WordKind, analyze_word,
};
pub use distance::{
    ConsonantWeights, DistanceExplanation, DistanceModel, FeatureContribution, FeatureDimension,
    FeatureValue, VowelWeights,
};
pub use error::{CompareError, ParseSegmentError, ParseSymbolError};
pub use inventory::{
    DiphthongDistribution, DiphthongRecord, Glide, GlideDirection, OpenPolicy, SymbolKind,
    SymbolRecord, alphabet, diphthongs, open_policies,
};
pub use profiles::{PronunciationProfile, profiles};
pub use prosody::{
    FollowingContext, ProsodyLimitation, ProsodySourceLimitation, StressAnalysis, StressChoice,
    StressError, StressLevel, Syllabification, SyllabificationMode, Syllable, assign_stress,
    syllabify,
};
pub use realization::{BufferEvent, RealizationError, RealizationPlan, plan_realization};
pub use types::{
    AlphabetSymbol, Backness, Features, Height, Manner, Place, Rounding, Segment, SegmentClass,
    Voicing,
};

/// Stable identifier for the acquired language-source snapshot.
pub const SOURCE_ID: &str = "P01-S20261022@0195f4b5b1f182ce8e21381210f766b326fb987f";
/// Stable identifier for the project's selected feature encoding.
pub const PROFILE_ID: &str = "representatives-v1";
