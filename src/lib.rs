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
//! # fn main() -> xshell_venv::Result<()> {
//! let sh = xshell::Shell::new()?;
//! let venv = VirtualEnv::new(&sh, "py3")?;
//!
//! venv.run("print('Hello World!')")?; // "Hello World!"
//! # Ok(())
//! # }
//! ```

mod error;

use std::env;
use std::path::{Path, PathBuf};

use xshell::{cmd, PushEnv, Shell};

pub use error::{Error, Result};

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
/// ```rust
/// use xshell;
/// use xshell_venv::VirtualEnv;
///
/// # fn main() -> xshell_venv::Result<()> {
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

fn guess_python(sh: &Shell) -> Result<&'static str, Error> {
    #[cfg(windows)]
    {
        if cmd!(sh, "python3.exe --version").run().is_ok() {
            return Ok("python3.exe");
        }

        if let Ok(output) = cmd!(sh, "python.exe --version").read() {
            if output.contains("Python 3.") {
                return Ok("python.exe");
            }
        }
    }

    if cmd!(sh, "python3 --version").run().is_ok() {
        return Ok("python3");
    }

    if let Ok(output) = cmd!(sh, "python --version").read() {
        if output.contains("Python 3.") {
            return Ok("python");
        }
    }

    Err("couldn't find Python 3 in $PATH".into())
}

fn create_venv(sh: &Shell, path: &Path) -> Result<(), Error> {
    let pybin = path.join("bin").join("python");
    if !pybin.exists() {
        let python = guess_python(sh)?;
        cmd!(sh, "{python} -m venv {path}").run()?;
    }
    Ok(())
}

fn find_directory(name: &str) -> PathBuf {
    let mut venv_dir = loop {
        // May be set by the user.
        if let Ok(target_dir) = env::var("CARGO_TARGET_DIR") {
            break PathBuf::from(target_dir);
        }

        // Find the `target/<arch>?/<profile> directory.`
        // `OUT_DIR` is usually something like
        // target/<arch>/debug/build/$cratename-$hash/out/,
        // so we strip out the last 3 ancestors.
        // This will be correct for plain crates, for workspaces
        // and even if the `TARGET_DIR` is not nested within the workspace.
        // Putting it there also means the venv stays available across builds.
        if let Ok(out_dir) = env::var("OUT_DIR") {
            let path = Path::new(&out_dir);
            let path = path
                .parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent());
            if let Some(out_dir) = path {
                break PathBuf::from(out_dir);
            }
        }

        // Create a `target/$venv` path next to where the project's `Cargo.toml` is located.
        // That will create an occasional `target` directory, when none existed before,
        // but I have no idea in what case `CARGO_MANIFEST_DIR` would be set
        // but `OUT_DIR` isn't.
        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            let mut p = PathBuf::from(manifest_dir);
            p.push("target");
            break p;
        }

        // As a last resort we use the host's temporary directory,
        // so something like `/tmp`.
        break env::temp_dir();
    };

    let name = format!("venv-{name}");
    venv_dir.push(&name);
    return venv_dir;
}

impl<'a> VirtualEnv<'a> {
    /// Create a Python virtual environment with the given name.
    ///
    /// This creates a new environment or reuses an existing one.
    /// Preserves the environment across calls and makes it available for all other commands
    /// within the same [`xshell::Shell`].
    ///
    /// This will try to build a path based on the following environment variables:
    ///
    /// - `CARGO_TARGET_DIR`
    /// - `OUT_DIR` 3 levels up<sup>1</sup>
    /// - `CARGO_MANIFEST_DIR`
    ///
    /// _<sup>1</sup> should usually be the crate's/workspace's target directory._
    ///
    /// If none of these are set it will use the system's temporary directory, e.g. `/tmp`.
    ///
    /// ## Example
    ///
    /// ```
    /// # use xshell;
    /// # use xshell_venv::VirtualEnv;
    /// # fn main() -> xshell_venv::Result<()> {
    /// let sh = xshell::Shell::new()?;
    /// let venv = VirtualEnv::new(&sh, "py3")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(shell: &'a Shell, name: &str) -> Result<VirtualEnv<'a>, Error> {
        let venv_dir = find_directory(name);

        Self::with_path(shell, &venv_dir)
    }

    /// Create a Python virtual environment in the given path.
    ///
    /// This creates a new environment or reuses an existing one.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use xshell;
    /// # use xshell_venv::VirtualEnv;
    /// # fn main() -> xshell_venv::Result<()> {
    /// let sh = xshell::Shell::new()?;
    ///
    /// let mut dir = std::env::temp_dir();
    /// dir.push("xshell-py3");
    /// let venv = VirtualEnv::with_path(&sh, &dir)?;
    ///
    /// let output = venv.run("print('hello python')")?;
    /// assert_eq!("hello python", output);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_path(shell: &'a Shell, venv_dir: &Path) -> Result<VirtualEnv<'a>, Error> {
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
    ///
    /// The package can be anything `pip` accepts,
    /// including specifying the version (`$name==1.0.0`)
    /// or repositories (`git+https://github.com/$name/$repo@branch#egg=$name`).
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// # use xshell;
    /// # use xshell_venv::VirtualEnv;
    /// # fn main() -> xshell_venv::Result<()> {
    /// let sh = xshell::Shell::new()?;
    /// let venv = VirtualEnv::new(&sh, "py3")?;
    ///
    /// venv.pip_install("flake8")?;
    /// let output = venv.run_module("flake8", &["--version"])?;
    /// assert!(output.contains("flake"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn pip_install(&self, package: &str) -> Result<()> {
        cmd!(self.shell, "pip install {package}").run()?;
        Ok(())
    }

    /// Upgrade a Python package in this virtual environment.
    ///
    /// The package can be anything `pip` accepts,
    /// including specifying the version (`$name==1.0.0`)
    /// or repositories (`git+https://github.com/$name/$repo@branch#egg=$name`).
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// # use xshell;
    /// # use xshell_venv::VirtualEnv;
    /// # fn main() -> xshell_venv::Result<()> {
    /// let sh = xshell::Shell::new()?;
    /// let venv = VirtualEnv::new(&sh, "py3")?;
    ///
    /// venv.pip_install("flake8==3.0.0")?;
    /// let output = venv.run_module("flake8", &["--version"])?;
    /// assert!(output.contains("3.0.0"));
    ///
    /// venv.pip_upgrade("flake8")?;
    /// let output = venv.run_module("flake8", &["--version"])?;
    /// assert!(!output.contains("3.0.0"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn pip_upgrade(&self, package: &str) -> Result<()> {
        cmd!(self.shell, "pip install --upgrade {package}").run()?;
        Ok(())
    }

    /// Run Python code in this virtual environment.
    ///
    /// Returns the code's output.
    ///
    /// ## Example
    ///
    /// ```
    /// # use xshell;
    /// # use xshell_venv::VirtualEnv;
    /// # fn main() -> xshell_venv::Result<()> {
    /// let sh = xshell::Shell::new()?;
    /// let venv = VirtualEnv::new(&sh, "py3")?;
    ///
    /// let output = venv.run("print('hello python')")?;
    /// assert_eq!("hello python", output);
    /// # Ok(())
    /// # }
    /// ```
    pub fn run(&self, code: &str) -> Result<String> {
        let py = cmd!(self.shell, "python");

        Ok(py.stdin(code).read()?)
    }

    /// Run library module as a script.
    ///
    /// This is `python -m $module`.
    /// Additional arguments are passed through as is.
    ///
    /// ## Example
    ///
    /// ```
    /// # use xshell;
    /// # use xshell_venv::VirtualEnv;
    /// # fn main() -> xshell_venv::Result<()> {
    /// let sh = xshell::Shell::new()?;
    /// let venv = VirtualEnv::new(&sh, "py3")?;
    ///
    /// let output = venv.run_module("pip", &["--version"])?;
    /// assert!(output.contains("pip"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn run_module(&self, module: &str, args: &[&str]) -> Result<String> {
        let py = cmd!(self.shell, "python -m {module} {args...}");
        Ok(py.read()?)
    }
}
