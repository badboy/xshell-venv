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

    let _ = venv.run(CODE)?;

    Ok(())
}
