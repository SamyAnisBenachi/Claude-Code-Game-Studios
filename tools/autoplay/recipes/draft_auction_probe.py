"""``draft-auction-probe`` -- exercise the shop/draft/auction UI.

This recipe does NOT claim to win an auction or complete a draft —
it drives the same low-level inputs a player exercises during the
shop and auction overlays. Reviewers verify outcomes from the
checkpoint screenshots.

Steps:
  1. checkpoint ``shop-loaded`` (screenshot);
  2. click the first shop slot;
  3. checkpoint ``shop-slot-clicked``;
  4. click Confirm (shop-side ready CTA);
  5. wait for the auction overlay to mount;
  6. checkpoint ``auction-loaded``;
  7. click the bid CTA;
  8. wait for bid acknowledgement (no observability hook today —
     ``CCGS_AUTOPLAY_AUCTION_BID_WAIT`` ticks, default 10);
  9. click Ready;
 10. checkpoint ``auction-ready``.
"""

from __future__ import annotations

from ._builder import RecipeBuilder
from ._coords import resolve

NAME = "draft-auction-probe"
DESCRIPTION = "Shop click + auction bid/ready click. Four checkpoints (shop-loaded, shop-slot-clicked, auction-loaded, auction-ready)."


def _wait_ticks(env: dict[str, str], key: str, default: int) -> int:
    raw = env.get(key)
    if raw is None:
        return default
    try:
        value = int(raw)
        if value < 1:
            return default
        return value
    except ValueError:
        return default


def build(ctx) -> list[dict]:
    b = RecipeBuilder(ctx.window_size)
    b.checkpoint("shop-loaded")

    slot, slot_note = resolve("SHOP_FIRST_SLOT", ctx.env)
    if slot_note:
        b.note(slot_note)
    sx, sy = b.frac(slot.fx, slot.fy)
    b.click(sx, sy)
    b.wait(4)
    b.checkpoint("shop-slot-clicked")

    shop_confirm, shop_confirm_note = resolve("SHOP_CONFIRM_BTN", ctx.env)
    if shop_confirm_note:
        b.note(shop_confirm_note)
    cx, cy = b.frac(shop_confirm.fx, shop_confirm.fy)
    b.click(cx, cy)
    b.wait(_wait_ticks(ctx.env, "CCGS_AUTOPLAY_AUCTION_MOUNT_WAIT", 12))
    b.checkpoint("auction-loaded")

    bid, bid_note = resolve("AUCTION_BID_BTN", ctx.env)
    if bid_note:
        b.note(bid_note)
    bx, by = b.frac(bid.fx, bid.fy)
    b.click(bx, by)
    b.wait(_wait_ticks(ctx.env, "CCGS_AUTOPLAY_AUCTION_BID_WAIT", 10))

    ready, ready_note = resolve("AUCTION_READY_BTN", ctx.env)
    if ready_note:
        b.note(ready_note)
    rx, ry = b.frac(ready.fx, ready.fy)
    b.click(rx, ry)
    b.wait(4)
    b.checkpoint("auction-ready")
    b.clear_input()
    return b.build()
