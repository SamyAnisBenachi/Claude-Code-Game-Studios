"""Tests for the relay daemon framing + health endpoint."""
from __future__ import annotations

import json
import socket
import struct
import threading
import time

from gcs_orchestrator import daemon


def test_frame_roundtrip():
    """Frame read/write should be exact-byte symmetric."""
    a, b = socket.socketpair()
    payload = {"hello": "world", "n": 42}
    daemon._write_frame(a, payload)
    a.close()
    got = daemon._read_frame(b)
    b.close()
    assert got == payload


def test_frame_handles_large_payload():
    a, b = socket.socketpair()
    payload = {"data": "x" * (100_000)}
    daemon._write_frame(a, payload)
    a.close()
    got = daemon._read_frame(b)
    b.close()
    assert got["data"] == "x" * 100_000


def test_frame_rejects_oversize():
    a, b = socket.socketpair()
    # Manually write a header claiming > 10 MB
    a.sendall(struct.pack(">I", daemon._FRAME_MAX_BYTES + 1))
    a.sendall(b"a" * 100)
    a.close()
    got = daemon._read_frame(b)
    b.close()
    assert got is None  # rejected


def test_frame_handles_truncated_socket():
    a, b = socket.socketpair()
    # Send incomplete header
    a.sendall(b"\x00\x00")
    a.close()
    got = daemon._read_frame(b)
    b.close()
    assert got is None
