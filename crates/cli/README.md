# bauske-cli

The `relojban` command for exploring [Bauske](https://github.com/oxur/bauske)'s phonology PoC.

```sh
cargo install bauske-cli --version 0.1.0 --locked
relojban inventory
relojban compare p b
relojban prosody klama --kind brivla
```

Requires Rust 1.98 or newer. Commands cover inventory, pronunciation profiles, segment comparisons, word and cluster inspection, and prosody. Run `relojban --help` for the full interface.

The model uses categorical articulatory features; its scores are not perceptually calibrated. Licensed under Apache-2.0.
