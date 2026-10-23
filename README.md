# Bauske

(Re)Lojban libraries and tools for phonology, morphology, and computational linguistics.

Bauske is a growing laboratory for asking how the language's sounds and word
structures work, and what we can make with them. It began with a poetic question:
can relationships between sounds offer a richer palette for poetry and song?
The aim is reusable Rust tools that make such questions inspectable and testable.

## Current proof of concept

The first foundation is phonological:

- An alphabet and diphthong inventory tied to a pinned Relojban grammar snapshot.
- Explicit articulatory features and explainable, configurable segment comparisons.
- Word and consonant-cluster inspection, written nuclei, syllabification, stress,
  and pronunciation-realization planning, with unresolved cases reported explicitly.
- A CLI, `relojban`, for exploring these analyses.

The distance model currently uses categorical articulatory mismatches within
consonants or within vowels. It is not a perceptually calibrated similarity
measure or a whole-word rhyme score. Morphology and broader computational
linguistics are future directions.

| Package | Role |
| --- | --- |
| [`bauske-phonology`](crates/phonology) | Inventory, features, comparisons, and prosody |
| [`bauske-cli`](crates/cli) | The `relojban` executable and its command interface |

## Try it

Use Rust 1.98.0 or newer. From this repository:

```sh
cargo +1.98.0 test --locked --workspace
cargo +1.98.0 run --locked -p bauske-cli -- inventory
cargo +1.98.0 run --locked -p bauske-cli -- compare p b
cargo +1.98.0 run --locked -p bauske-cli -- prosody klama --kind brivla
```

[Vlajmaji](https://github.com/oxur/vlajmaji) uses this phonological foundation
for dictionary searches. The repositories currently build together as sibling
checkouts; Bauske also builds on its own.

## Where this is going

We want to investigate phonological feature spaces, test sound-similarity models,
and develop tools for morphology, poetic structure, and musical text setting.
Source claims, modelling choices, and empirical findings will remain distinct
as that work grows.

These are research PoCs with evolving APIs. Packages currently have
`publish = false`; release metadata and packaging remain to be prepared for
the first 0.1.0 release.
