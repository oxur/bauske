# bauske
(Re)Lojban libraries and tools for the study of its phonems, morphemes, and general computational linguistics


# Bauske

This repository contains the migrated Relojban phonology library and command-line tools. The migration changes Rust package and library identities while preserving the `relojban` executable and phonological behavior.

Build and test with the declared Rust 1.98 minimum:

```sh
cargo +1.98.0 build --workspace --locked
cargo +1.98.0 test --workspace --locked
cargo +1.98.0 run --locked -p bauske-cli -- inventory
```

The source package/path map and provenance are recorded in the Relojban research checkout at `research/bauske-migration.md`. The pre-existing destination README opening and repository notices are retained. This migration does not make a project-wide source-license selection.
