# Bauske

[![Build Status][gh-actions-badge]][gh-actions]
[![Tags][github-tags-badge]][github-tags]
[![][crate-badge]][crate]
[![][docs-badge]][docs]

[![Project Logo][logo]][logo-large]

*(Re)Lojban libraries and tools for phonology, morphology, and computational linguistics*

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
The Make helpers require Bash 3.2 or newer and Git; Python is not needed.
`make test-tools` runs the Bash tooling suite with disposable local Git remotes
and a Cargo stub, without building Rust or accessing the network.

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

## License

``` text
Apache License, Version 2.0
```

[//]: ---Named-Links---

[logo]: assets/images/v1-x250.png
[logo-large]: assets/images/v1-x1040.png
[gh-actions-badge]: https://github.com/oxur/bauske/actions/workflows/ci.yml/badge.svg
[gh-actions]: https://github.com/oxur/bauske/actions/workflows/ci.yml
[version]: https://github.com/oxur/bauske/blob/main/.github/workflows/ci.yml
[github-tags]: https://github.com/oxur/bauske/tags
[github-tags-badge]: https://img.shields.io/github/tag/oxur/bauske.svg
[crate]: https://crates.io/crates/bauske
[crate-badge]: https://img.shields.io/crates/v/bauske.svg
[docs]: https://docs.rs/bauske/
[docs-badge]: https://img.shields.io/badge/rust-documentation-blue.svg
