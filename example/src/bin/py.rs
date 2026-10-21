use std::env;
use std::path::PathBuf;
use xshell_venv::{Result, Shell, VirtualEnv};

const CODE: &str = r#"
print("Hello Python World!")
"#;

fn main() -> Result<()> {
    let sh = Shell::new()?;

    let venv = if let Ok(venv_dir) = env::var("VENV_DIR") {
        let venv_dir = PathBuf::from(venv_dir);
        VirtualEnv::with_path(&sh, &venv_dir)?
    } else {
        VirtualEnv::new(&sh, "py3")?
    };

    // You can run individual modules (`python -m $module`).
    // To demonstrate we call `pip`, which should exist.
    let pip_version = venv.run_module("pip", &["--version"])?;
    assert!(
        pip_version.contains("pip "),
        "Expected `pip` in the output. Got: {}",
        pip_version
    );

    let out = venv.run(CODE)?;
    println!("{out}");
    Ok(())
}
