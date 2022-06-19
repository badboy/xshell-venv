# `xshell-venv`

[![crates.io](https://img.shields.io/crates/v/xshell-venv.svg?style=flat-square)](https://crates.io/crates/xshell-venv)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/xshell-venv)
[![License: MIT](https://img.shields.io/github/license/badboy/xshell-venv?style=flat-square)](LICENSE)
[![Build Status](https://img.shields.io/github/workflow/status/badboy/xshell-venv/Test/main?style=flat-square)](https://github.com/badboy/xshell-venv/actions/workflows/test.yml)

`xshell-venv` manages your Python virtual environments in code.

`xshell-venv` is an extension to [xshell], the swiss-army knife for writing cross-platform “bash” scripts in Rust.

[xshell]: https://docs.rs/xshell/

## Example

```rust
use xshell_venv::{Shell, VirtualEnv};

let sh = Shell::new()?;
let venv = VirtualEnv::new(&sh, "py3")?;

venv.run("print('Hello World!')")?; // "Hello World!"
```

## License

[MIT](LICENSE).
