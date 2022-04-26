//! xshell-venv manages your Python virtual environments in code.
//!
//! This is an extension to [xshell], the swiss-army knife for writing cross-platform “bash” scripts in Rust.
//!
//! [xshell]: https://docs.rs/xshell/
//!
//! ## Example
//!
//! ```rust
//! use xshell;
//! use xshell_venv::VirtualEnv;
//!
//! # fn main() -> xshell::Result<()> {
//! let sh = xshell::Shell::new()?;
//! let venv = VirtualEnv::new(&sh, "py3")?;
//!
//! venv.run("print('Hello World!')")?; // "Hello World!"
//! # Ok(())
//! # }
//! ```

use std::env;
use std::path::{Path, PathBuf};

use xshell::{cmd, PushEnv, Result, Shell};

/// A Python virtual environment.
///
///
/// This creates or re-uses a virtual environment.
/// All Python invocations in this environment will have access to the environment's code,
/// including installed libraries and packages.
///
/// Use [`VirtualEnv::new`] to create a new environment.
///
/// The virtual environment gets deactivated on `Drop`.
///
/// ## Example
///
/// ```rust,no_run
/// use xshell;
/// use xshell_venv::VirtualEnv;
///
/// # fn main() -> xshell::Result<()> {
/// let sh = xshell::Shell::new()?;
/// let venv = VirtualEnv::new(&sh, "py3")?;
///
/// venv.run("print('Hello World!')")?; // "Hello World!"
/// # Ok(())
/// # }
/// ```
pub struct VirtualEnv<'a> {
    shell: &'a Shell,
    _env: Vec<PushEnv<'a>>,
}

fn create_venv(sh: &Shell, path: &Path) -> Result<()> {
    let pybin = path.join("bin").join("python");
    if !pybin.exists() {
        cmd!(sh, "python -m venv {path}").run()?;
    }
    Ok(())
}

fn find_directory(name: &str) -> PathBuf {
    let mut venv_dir = loop {
        if let Ok(target_dir) = env::var("CARGO_TARGET_DIR") {
            break PathBuf::from(target_dir);
        }

        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            let mut p = PathBuf::from(manifest_dir);
            p.push("target");
            break p;
        }

        if let Ok(out_dir) = env::var("OUT_DIR") {
            break PathBuf::from(out_dir);
        }

        break PathBuf::from("/tmp");
    };

    let name = format!("venv-{name}");
    venv_dir.push(&name);
    return venv_dir;
}

impl<'a> VirtualEnv<'a> {
    /// Create a Python virtual environment with the given name.
    ///
    /// This creates a new environment or reuses an existing one.
    ///
    /// This will try to build a path based on the following environment variables:
    ///
    /// - `CARGO_TARGET_DIR`
    /// - `CARGO_MANIFEST_DIR`
    /// - `OUT_DIR`
    ///
    /// If none of these are set it will use `/tmp`.
    pub fn new(shell: &'a Shell, name: &str) -> Result<VirtualEnv<'a>> {
        let venv_dir = find_directory(name);

        Self::with_path(shell, &venv_dir)
    }

    /// Create a Python virtual environment in the given path.
    ///
    /// This creates a new environment or reuses an existing one.
    pub fn with_path(shell: &'a Shell, venv_dir: &Path) -> Result<VirtualEnv<'a>> {
        create_venv(shell, venv_dir)?;

        let path = env::var("PATH").unwrap_or_else(|_| "/bin:/usr/bin".to_string());
        let path = format!("{}/bin:{}", venv_dir.display(), path);

        let mut env = vec![];
        env.push(shell.push_env("VIRTUAL_ENV", format!("{}", venv_dir.display())));
        env.push(shell.push_env("PATH", path));
        env.push(shell.push_env("PYTHONHOME", ""));

        Ok(VirtualEnv { shell, _env: env })
    }

    /// Install a Python package in this virtual environment.
    pub fn pip_install(&self, package: &str) -> Result<()> {
        cmd!(self.shell, "pip install {package}").run()?;
        Ok(())
    }

    /// Run Python code in this virtual environment.
    ///
    /// Returns the code's output.
    pub fn run(&self, code: &str) -> Result<String> {
        let py = cmd!(self.shell, "python");

        py.stdin(code).read()
    }
}
