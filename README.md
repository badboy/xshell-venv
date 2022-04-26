# `xshell-venv` - manage Python virtual environments in code.

`xshell-env` is an extension to [xshell], the swiss-army knife for writing cross-platform “bash” scripts in Rust.

[xshell]: https://docs.rs/xshell/

## Example

```rust
use xshell;
use xshell_venv::VirtualEnv;

let sh = xshell::Shell::new()?;
let venv = VirtualEnv::new(&sh, "py3")?;

venv.run("print('Hello World!')")?; // "Hello World!"
```
