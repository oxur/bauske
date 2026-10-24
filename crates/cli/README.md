# bauske-cli

The `bauske` command for exploring [Bauske](https://github.com/oxur/bauske)'s phonology PoC.

For the 0.2.0 development checkout, run these commands from the workspace root:

```sh
make build
./bin/bauske inventory
./bin/bauske compare p b
./bin/bauske prosody klama --kind brivla
```

Use `make build-release` for an optimized build, or override `INSTALL_DIR` to
choose where the binary is copied. The published 0.1.0 package still installs
the previous `relojban` command.

Requires Rust 1.98 or newer. Commands cover inventory, pronunciation profiles, segment comparisons, word and cluster inspection, and prosody. Run `bauske --help` for the full interface.

The model uses categorical articulatory features; its scores are not perceptually calibrated. Licensed under Apache-2.0.
