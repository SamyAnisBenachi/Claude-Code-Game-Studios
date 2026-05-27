"""Static regression tests for tools/autoplay recipe library (PROMPT 1647).

No GUI, no Bevy launch, no Cargo. These tests exercise:
  - registry completeness (11 expected recipes present)
  - method-allowlist contract (every recipe emits only permitted methods)
  - env-gate blocking (recipes that need env vars emit local.block when unset)
  - checkpoint presence (expected labels appear in builds when env is satisfied)

Run with:
    pytest tests/tools/autoplay/test_recipe_static.py -v
"""
from __future__ import annotations

import sys
from pathlib import Path

# Make ``tools/autoplay`` importable without installing the package.
_TOOLS_AUTOPLAY = Path(__file__).resolve().parents[3] / "tools" / "autoplay"
if str(_TOOLS_AUTOPLAY) not in sys.path:
    sys.path.insert(0, str(_TOOLS_AUTOPLAY))

import pytest

from recipes import REGISTRY, RecipeContext, names as recipe_names  # noqa: E402


# ---------------------------------------------------------------------------
# Constants mirrored from driver.py so tests stay decoupled from the driver.
# ---------------------------------------------------------------------------

ALLOWED_RPC_METHODS = {
    "autoplay/capabilities",
    "autoplay/status",
    "autoplay/input",
    "autoplay/clear_input",
    "autoplay/screenshot",
}

LOCAL_METHODS = {
    "local.checkpoint",
    "local.note",
    "local.block",
}

ALLOWED_METHODS = ALLOWED_RPC_METHODS | LOCAL_METHODS

EXPECTED_RECIPES = {
    "smoke",
    "idle",
    "add-bot-lobby",
    "lobby-create",
    "class-select",
    "draft-auction-probe",
    "placement-drag-probe",
    "resolution-observe",
    "game-over-observe",
    "round-loop",
    "full-game",
}

_DEFAULT_CTX = RecipeContext(window_size=(1280.0, 720.0), env={})
_BOT_ENV = {"CCGS_AUTOPLAY_BOT_ROOM_READY": "1"}


def _ctx(**env_overrides: str) -> RecipeContext:
    return RecipeContext(window_size=(1280.0, 720.0), env=dict(env_overrides))


def _checkpoint_labels(actions: list[dict]) -> list[str]:
    return [
        a["params"]["label"]
        for a in actions
        if a.get("method") == "local.checkpoint"
    ]


def _has_block(actions: list[dict]) -> bool:
    return any(a.get("method") == "local.block" for a in actions)


# ---------------------------------------------------------------------------
# 1. Registry completeness
# ---------------------------------------------------------------------------


class TestRegistry:
    def test_expected_recipe_count(self):
        assert len(REGISTRY) == len(EXPECTED_RECIPES), (
            f"Expected {len(EXPECTED_RECIPES)} recipes, got {len(REGISTRY)}. "
            f"Registry: {set(REGISTRY)}"
        )

    def test_expected_recipe_names_present(self):
        missing = EXPECTED_RECIPES - set(REGISTRY)
        extra = set(REGISTRY) - EXPECTED_RECIPES
        assert not missing, f"Recipes missing from registry: {missing}"
        assert not extra, f"Unexpected recipes in registry: {extra}"

    def test_names_returns_sorted_list(self):
        listed = recipe_names()
        assert listed == sorted(listed)

    def test_each_registry_entry_has_description_and_builder(self):
        for name, (desc, builder) in REGISTRY.items():
            assert isinstance(desc, str) and desc, f"{name}: description is empty"
            assert callable(builder), f"{name}: builder is not callable"


# ---------------------------------------------------------------------------
# 2. Method-allowlist contract
# ---------------------------------------------------------------------------


class TestMethodAllowlist:
    """Every action emitted by every recipe must use an allowed method."""

    @pytest.mark.parametrize("recipe_name", sorted(EXPECTED_RECIPES))
    def test_recipe_only_emits_allowed_methods(self, recipe_name: str):
        _, builder = REGISTRY[recipe_name]
        # Use bot env so gated recipes don't short-circuit to block early —
        # we want to exercise the full action stream.
        env = {
            "CCGS_AUTOPLAY_BOT_ROOM_READY": "1",
            "CCGS_DEBUG_UI": "1",
        }
        actions = builder(_ctx(**env))
        bad = {
            a["method"]
            for a in actions
            if a.get("method") not in ALLOWED_METHODS
        }
        assert not bad, (
            f"Recipe '{recipe_name}' emitted disallowed method(s): {bad}"
        )

    @pytest.mark.parametrize("recipe_name", sorted(EXPECTED_RECIPES))
    def test_recipe_actions_have_required_keys(self, recipe_name: str):
        _, builder = REGISTRY[recipe_name]
        env = {"CCGS_AUTOPLAY_BOT_ROOM_READY": "1", "CCGS_DEBUG_UI": "1"}
        actions = builder(_ctx(**env))
        for i, action in enumerate(actions):
            assert "tick" in action, f"{recipe_name} action[{i}] missing 'tick'"
            assert "method" in action, f"{recipe_name} action[{i}] missing 'method'"
            assert isinstance(action["tick"], int), (
                f"{recipe_name} action[{i}]['tick'] must be int"
            )

    @pytest.mark.parametrize("recipe_name", sorted(EXPECTED_RECIPES))
    def test_recipe_ticks_are_positive(self, recipe_name: str):
        _, builder = REGISTRY[recipe_name]
        env = {"CCGS_AUTOPLAY_BOT_ROOM_READY": "1", "CCGS_DEBUG_UI": "1"}
        actions = builder(_ctx(**env))
        bad = [a for a in actions if a.get("tick", 0) < 1]
        assert not bad, (
            f"{recipe_name}: action(s) with tick < 1: {bad}"
        )


# ---------------------------------------------------------------------------
# 3. Env-gate blocking
# ---------------------------------------------------------------------------


class TestEnvGateBlocking:
    """Env-gated recipes must emit local.block when their gate var is absent."""

    def test_full_game_blocks_without_bot_env(self):
        _, builder = REGISTRY["full-game"]
        actions = builder(_ctx())
        assert _has_block(actions), (
            "full-game must emit local.block when CCGS_AUTOPLAY_BOT_ROOM_READY is unset"
        )

    def test_full_game_does_not_block_with_bot_env(self):
        _, builder = REGISTRY["full-game"]
        actions = builder(_ctx(**_BOT_ENV))
        assert not _has_block(actions), (
            "full-game must NOT emit local.block when CCGS_AUTOPLAY_BOT_ROOM_READY=1"
        )

    def test_round_loop_blocks_without_bot_env(self):
        _, builder = REGISTRY["round-loop"]
        actions = builder(_ctx())
        assert _has_block(actions), (
            "round-loop must emit local.block when CCGS_AUTOPLAY_BOT_ROOM_READY is unset"
        )

    def test_round_loop_does_not_block_with_bot_env(self):
        _, builder = REGISTRY["round-loop"]
        actions = builder(_ctx(**_BOT_ENV))
        assert not _has_block(actions), (
            "round-loop must NOT emit local.block when CCGS_AUTOPLAY_BOT_ROOM_READY=1"
        )

    def test_add_bot_lobby_blocks_without_debug_ui(self):
        _, builder = REGISTRY["add-bot-lobby"]
        actions = builder(_ctx())
        assert _has_block(actions), (
            "add-bot-lobby must emit local.block when CCGS_DEBUG_UI is unset"
        )

    def test_add_bot_lobby_does_not_block_with_debug_ui(self):
        _, builder = REGISTRY["add-bot-lobby"]
        actions = builder(_ctx(CCGS_DEBUG_UI="1"))
        assert not _has_block(actions), (
            "add-bot-lobby must NOT emit local.block when CCGS_DEBUG_UI=1"
        )

    def test_full_game_block_row_is_first_meaningful_action(self):
        """The block row must appear early — recipe should not do work then block."""
        _, builder = REGISTRY["full-game"]
        actions = builder(_ctx())
        block_idx = next(
            (i for i, a in enumerate(actions) if a.get("method") == "local.block"),
            None,
        )
        assert block_idx is not None
        # Allow only checkpoint/note rows before the block.
        pre_block = actions[:block_idx]
        rpc_before_block = [
            a for a in pre_block if a.get("method") in ALLOWED_RPC_METHODS
        ]
        assert not rpc_before_block, (
            "full-game emitted RPC calls before local.block — "
            "blocking should happen before any network I/O"
        )


# ---------------------------------------------------------------------------
# 4. Checkpoint contracts
# ---------------------------------------------------------------------------


class TestCheckpointContracts:
    """Expected checkpoint labels must appear in build output."""

    # -- resolution-observe --------------------------------------------------

    def test_resolution_observe_checkpoints(self):
        _, builder = REGISTRY["resolution-observe"]
        labels = _checkpoint_labels(builder(_ctx()))
        assert "resolution-started" in labels
        assert "resolution-complete" in labels

    def test_resolution_observe_checkpoint_order(self):
        _, builder = REGISTRY["resolution-observe"]
        labels = _checkpoint_labels(builder(_ctx()))
        assert labels.index("resolution-started") < labels.index("resolution-complete")

    # -- game-over-observe ---------------------------------------------------

    def test_game_over_observe_checkpoints(self):
        _, builder = REGISTRY["game-over-observe"]
        labels = _checkpoint_labels(builder(_ctx()))
        assert "game-over-wait-start" in labels
        assert "game-over-screen" in labels
        assert "winner-confirmed" in labels

    def test_game_over_observe_checkpoint_order(self):
        _, builder = REGISTRY["game-over-observe"]
        labels = _checkpoint_labels(builder(_ctx()))
        assert labels.index("game-over-wait-start") < labels.index("game-over-screen")
        assert labels.index("game-over-screen") < labels.index("winner-confirmed")

    # -- lobby-create --------------------------------------------------------

    def test_lobby_create_checkpoints(self):
        _, builder = REGISTRY["lobby-create"]
        labels = _checkpoint_labels(builder(_ctx()))
        assert "lobby-loaded" in labels
        assert "lobby-confirmed" in labels

    # -- class-select --------------------------------------------------------

    def test_class_select_checkpoints(self):
        _, builder = REGISTRY["class-select"]
        labels = _checkpoint_labels(builder(_ctx()))
        assert "class-select-loaded" in labels
        assert "class-confirmed" in labels

    # -- draft-auction-probe -------------------------------------------------

    def test_draft_auction_probe_checkpoints(self):
        _, builder = REGISTRY["draft-auction-probe"]
        labels = _checkpoint_labels(builder(_ctx()))
        assert "shop-loaded" in labels
        assert "auction-loaded" in labels

    # -- placement-drag-probe ------------------------------------------------

    def test_placement_drag_probe_checkpoints(self):
        _, builder = REGISTRY["placement-drag-probe"]
        labels = _checkpoint_labels(builder(_ctx()))
        assert "placement-loaded" in labels
        assert "placement-submitted" in labels

    # -- add-bot-lobby (when unblocked) --------------------------------------

    def test_add_bot_lobby_checkpoints_when_unblocked(self):
        _, builder = REGISTRY["add-bot-lobby"]
        labels = _checkpoint_labels(builder(_ctx(CCGS_DEBUG_UI="1")))
        assert "lobby-loaded" in labels
        assert "bot-added" in labels
        assert "lobby-confirmed" in labels

    # -- full-game (when unblocked, default: resolution on, gameover off) ----

    def test_full_game_tail_checkpoint_post_resolution(self):
        """Default build (resolution on, gameover off) ends with full-game-post-resolution."""
        _, builder = REGISTRY["full-game"]
        labels = _checkpoint_labels(builder(_ctx(**_BOT_ENV)))
        assert "full-game-post-resolution" in labels

    def test_full_game_tail_checkpoint_post_placement(self):
        """Resolution disabled → tail checkpoint is full-game-post-placement."""
        _, builder = REGISTRY["full-game"]
        env = {**_BOT_ENV, "CCGS_AUTOPLAY_FULL_GAME_RESOLUTION": "0"}
        labels = _checkpoint_labels(builder(_ctx(**env)))
        assert "full-game-post-placement" in labels

    def test_full_game_tail_checkpoint_complete(self):
        """GameOver enabled → tail checkpoint is full-game-complete."""
        _, builder = REGISTRY["full-game"]
        env = {**_BOT_ENV, "CCGS_AUTOPLAY_FULL_GAME_GAMEOVER": "1"}
        labels = _checkpoint_labels(builder(_ctx(**env)))
        assert "full-game-complete" in labels

    def test_full_game_includes_sub_recipe_checkpoints(self):
        """Full-game must include checkpoints from each sub-recipe phase."""
        _, builder = REGISTRY["full-game"]
        labels = _checkpoint_labels(builder(_ctx(**_BOT_ENV)))
        for expected in (
            "lobby-loaded",
            "class-select-loaded",
            "shop-loaded",
            "placement-loaded",
            "resolution-started",
        ):
            assert expected in labels, (
                f"full-game missing sub-recipe checkpoint: {expected!r}"
            )

    # -- round-loop (when unblocked, default 2 rounds) -----------------------

    def test_round_loop_tail_checkpoint(self):
        _, builder = REGISTRY["round-loop"]
        labels = _checkpoint_labels(builder(_ctx(**_BOT_ENV)))
        assert "round-loop-complete" in labels

    def test_round_loop_includes_game_over_checkpoints(self):
        _, builder = REGISTRY["round-loop"]
        labels = _checkpoint_labels(builder(_ctx(**_BOT_ENV)))
        assert "game-over-screen" in labels
        assert "winner-confirmed" in labels

    def test_round_loop_round2_marker_present_for_default_count(self):
        """With default loop count of 2, a round-2-start marker must appear."""
        _, builder = REGISTRY["round-loop"]
        labels = _checkpoint_labels(builder(_ctx(**_BOT_ENV)))
        assert "round-2-start" in labels

    def test_round_loop_no_round3_marker_at_default_count(self):
        """Default count is 2, so round-3-start must NOT appear."""
        _, builder = REGISTRY["round-loop"]
        labels = _checkpoint_labels(builder(_ctx(**_BOT_ENV)))
        assert "round-3-start" not in labels

    def test_round_loop_count_env_adds_extra_round_markers(self):
        env = {**_BOT_ENV, "CCGS_AUTOPLAY_ROUND_LOOP_COUNT": "3"}
        _, builder = REGISTRY["round-loop"]
        labels = _checkpoint_labels(builder(_ctx(**env)))
        assert "round-2-start" in labels
        assert "round-3-start" in labels

    # -- smoke / idle (simple) -----------------------------------------------

    def test_smoke_emits_actions(self):
        _, builder = REGISTRY["smoke"]
        assert len(builder(_ctx())) > 0

    def test_idle_emits_no_actions(self):
        _, builder = REGISTRY["idle"]
        assert builder(_ctx()) == []
