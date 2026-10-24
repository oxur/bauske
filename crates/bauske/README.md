# Bauske

An umbrella library for [Bauske](https://github.com/oxur/bauske), a (Re)Lojban
linguistic workbench. It re-exports the phonology library as `bauske::phonology`.

## Component crates

| Crate | Role | Documentation |
| --- | --- | --- |
| [bauske-phonology](https://crates.io/crates/bauske-phonology) | Inventories, articulatory features, explainable segment comparisons, clusters, prosody, and pronunciation realization. | [API docs](https://docs.rs/bauske-phonology) |
| [bauske-cli](https://crates.io/crates/bauske-cli) | The separately installed `bauske` command-line tool. | [API docs](https://docs.rs/bauske-cli) and [CLI guide](https://github.com/oxur/bauske/tree/main/crates/cli) |

Use this crate for the umbrella namespace, or depend on a component crate
directly. The CLI package is installed separately from the library.

## Use as a library

```toml
[dependencies]
bauske = "0.2.0"
```

## Example

```rust
use bauske::phonology::{DistanceModel, Segment};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let comparison = DistanceModel::default().compare(
        "p".parse::<Segment>()?,
        "b".parse::<Segment>()?,
    )?;
    assert_eq!(comparison.total(), 1);
    Ok(())
}
```

The categorical distance model is not perceptually calibrated and does not
provide a whole-word rhyme score. Source limitations and unresolved analyses
remain explicit in the underlying APIs.

Requires Rust 1.98 or newer. Licensed under Apache-2.0.
