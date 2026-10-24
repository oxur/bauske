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
- A CLI, `bauske`, for exploring these analyses.

The distance model currently uses categorical articulatory mismatches within
consonants or within vowels. It is not a perceptually calibrated similarity
measure or a whole-word rhyme score. Morphology and broader computational
linguistics are future directions.

| Package | Role |
| --- | --- |
| [`bauske-phonology`](crates/phonology) | Inventory, features, comparisons, and prosody |
| [`bauske-cli`](crates/cli) | The `bauske` executable and its command interface |

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

The workspace is on the **0.2.0 development cycle**; its APIs are experimental.
The latest published release is 0.1.0, whose CLI command is still `relojban`.

## Development and releases

Run `make help` for build, test, lint, documentation, and publishing targets.
`make check` runs the core quality gates.

`make build` builds and copies `bauske` to `./bin`; `make build-release` does
the same for an optimized build. Set `INSTALL_DIR=./local-bin` to choose another
destination. To copy an existing build, run `make install` (or
`make install MODE=release` for release artifacts).

`make planning` sets up `.worktrees/planning` on the separate `planning` branch.
It fetches an existing branch from `origin`, or creates and pushes an empty
orphan branch if none exists. Later updates are fast-forward only; local
planning work is preserved.

Use `make publish-dry-run` to verify the workspace packages and `make publish`
to upload them to crates.io. The packaged crates include their own READMEs and licenses.
