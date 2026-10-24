# bauske-cli

The `bauske` command for exploring [Bauske](https://github.com/oxur/bauske)'s phonology PoC.

For a source checkout of 0.2.0, run these commands from the workspace root:

```sh
make build
./bin/bauske inventory
./bin/bauske compare p b
./bin/bauske prosody klama --kind brivla
```

Use `make build-release` for an optimized build, or override `INSTALL_DIR` to
choose where the binary is copied. To install the published command, run
`cargo install bauske-cli --version 0.2.0 --locked`. Version 0.1.0 used
the previous `relojban` command.

Requires Rust 1.98 or newer. Commands cover inventory, pronunciation profiles, segment comparisons, word and cluster inspection, and prosody. Run `bauske --help` for the full interface.

The model uses categorical articulatory features; its scores are not perceptually calibrated. Licensed under Apache-2.0.
