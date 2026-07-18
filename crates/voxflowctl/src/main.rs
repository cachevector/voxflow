use anyhow::{bail, Context, Result};
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

fn socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(runtime_dir).join("voxflow.sock")
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [cmd, action] if cmd == "trigger" => trigger(action),
        _ => {
            eprintln!("usage: voxflowctl trigger <down|up|toggle>");
            std::process::exit(2);
        }
    }
}

fn trigger(action: &str) -> Result<()> {
    if !matches!(action, "down" | "up" | "toggle") {
        bail!("unknown trigger action: {action} (expected down|up|toggle)");
    }
    let path = socket_path();
    let mut stream = UnixStream::connect(&path)
        .with_context(|| format!("connect to {} (is voxflow-app running?)", path.display()))?;
    writeln!(stream, "{action}").context("write to voxflow socket")?;
    Ok(())
}
