# bauske-phonology

Source-grounded Relojban phonology for the [Bauske](https://github.com/oxur/bauske) linguistic workbench.

The library provides an alphabet and diphthong inventory, explicit articulatory features, explainable segment comparisons, cluster inspection, written-nucleus analysis, syllabification, stress, and pronunciation-realization planning.

The next development version is 0.2.0. Until it is published, use a local
path dependency on this crate; the version below describes the upcoming release.

```toml
[dependencies]
bauske-phonology = "0.2.0"
```

Requires Rust 1.98 or newer. The categorical distance model compares consonants with consonants and vowels with vowels; perceptual calibration and whole-word rhyme scoring remain research questions. Source limitations and unresolved analyses are reported explicitly.

See the [API documentation](https://docs.rs/bauske-phonology) for examples. Licensed under Apache-2.0.
