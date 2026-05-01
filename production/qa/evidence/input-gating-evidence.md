# CARD-ANIM-008 Input Gating Evidence

Status: Implementation evidence captured; visual walkthrough pending playable UI integration.

## Automated Evidence

- CA-13: Covered by `card_animations_input_gating_test::timer_bar_ease_request_starts_animator_same_update`.
- CA-23: Covered by `card_animations_input_gating_test::drag_start_in_placement_starts_drag_sprite_animator_same_update`.
- CA-24: Covered by `card_animations_input_gating_test::hover_enter_keeps_returning_card_playing_and_starts_new_hover`.

## Manual Evidence Pending

- CA-13b: Bid buttons enabled during timer-bar tween.
  - Blocker: Shop/Auction UI bid buttons are not implemented in this branch.
  - Required capture later: screenshot or clip showing a successful bid click while timer-bar ease is still in flight.

- CA-22: <= 2 animated UI regions at DRAFT_INITIAL phase entry.
  - Blocker: DRAFT_INITIAL panel and hand card-draw UI sequencing is not implemented in this branch.
  - Required capture later: screenshot or clip showing panel slide and card-draw animations do not overlap.

Lead sign-off: Pending manual walkthrough.
