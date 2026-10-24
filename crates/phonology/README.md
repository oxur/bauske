# bauske-phonology

This checkout is on the 0.3.0 development cycle. The dependency and installation
examples below target the published 0.2.0 release.

Source-grounded Relojban phonology for the [Bauske](https://github.com/oxur/bauske) linguistic workbench.

The library provides an alphabet and diphthong inventory, explicit articulatory features, explainable segment comparisons, cluster inspection, written-nucleus analysis, syllabification, stress, and pronunciation-realization planning.

```toml
[dependencies]
bauske-phonology = "0.2.0"
```

Requires Rust 1.98 or newer. The categorical distance model compares consonants with consonants and vowels with vowels; perceptual calibration and whole-word rhyme scoring remain research questions. Source limitations and unresolved analyses are reported explicitly.

See the [API documentation](https://docs.rs/bauske-phonology) for examples. Licensed under Apache-2.0.
