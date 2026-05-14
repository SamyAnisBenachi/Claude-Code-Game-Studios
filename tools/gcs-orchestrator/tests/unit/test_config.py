"""Tests for unified gcs.toml config loader."""
from __future__ import annotations

import os

import pytest

from gcs_orchestrator import config as cfg_module
from gcs_orchestrator.config import GcsConfig, load


def _write_toml(path, content):
    path.write_text(content, encoding="utf-8")


@pytest.fixture(autouse=True)
def _isolate_env(monkeypatch, tmp_path):
    """Clear all GCS_/OCTOGENT_ env vars so live shell state doesn't leak."""
    for k in list(os.environ.keys()):
        if k.startswith("GCS_") or k.startswith("OCTOGENT_"):
            monkeypatch.delenv(k, raising=False)
    # Also reset the codex_home so legacy file fallback uses tmp_path-equivalent
    # (each test sets its own toml; the no-file test gets a sandbox dir)
    yield


def test_defaults_when_no_file(tmp_path, monkeypatch):
    """Default config when no toml + no legacy fallback files exist."""
    p = tmp_path / "gcs.toml"
    # Empty toml that just pins codex_home so legacy fallback looks in tmp (where nothing exists)
    _write_toml(p, f'[paths]\ncodex_home = "{str(tmp_path).replace(chr(92), "/")}"\n')
    monkeypatch.setattr(cfg_module, "CONFIG_PATH_DEFAULT", p)
    cfg = load()
    assert cfg.orchestrator.mode == "channel-send"
    assert cfg.transport.ws_url == "ws://127.0.0.1:9787"
    assert cfg.reliability.receipt_ttl_days == 14


def test_invalid_mode_raises(tmp_path, monkeypatch):
    p = tmp_path / "gcs.toml"
    _write_toml(p, f'[paths]\ncodex_home = "{str(tmp_path).replace(chr(92), "/")}"\n[orchestrator]\nmode = "garbage"\n')
    monkeypatch.setattr(cfg_module, "CONFIG_PATH_DEFAULT", p)
    with pytest.raises(Exception):
        load()


def test_relay_mode_requires_session_id(tmp_path, monkeypatch):
    p = tmp_path / "gcs.toml"
    _write_toml(p, f'[paths]\ncodex_home = "{str(tmp_path).replace(chr(92), "/")}"\n[orchestrator]\nmode = "relay"\nsession_id = ""\n')
    monkeypatch.setattr(cfg_module, "CONFIG_PATH_DEFAULT", p)
    with pytest.raises(ValueError, match="session_id"):
        load()


def test_relay_mode_with_session_id_ok(tmp_path, monkeypatch):
    p = tmp_path / "gcs.toml"
    _write_toml(p, '[orchestrator]\nmode = "relay"\nsession_id = "019dddb4-1111-2222-3333-444444444444"\n')
    monkeypatch.setattr(cfg_module, "CONFIG_PATH_DEFAULT", p)
    cfg = load()
    assert cfg.orchestrator.mode == "relay"
    assert cfg.orchestrator.session_id.startswith("019dddb4")


def test_env_override_takes_precedence(tmp_path, monkeypatch):
    p = tmp_path / "gcs.toml"
    _write_toml(p, '[transport]\nws_url = "ws://from-file:9787"\n')
    monkeypatch.setattr(cfg_module, "CONFIG_PATH_DEFAULT", p)
    monkeypatch.setenv("GCS_APPSERVER_WS", "ws://from-env:7777")
    cfg = load()
    assert cfg.transport.ws_url == "ws://from-env:7777"


def test_env_override_numeric_coerced(tmp_path, monkeypatch):
    p = tmp_path / "gcs.toml"
    _write_toml(p, '[transport]\nturn_timeout_s = 300\n')
    monkeypatch.setattr(cfg_module, "CONFIG_PATH_DEFAULT", p)
    monkeypatch.setenv("GCS_TURN_TIMEOUT_S", "120")
    cfg = load()
    assert cfg.transport.turn_timeout_s == 120.0


def test_legacy_mode_file_fallback(tmp_path, monkeypatch):
    p = tmp_path / "gcs.toml"
    _write_toml(p, '[paths]\ncodex_home = "' + str(tmp_path).replace('\\', '/') + '"\n')
    monkeypatch.setattr(cfg_module, "CONFIG_PATH_DEFAULT", p)
    (tmp_path / "gcs-mode").write_text("relay", encoding="utf-8")
    (tmp_path / "gcs-orch-session-id").write_text("019aaaaa-1111-2222-3333-444444444444", encoding="utf-8")
    cfg = load()
    assert cfg.orchestrator.mode == "relay"
    assert cfg.orchestrator.session_id == "019aaaaa-1111-2222-3333-444444444444"


def test_relay_base_dir_derived_from_localappdata(monkeypatch):
    monkeypatch.setenv("LOCALAPPDATA", r"C:\fake\Local")
    cfg = GcsConfig()
    # paths.relay_base_dir is empty by default → derives
    assert "gcs-app-relay" in str(cfg.relay_base_dir())


def test_relay_base_dir_explicit_override():
    cfg = GcsConfig(paths={"relay_base_dir": "D:/explicit/relay"})  # type: ignore
    assert str(cfg.relay_base_dir()).replace("\\", "/") == "D:/explicit/relay"
