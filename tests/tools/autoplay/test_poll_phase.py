"""Focused tests for the poll_phase pseudo-action (PROMPT 2020).

Covers:
  - RecipeBuilder.poll_phase: emitted action shape, params, tick advancement
  - _poll_for_phase driver helper: successful poll, timeout behavior, checkpoint rows
  - Serialisation/static compatibility: local.poll_phase in allowlists, JSON round-trip

Run with:
    pytest tests/tools/autoplay/test_poll_phase.py -v
"""
from __future__ import annotations

import json
import sys
import time
from pathlib import Path

# Make ``tools/autoplay`` importable without installing the package.
_TOOLS_AUTOPLAY = Path(__file__).resolve().parents[3] / "tools" / "autoplay"
if str(_TOOLS_AUTOPLAY) not in sys.path:
    sys.path.insert(0, str(_TOOLS_AUTOPLAY))

import pytest

from recipes._builder import RecipeBuilder  # noqa: E402
import driver as _driver  # noqa: E402


# ---------------------------------------------------------------------------
# 1. Builder tests
# ---------------------------------------------------------------------------


class TestPollPhaseBuilder:
    def _builder(self) -> RecipeBuilder:
        return RecipeBuilder(window_size=(1280.0, 720.0), start_tick=1)

    def test_poll_phase_emits_local_poll_phase(self):
        b = self._builder()
        b.poll_phase("shop-phase")
        actions = b.build()
        methods = [a["method"] for a in actions]
        assert "local.poll_phase" in methods

    def test_poll_phase_params_contain_label(self):
        b = self._builder()
        b.poll_phase("draft-phase")
        action = b.build()[0]
        assert action["params"]["label"] == "draft-phase"

    def test_poll_phase_params_contain_max_ticks(self):
        b = self._builder()
        b.poll_phase("draft-phase", max_ticks=50)
        action = b.build()[0]
        assert action["params"]["max_ticks"] == 50

    def test_poll_phase_default_max_ticks_is_30(self):
        b = self._builder()
        b.poll_phase("any-phase")
        action = b.build()[0]
        assert action["params"]["max_ticks"] == 30

    def test_poll_phase_max_ticks_stored_as_int(self):
        b = self._builder()
        b.poll_phase("any-phase", max_ticks=15)
        action = b.build()[0]
        assert isinstance(action["params"]["max_ticks"], int)

    def test_poll_phase_tick_advances_by_one(self):
        b = self._builder()
        tick_before = b.tick
        b.poll_phase("some-phase")
        assert b.tick == tick_before + 1

    def test_poll_phase_action_emitted_at_current_tick(self):
        b = self._builder()
        b.wait(5)
        expected_tick = b.tick
        b.poll_phase("some-phase")
        action = b.build()[-1]
        assert action["tick"] == expected_tick

    def test_poll_phase_action_has_tick_method_params_keys(self):
        b = self._builder()
        b.poll_phase("check-phase")
        action = b.build()[0]
        assert "tick" in action
        assert "method" in action
        assert "params" in action

    def test_poll_phase_tick_is_positive_int(self):
        b = self._builder()
        b.poll_phase("check-phase")
        action = b.build()[0]
        assert isinstance(action["tick"], int)
        assert action["tick"] >= 1

    def test_poll_phase_returns_builder_for_chaining(self):
        b = self._builder()
        result = b.poll_phase("check-phase")
        assert result is b

    def test_poll_phase_chained_after_other_primitives(self):
        b = self._builder()
        b.wait(3).poll_phase("lobby-phase").wait(2)
        actions = b.build()
        labels = [a["params"]["label"] for a in actions if a["method"] == "local.poll_phase"]
        assert labels == ["lobby-phase"]

    def test_poll_phase_action_serialises_to_json(self):
        b = self._builder()
        b.poll_phase("phase-x", max_ticks=10)
        action = b.build()[0]
        serialised = json.dumps(action)
        roundtrip = json.loads(serialised)
        assert roundtrip["method"] == "local.poll_phase"
        assert roundtrip["params"]["label"] == "phase-x"
        assert roundtrip["params"]["max_ticks"] == 10


# ---------------------------------------------------------------------------
# 2. Driver allowlist / structural tests
# ---------------------------------------------------------------------------


class TestPollPhaseDriverStructure:
    def test_local_poll_phase_in_driver_local_methods(self):
        assert "local.poll_phase" in _driver.LOCAL_METHODS

    def test_poll_for_phase_function_exists(self):
        assert hasattr(_driver, "_poll_for_phase")
        assert callable(_driver._poll_for_phase)

    def test_poll_for_phase_accepts_required_args(self):
        """_poll_for_phase is callable with the expected signature (smoke check)."""
        import inspect
        sig = inspect.signature(_driver._poll_for_phase)
        param_names = list(sig.parameters)
        assert "current_status" in param_names
        assert "url" in param_names
        assert "label" in param_names
        assert "max_ticks" in param_names
        assert "tick_secs" in param_names
        assert "tick" in param_names
        assert "log_fn" in param_names
        assert "emit_checkpoint_fn" in param_names


# ---------------------------------------------------------------------------
# 3. _poll_for_phase behavioural tests
# ---------------------------------------------------------------------------


def _make_status(phase: str | None) -> dict:
    return {"frame": 1, "window_logical_size": [1280, 720], "phase": phase}


class TestPollForPhaseBehaviour:
    """Unit-tests for _poll_for_phase using injected _rpc / _sleep."""

    def _run(
        self,
        statuses: list[dict | None],
        label: str = "target-phase",
        max_ticks: int = 5,
        tick_secs: float = 0.0,
    ) -> tuple[bool, list[dict]]:
        """Run _poll_for_phase with a queue of canned status responses.

        The first item is passed as ``current_status`` (the tick-start snapshot).
        Subsequent items are returned by the injected rpc_fn in order.
        """
        current = statuses[0]
        rpc_responses = list(statuses[1:])
        rpc_call_count = 0

        def fake_rpc(url, method):
            nonlocal rpc_call_count
            if rpc_call_count < len(rpc_responses):
                resp = rpc_responses[rpc_call_count]
                rpc_call_count += 1
                if resp is None:
                    raise ConnectionError("simulated RPC failure")
                return resp
            raise ConnectionError("no more responses")

        def fake_sleep(_secs):
            pass

        checkpoints: list[dict] = []
        logs: list[str] = []

        started = time.monotonic()
        result = _driver._poll_for_phase(
            current_status=current,
            url="http://fake/",
            label=label,
            max_ticks=max_ticks,
            tick_secs=tick_secs,
            tick=1,
            started=started,
            log_fn=logs.append,
            emit_checkpoint_fn=checkpoints.append,
            _rpc=fake_rpc,
            _sleep=fake_sleep,
        )
        return result, checkpoints

    # --- success: immediate match ---

    def test_poll_for_phase_matches_immediately(self):
        result, _ = self._run([_make_status("target-phase")])
        assert result is True

    def test_poll_for_phase_immediate_match_costs_zero_extra_rpcs(self):
        """When current_status already has the right phase, no extra RPC fires."""
        rpc_called = []

        def fake_rpc(url, method):
            rpc_called.append(method)
            return _make_status("wrong-phase")

        started = time.monotonic()
        result = _driver._poll_for_phase(
            current_status=_make_status("target-phase"),
            url="http://fake/",
            label="target-phase",
            max_ticks=5,
            tick_secs=0.0,
            tick=1,
            started=started,
            log_fn=lambda _: None,
            emit_checkpoint_fn=lambda _: None,
            _rpc=fake_rpc,
            _sleep=lambda _: None,
        )
        assert result is True
        assert rpc_called == [], "Should not call rpc when current_status already matches"

    def test_poll_for_phase_immediate_match_checkpoint_matched_true(self):
        _, checkpoints = self._run([_make_status("target-phase")])
        assert len(checkpoints) == 1
        assert checkpoints[0]["matched"] is True

    def test_poll_for_phase_immediate_match_polls_is_one(self):
        _, checkpoints = self._run([_make_status("target-phase")])
        assert checkpoints[0]["polls"] == 1

    # --- success: match after N polls ---

    def test_poll_for_phase_matches_after_two_polls(self):
        statuses = [
            _make_status("wrong"),
            _make_status("target-phase"),
        ]
        result, _ = self._run(statuses, max_ticks=5)
        assert result is True

    def test_poll_for_phase_matches_after_two_polls_checkpoint_polls_is_two(self):
        statuses = [
            _make_status("wrong"),
            _make_status("target-phase"),
        ]
        _, checkpoints = self._run(statuses, max_ticks=5)
        assert checkpoints[0]["polls"] == 2

    def test_poll_for_phase_matches_last_poll(self):
        """Match on the final allowed poll."""
        statuses = [
            _make_status("no"),
            _make_status("no"),
            _make_status("target-phase"),
        ]
        result, checkpoints = self._run(statuses, max_ticks=3)
        assert result is True
        assert checkpoints[0]["polls"] == 3

    # --- timeout ---

    def test_poll_for_phase_timeout_returns_false(self):
        statuses = [_make_status("wrong")] * 6
        result, _ = self._run(statuses, max_ticks=3)
        assert result is False

    def test_poll_for_phase_timeout_checkpoint_timed_out_true(self):
        statuses = [_make_status("wrong")] * 6
        _, checkpoints = self._run(statuses, max_ticks=3)
        assert checkpoints[0].get("timed_out") is True

    def test_poll_for_phase_timeout_checkpoint_matched_false(self):
        statuses = [_make_status("wrong")] * 6
        _, checkpoints = self._run(statuses, max_ticks=3)
        assert checkpoints[0]["matched"] is False

    def test_poll_for_phase_timeout_checkpoint_polls_equals_max_ticks(self):
        max_ticks = 4
        statuses = [_make_status("wrong")] * (max_ticks + 2)
        _, checkpoints = self._run(statuses, max_ticks=max_ticks)
        assert checkpoints[0]["polls"] == max_ticks

    # --- checkpoint row contract ---

    def test_poll_for_phase_emits_exactly_one_checkpoint(self):
        result, checkpoints = self._run([_make_status("target-phase")])
        assert len(checkpoints) == 1

    def test_poll_for_phase_checkpoint_kind_is_poll_phase(self):
        _, checkpoints = self._run([_make_status("target-phase")])
        assert checkpoints[0]["kind"] == "poll_phase"

    def test_poll_for_phase_checkpoint_label_matches_requested(self):
        _, checkpoints = self._run([_make_status("my-label")], label="my-label")
        assert checkpoints[0]["label"] == "my-label"

    def test_poll_for_phase_checkpoint_has_tick(self):
        _, checkpoints = self._run([_make_status("target-phase")])
        assert "tick" in checkpoints[0]
        assert checkpoints[0]["tick"] == 1

    def test_poll_for_phase_checkpoint_has_elapsed_secs(self):
        _, checkpoints = self._run([_make_status("target-phase")])
        assert "elapsed_secs" in checkpoints[0]
        assert isinstance(checkpoints[0]["elapsed_secs"], float)

    # --- RPC failure tolerance ---

    def test_poll_for_phase_rpc_failure_on_retry_does_not_crash(self):
        """A transient RPC error on a retry poll should be tolerated."""
        statuses = [
            _make_status("wrong"),
            None,  # simulates ConnectionError
            _make_status("target-phase"),
        ]
        result, _ = self._run(statuses, max_ticks=5)
        assert result is True

    def test_poll_for_phase_all_rpc_failures_returns_false(self):
        """All retries failing → timeout, returns False."""
        statuses = [
            _make_status("wrong"),
            None, None, None,
        ]
        result, _ = self._run(statuses, max_ticks=3)
        assert result is False

    # --- None current_status ---

    def test_poll_for_phase_none_current_status_does_not_crash(self):
        """None current_status (e.g., RPC failed before dispatch) is handled gracefully."""
        statuses = [None, _make_status("target-phase")]
        result, _ = self._run(statuses, max_ticks=5)
        assert result is True

    # --- max_ticks=1 edge case ---

    def test_poll_for_phase_max_ticks_one_match(self):
        result, _ = self._run([_make_status("target-phase")], max_ticks=1)
        assert result is True

    def test_poll_for_phase_max_ticks_one_no_match(self):
        result, _ = self._run([_make_status("wrong")], max_ticks=1)
        assert result is False


# ---------------------------------------------------------------------------
# 4. Serialisation / static compatibility
# ---------------------------------------------------------------------------


class TestPollPhaseStaticCompat:
    """Verify local.poll_phase fits into the existing allowlist machinery."""

    def test_local_poll_phase_would_pass_driver_allowlist_check(self):
        """An action with local.poll_phase is accepted by the allowlist union."""
        allowed = _driver.ALLOWED_RPC_METHODS | _driver.LOCAL_METHODS
        assert "local.poll_phase" in allowed

    def test_poll_phase_action_json_round_trips_cleanly(self):
        b = RecipeBuilder(window_size=(1280.0, 720.0))
        b.poll_phase("integration-test-phase", max_ticks=20)
        action = b.build()[0]
        dumped = json.dumps(action)
        loaded = json.loads(dumped)
        assert loaded == action

    def test_existing_local_methods_still_present(self):
        """Adding local.poll_phase must not remove any pre-existing LOCAL_METHODS."""
        required = {"local.checkpoint", "local.note", "local.block"}
        assert required.issubset(_driver.LOCAL_METHODS)
