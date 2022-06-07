use xshell::{Result, Shell};
use xshell_venv::VirtualEnv;

const CODE: &str = r#"
import os
from pathlib import Path

out_dir = Path(os.environ["OUT_DIR"])
fp = out_dir / "python.rs"

with open(fp, "w") as f:
    f.write("const MSG: &str = \"hello from python\";");
"#;

fn main() -> Result<()> {
    let sh = Shell::new()?;
    let venv = VirtualEnv::new(&sh, "py3")?;

    // You can run individual modules (`python -m $module`).
    // To demonstrate we call `pip`, which should exist.
    let pip_version = venv.run_module("pip", &["--version"])?;
    assert!(pip_version.contains("pip "));

    // We can run code as well easily.
    // These are run as adhoc scripts,
    // passing everything in stdin.
    let _ = venv.run(CODE)?;

    Ok(())
}
