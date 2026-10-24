# AGENTS.md

This file provides guidance to LLM-based coding agents when working with code in this repository.

## Project Overview

Bauske is a Rust workspace for (Re)Lojban phonology, morphology, and computational
linguistics. Its current foundation covers alphabet and diphthong inventories,
articulatory features, explainable segment comparisons, consonant clusters,
written nuclei, syllabification, stress, and pronunciation-realization planning.
The `bauske` CLI exposes these capabilities for exploration.

The workspace is at version 0.2.0, uses Rust edition 2024, and
requires Rust 1.98 or newer. Development commands and CI select Rust 1.98.0.
Supported platforms are macOS and Linux.

Keep linguistic source claims, modelling choices, and empirical findings distinct.
The current categorical distance model is not perceptually calibrated and does
not provide a whole-word rhyme score. Preserve explicit source limitations and
unresolved analyses rather than silently choosing an interpretation.

## Design and Project Planning Documents

- Start with `README.md`, the crate READMEs, and public API documentation.
- Planning belongs on the separate orphan `planning` branch in
  `.worktrees/planning/`. The existing planning worktree currently has no tracked
  plans; do not invent a project or slice history to fill that gap.
- `make planning` fetches or creates the planning worktree. It can create and push
  an empty orphan branch when the remote branch is missing; inspect
  `git worktree list` and reuse the existing checkout before invoking it.
- Keep implementation and user documentation on the source branch. Keep plans,
  ledgers, assignments, and verification records in the planning worktree, following
  its own instructions when present.
- Vlajmaji is a separate sibling repository and consumes `bauske-phonology`.
  Inspect the relevant consumer contract before changing public types, identifiers,
  or linguistic behavior shared with that project.

## Crate Architecture

| Package | Path | Responsibility |
| --- | --- | --- |
| `bauske` | `crates/bauske` | Umbrella library re-exporting `bauske-phonology` as `phonology`; its README also supplies its rustdoc landing page. |
| `bauske-phonology` | `crates/phonology` | Reusable phonological inventories, features, comparisons, word/cluster analysis, prosody, and realization plans. |
| `bauske-cli` | `crates/cli` | The `bauske` binary: Clap arguments, command dispatch, rendering, and process-level tests. |

`crates/phonology/src/lib.rs` re-exports the public API and declares `SOURCE_ID`
and `PROFILE_ID`. Keep source provenance and profile semantics explicit when
changing those contracts. Put linguistic behavior in the library and retain the
CLI's separation between analysis, rendering, and process entrypoint.

Use absolute crates.io, docs.rs, and repository links in the umbrella README so
they work in the packaged README and generated rustdoc. The CLI binary has
`doc = false` to avoid overwriting the umbrella library's `bauske` docs; the
`bauske-cli` library remains documented.

Integration tests live in each crate's `tests/` directory. The independent
`tests/test-build-tools.sh` suite exercises Make and the planning helper with
disposable local Git fixtures and a Cargo stub.

## Build & Test Commands

Run `make` or `make help` from the repository root to discover available targets.
Make uses `RUST_TOOLCHAIN=1.98.0` by default; override it explicitly when validating
another supported toolchain. The helper scripts require Bash 3.2 or newer and Git.

Always run `make format` after changes, then `make lint` before testing.
Before committing, always run `make check`.

| Command | Purpose |
| --- | --- |
| `make build` | Build the workspace and copy the debug `bauske` binary to `./bin`. |
| `make build-release` | Build and install the optimized binary. |
| `make format` / `make fmt-check` | Format Rust / check formatting without modifying files. |
| `make lint` | Run workspace Clippy across all targets with warnings denied. |
| `make test` | Run locked workspace tests with default features. |
| `make test-no-default` | Run locked workspace tests without default features. |
| `make test-all-features` | Run locked workspace tests with all features. |
| `make test-tools` | Run shell syntax checks and the isolated build-helper suite. |
| `make docs` | Build workspace API documentation without dependency docs. |
| `make check` | Check tools, formatting, Clippy, default tests, docs, and build helpers. |

For a focused Cargo command, retain the selected toolchain and lockfile, for
example `cargo +1.98.0 test --workspace --locked`.
CI is defined in `.github/workflows/ci.yml`; it tests macOS and Linux and excludes
the document-only `planning` branch. Packaging and publication targets are
separate from `make check`; publishing requires an explicit release request.

## Conventions

- Test naming: `test_<fn>_<scenario>_<expectation>` for new tests.
- Load `/rust-guidelines` before writing Rust code, including its anti-patterns guidance.
- Honor workspace lints: unsafe code is forbidden and missing documentation is
  warned about. Document public APIs and retain meaningful error distinctions.
- Preserve the distinctions between alphabet symbols, comparable segments,
  written nuclei, syllable analysis, and pronunciation realization.
- Preserve source offsets, diagnostics, source/profile identifiers, and
  explainable feature contributions when changing analyses.
- Logging via the `twyg` crate, config via the `confyg` crate when those
  capabilities are introduced; neither is currently a workspace dependency.
- Keep shell helpers compatible with Bash 3.2 and test failures as well as success.
- Keep changes scoped to the task and preserve unrelated working-tree changes.

## Commit Message Guidance

Commit only the intended files, using explicit paths and `git commit --only`
when other work may be present. Every assistant-authored commit message must
include these trailers:

```text
Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>
```

## Workbench

`workbench/` is reserved for operator scratch and temporary files. Keep it
excluded from version control and verify exclusion with `git check-ignore`
before writing scratch files there; use a temporary directory outside the
repository when no ignore rule is available. Keep durable project evidence in
the planning worktree rather than relying on scratch files.
