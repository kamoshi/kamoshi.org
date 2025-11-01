use std::{
    io::Write,
    process::{Command, Stdio},
};

use hauchiwa::error::RuntimeError;
use sequoia_openpgp::anyhow;

pub fn render_typst(code: &str) -> Result<String, RuntimeError> {
    let mut child = Command::new("typst")
        .arg("c")
        .arg("--format=svg")
        .arg("-")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or(anyhow::anyhow!("stdin not piped"))?;
        stdin.write_all(code.as_bytes())?;
        stdin.flush()?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        Err(anyhow::anyhow!("Typst SSR failed:\n{stderr}"))?
    }

    Ok(String::from_utf8(output.stdout)?)
}
