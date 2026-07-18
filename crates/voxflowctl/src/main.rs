use anyhow::{bail, Context, Result};
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use voxflow_config::{load_settings, save_settings, ProviderConfig, ProviderKind};

fn socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(runtime_dir).join("voxflow.sock")
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [cmd, action] if cmd == "trigger" => trigger(action),
        [cmd, key_ref] if cmd == "set-key" => set_key(key_ref),
        [cmd, key_ref] if cmd == "check-key" => check_key(key_ref),
        [cmd] if cmd == "configure-groq" => configure_groq(),
        _ => {
            eprintln!(
                "usage:\n  \
                 voxflowctl trigger <down|up|toggle>\n  \
                 voxflowctl configure-groq\n  \
                 voxflowctl set-key <key-ref>   (prompts for the secret, hidden input)"
            );
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

/// Points both transcription and rewrite at Groq, sharing one keyring
/// entry ("groq") since a single Groq API key covers both endpoints.
/// Run `voxflowctl set-key groq` afterward to actually store the key.
fn configure_groq() -> Result<()> {
    let mut settings = load_settings().context("loading settings")?;

    settings.transcription_provider = ProviderConfig {
        kind: ProviderKind::Groq,
        base_url: None,
        model: "whisper-large-v3-turbo".into(),
        accurate_model: Some("whisper-large-v3".into()),
        api_key_ref: Some("groq".into()),
    };
    settings.rewrite_provider = ProviderConfig {
        kind: ProviderKind::Groq,
        base_url: None,
        model: "llama-3.1-8b-instant".into(),
        accurate_model: None,
        api_key_ref: Some("groq".into()),
    };

    save_settings(&settings).context("saving settings")?;
    println!("settings updated: transcription + rewrite both point at Groq (api_key_ref=\"groq\")");
    println!("now run: voxflowctl set-key groq");
    Ok(())
}

/// Prompts for a secret with echo disabled and stores it in the OS
/// keyring under `key_ref`. Run this in your own terminal, not through an
/// AI assistant's `!` passthrough — that would put the key in the
/// conversation transcript.
fn set_key(key_ref: &str) -> Result<()> {
    let secret = rpassword::prompt_password(format!("Secret value for \"{key_ref}\": "))
        .context("reading secret")?;
    if secret.trim().is_empty() {
        bail!("empty secret, not storing anything");
    }
    voxflow_secrets::set_secret(key_ref, secret.trim()).context("storing secret in keyring")?;
    println!("stored under keyring ref \"{key_ref}\"");
    Ok(())
}

/// Confirms a secret is present without ever printing its value.
fn check_key(key_ref: &str) -> Result<()> {
    match voxflow_secrets::get_secret(key_ref).context("reading secret from keyring")? {
        Some(value) => println!("\"{key_ref}\" is set ({} chars)", value.len()),
        None => println!("\"{key_ref}\" is NOT set"),
    }
    Ok(())
}
