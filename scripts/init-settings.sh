#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" == "Darwin" ]]; then
  CONFIG_DIR="$HOME/Library/Application Support/com.maskedsyntax.VoxFlow"
else
  CONFIG_DIR="$HOME/.config/com.maskedsyntax.VoxFlow"
fi

mkdir -p "$CONFIG_DIR"
SETTINGS_FILE="$CONFIG_DIR/settings.json"

if [[ -f "$SETTINGS_FILE" ]]; then
  echo "Already exists: $SETTINGS_FILE"
  echo "Delete it first if you want to recreate it."
  exit 1
fi

cat > "$SETTINGS_FILE" <<'EOF'
{
  "schema_version": 1,
  "quality_mode": "hybrid",
  "dictation_mode": "push_to_talk",
  "hotkey": {
    "key_code": 59,
    "modifiers": 0,
    "label": "Left Control"
  },
  "microphone_device": null,
  "openai_api_key": "",
  "openai_model": "gpt-4o-mini-transcribe",
  "local_model": "tiny",
  "session_cap_seconds": 120,
  "clipboard_restore": true,
  "launch_at_login": false,
  "bar_position": "bottom_center",
  "cost_control": {
    "monthly_minute_cap": null,
    "monthly_spend_cap_usd": 5.0,
    "warn_at_percent": [50, 80, 100],
    "auto_local_after_cap": true
  },
  "privacy": {
    "save_history": true,
    "auto_delete_days": 30,
    "never_save_audio": true,
    "sensitive_app_blocklist": [
      "com.1password.1password",
      "com.apple.keychainaccess"
    ]
  },
  "crash_reporting_opt_in": false,
  "analytics_opt_in": false,
  "onboarding_complete": true,
  "beta_invite_code": "",
  "license": {
    "key": "",
    "tier": "free",
    "validated_at": null
  },
  "app_profiles": [],
  "snippets": [],
  "dictionary": [],
  "cleanup_enabled": false,
  "cleanup_prompt": "Fix punctuation and capitalization only. Keep meaning.",
  "rewrite_commands": [],
  "history_limit_free": 50
}
EOF

chmod 600 "$SETTINGS_FILE"
echo "Created: $SETTINGS_FILE"
echo ""
echo "Edit and add your OpenAI key to openai_api_key:"
echo "  open -e \"$SETTINGS_FILE\""
