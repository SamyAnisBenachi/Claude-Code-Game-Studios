"""Unified configuration for gcs-orchestrator.

Single source of truth: `~/.codex/gcs.toml`. Validated with pydantic at boot
of every entry point. Env vars override file values for ops convenience.

Schema:

    [orchestrator]
    mode = "relay" | "channel-send"   # default channel-send (legacy fallback)
    session_id = "uuid-here"          # required when mode=relay
    cwd_override = "D:/path"          # optional, applied at thread/resume + turn/start

    [transport]
    ws_url = "ws://127.0.0.1:9787"
    readyz_url = "http://127.0.0.1:9787/readyz"
    handshake_timeout_s = 120.0
    turn_timeout_s = 600.0

    [octogent]
    port = 8787
    parent_terminal_id = "codex-orchestrator-main"
    tentacle_id = "gcs-orchestrator"

    [paths]
    codex_home = "C:/Users/Sam/.codex"
    relay_base_dir = ""               # default %LOCALAPPDATA%/gcs-app-relay
    sounds_dir = "C:/Users/Sam/.claude/sounds"

    [pin]
    codex_version = "0.130.0"         # required exact match; warn-fail at boot
    codex_bin = "C:/Users/Sam/.codex/bin-pinned/codex.exe"

    [reliability]
    ws_ping_interval_s = 20
    ws_pong_timeout_s = 10
    receipt_ttl_days = 14
    receipt_max_count = 500

    [backup]
    enabled = true
    target_dir = "D:/_DEV/Work/Claude-Code-Game-Studios/reports/.backups/auto-daily"
    gfs_daily = 7
    gfs_weekly = 4
    gfs_monthly = 12

Env-var overrides take precedence:
    GCS_MODE, GCS_ORCH_SESSION_ID, GCS_APPSERVER_WS, GCS_TURN_TIMEOUT_S, …
"""
from __future__ import annotations

import os
import sys
from pathlib import Path
from typing import Optional

from pydantic import BaseModel, Field, field_validator

try:
    import tomllib  # py311+
except ImportError:  # pragma: no cover
    import tomli as tomllib


CONFIG_PATH_DEFAULT = Path("C:/Users/Sam/.codex/gcs.toml")


class OrchestratorSection(BaseModel):
    mode: str = "channel-send"
    session_id: str = ""
    cwd_override: str = ""

    @field_validator("mode")
    @classmethod
    def _check_mode(cls, v: str) -> str:
        if v not in {"relay", "channel-send"}:
            raise ValueError(f"orchestrator.mode must be 'relay' or 'channel-send', got {v!r}")
        return v


class TransportSection(BaseModel):
    ws_url: str = "ws://127.0.0.1:9787"
    readyz_url: str = "http://127.0.0.1:9787/readyz"
    handshake_timeout_s: float = 120.0
    turn_timeout_s: float = 600.0


class OctogentSection(BaseModel):
    port: int = 8787
    parent_terminal_id: str = "codex-orchestrator-main"
    tentacle_id: str = "gcs-orchestrator"


class PathsSection(BaseModel):
    codex_home: str = "C:/Users/Sam/.codex"
    relay_base_dir: str = ""  # empty = derive from %LOCALAPPDATA%
    sounds_dir: str = "C:/Users/Sam/.claude/sounds"


class PinSection(BaseModel):
    codex_version: str = "0.130.0"
    codex_bin: str = "C:/Users/Sam/.codex/bin-pinned/codex.exe"


class ReliabilitySection(BaseModel):
    ws_ping_interval_s: int = 20
    ws_pong_timeout_s: int = 10
    receipt_ttl_days: int = 14
    receipt_max_count: int = 500


class BackupSection(BaseModel):
    enabled: bool = True
    target_dir: str = "D:/_DEV/Work/Claude-Code-Game-Studios/reports/.backups/auto-daily"
    gfs_daily: int = 7
    gfs_weekly: int = 4
    gfs_monthly: int = 12


class GcsConfig(BaseModel):
    orchestrator: OrchestratorSection = Field(default_factory=OrchestratorSection)
    transport: TransportSection = Field(default_factory=TransportSection)
    octogent: OctogentSection = Field(default_factory=OctogentSection)
    paths: PathsSection = Field(default_factory=PathsSection)
    pin: PinSection = Field(default_factory=PinSection)
    reliability: ReliabilitySection = Field(default_factory=ReliabilitySection)
    backup: BackupSection = Field(default_factory=BackupSection)

    def relay_base_dir(self) -> Path:
        """Resolve the relay base directory (lock + receipts + logs)."""
        if self.paths.relay_base_dir:
            return Path(self.paths.relay_base_dir)
        local = os.environ.get("LOCALAPPDATA") or os.path.expanduser("~/.codex")
        return Path(local) / "gcs-app-relay"


# Env-var overrides table. Each maps to a dotted config path.
_ENV_OVERRIDES = {
    "GCS_MODE": "orchestrator.mode",
    "GCS_ORCH_SESSION_ID": "orchestrator.session_id",
    "GCS_ORCH_CWD": "orchestrator.cwd_override",
    "GCS_APPSERVER_WS": "transport.ws_url",
    "GCS_APPSERVER_READYZ": "transport.readyz_url",
    "GCS_HANDSHAKE_TIMEOUT_S": "transport.handshake_timeout_s",
    "GCS_TURN_TIMEOUT_S": "transport.turn_timeout_s",
    "OCTOGENT_PORT": "octogent.port",
    "GCS_CODEX_BIN": "pin.codex_bin",
    "GCS_CODEX_PINNED_VERSION": "pin.codex_version",
}


def _apply_env_overrides(data: dict) -> dict:
    """Apply env-var overrides onto the loaded TOML dict."""
    for env_key, dotted in _ENV_OVERRIDES.items():
        val = os.environ.get(env_key)
        if val is None or val == "":
            continue
        section, _, key = dotted.partition(".")
        data.setdefault(section, {})
        # Numeric coercion for known numeric fields
        if key.endswith("_s") or key in {"port", "ws_ping_interval_s", "ws_pong_timeout_s",
                                          "receipt_ttl_days", "receipt_max_count"}:
            try:
                data[section][key] = float(val) if "." in val or "_s" in key else int(val)
            except ValueError:
                data[section][key] = val
        else:
            data[section][key] = val
    return data


def _load_legacy_toggle_files(data: dict, paths_codex_home: Path) -> dict:
    """Backward-compat: read ~/.codex/gcs-mode and gcs-orch-session-id if set.

    These were the original toggle mechanism before gcs.toml. If gcs.toml
    already specifies orchestrator.mode/session_id, those win. Otherwise
    the legacy files are honored so existing setups keep working without
    touching gcs.toml.
    """
    legacy_mode_file = paths_codex_home / "gcs-mode"
    legacy_sid_file = paths_codex_home / "gcs-orch-session-id"
    if legacy_mode_file.exists() and "orchestrator" not in data:
        data.setdefault("orchestrator", {})
    if "orchestrator" in data:
        if not data["orchestrator"].get("mode") and legacy_mode_file.exists():
            try:
                data["orchestrator"]["mode"] = legacy_mode_file.read_text(encoding="utf-8").strip()
            except OSError:
                pass
        if not data["orchestrator"].get("session_id") and legacy_sid_file.exists():
            try:
                data["orchestrator"]["session_id"] = legacy_sid_file.read_text(encoding="utf-8").strip()
            except OSError:
                pass
    return data


def load(path: Optional[Path] = None) -> GcsConfig:
    """Load + validate gcs-orchestrator configuration.

    Resolution order:
    1. If `path` is given, load it (must exist).
    2. Else load `~/.codex/gcs.toml` if it exists.
    3. Else use defaults.
    4. Apply env-var overrides.
    5. Apply legacy toggle files (gcs-mode / gcs-orch-session-id) as
       last-resort fallback for fields not yet set.
    6. Validate with pydantic; raise ValidationError on bad data.
    """
    if path is None:
        path = CONFIG_PATH_DEFAULT

    data: dict = {}
    if path.exists():
        with path.open("rb") as fh:
            data = tomllib.load(fh)

    data = _apply_env_overrides(data)

    # Resolve codex_home before legacy lookup
    codex_home = Path(data.get("paths", {}).get("codex_home", "C:/Users/Sam/.codex"))
    data = _load_legacy_toggle_files(data, codex_home)

    cfg = GcsConfig.model_validate(data)

    # Cross-field: if mode=relay, session_id must be non-empty
    if cfg.orchestrator.mode == "relay" and not cfg.orchestrator.session_id:
        raise ValueError(
            "orchestrator.mode = 'relay' requires orchestrator.session_id "
            "(set in gcs.toml, GCS_ORCH_SESSION_ID env, or ~/.codex/gcs-orch-session-id legacy file)"
        )

    return cfg


def cli() -> int:
    """`gcs-config` console script — print resolved config."""
    import json
    try:
        cfg = load()
        print(json.dumps(cfg.model_dump(), indent=2))
        return 0
    except Exception as exc:
        sys.stderr.write(f"config load failed: {exc!r}\n")
        return 1


if __name__ == "__main__":
    sys.exit(cli())
