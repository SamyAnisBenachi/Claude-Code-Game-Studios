# Story 021: S19-UI-SHOP-AUCTION-CARD-PRODUCT-LAYOUT-001 -- Shop/Auction Card-Product Layout

> **Epic**: Shop / Auction UI
> **Story ID**: `S19-UI-SHOP-AUCTION-CARD-PRODUCT-LAYOUT-001`
> **Status**: Draft -- future Sprint 19 candidate; NOT activated
> **Layer**: Presentation / Shop-Auction UI
> **Type**: UI + Integration
> **Sprint**: Future Sprint 19 implementation wave; depends on Sprint 18 shop/auction polish and card primitive work
> **Authored**: 2026-05-18 by PROMPT 1280
> **Authoring source-of-truth**: local `HEAD@051b59d`; workspace had unrelated dirty code/runtime files that were not touched
> **Source reports**: PROMPT 1265, 1266, 1267
> **Estimated effort**: ~0.6d

---

## Status / No-Claim Banner

This story is a future candidate only. It does not activate Sprint 19 and does
not claim release readiness, final-art completion, Standard-tier accessibility
completion, playtest validation, full game completion, or closure of
`PAW-TD-*-a`.

Krosmaga does not contain CCGS DraftShop/DraftAuction. This story translates
Krosmaga card-decision, card-hover, and action-cluster patterns to CCGS shop and
auction rules without changing CCGS economy, auction, or RSM semantics.

---

## Source Findings

PROMPT 1266 gives the direct CCGS translation:

- Shop offers should read like real card products, not placeholder slabs.
- At 16:9, each shop offer should target roughly `12%-15%` viewport width and
  `34%-42%` viewport height inside a centered or lower-middle shop band.
- Auction featured card should be the hero, roughly `18%-24%` viewport width and
  `42%-52%` viewport height.
- Bid increments must be text-explicit (`+1`, `+2`, `+3`) and not fallback
  glyphs.
- Highest bid, refund, available gold, and timer should be grouped as status
  chips near the CTA row, not collapsed into one run.
- Buttons must be visually distinct from read-only chips.

PROMPT 1265 confirms CCGS already has shop/auction logical surfaces; the gap is
readability and continuity. PROMPT 1267 maps shop/auction chrome and shared
buttons/panels as dev-only candidates after provenance gating.

---

## Scope

### In Scope

- Re-layout DraftShop offers as card-product slots with shared card anatomy,
  cost/status chips, and distinct Buy/Refresh/Ready controls.
- Re-layout DraftAuction around a large featured card, explicit bid increments,
  highest-bid/refund/free-gold/timer chips, and a clear primary CTA cluster.
- Preserve read-only chip semantics: status chips must not carry `Button`
  behavior or hover cursor.
- Keep timer urgency numeric plus visual and located near the active auction CTA.
- Keep shop/auction panels from occluding the hand unless the hand is explicitly
  inactive and visually dimmed.
- Integrate hover inspect/glossary from
  `S19-UI-CARD-RENDERING-FIDELITY-HOVER-GLOSSARY-001` when available.
- Add QA snapshot/debug fields for offer card bounds, featured card bounds,
  CTA bounds, chip/button role classification, and timer urgency stage.

### Out Of Scope

- Auction settlement/disposition semantics owned by Story 020.
- New bid increments or economy formulas.
- Server protocol changes.
- Krosmaga AP/end-turn rules.
- Final art or release approval.

---

## Dependencies And Parallelism

| Dependency | Required posture |
|---|---|
| `S19-UI-CARD-RENDERING-FIDELITY-HOVER-GLOSSARY-001` | Should land first or expose a stable shared card primitive. |
| Shop/Auction Story 020 | Auction-won disposition affordance must remain compatible. |
| Sprint 18 layout/panel/interaction-state stories | Reuse PlayArea, chip-vs-button, and interaction-state primitives where landed. |
| `S18-PAW-KROSMAGA-DEV-PROXY-PACK-BOUNDARY-001` | Required before local dev-proxy chrome use. |

This story owns `client/src/ui/shop_auction/**` layout and should not run
concurrently with another shop/auction layout rewrite.

---

## Acceptance Criteria

- [ ] **AC1 -- Shop offers are card products**: Each shop slot renders a
  portrait card-product layout with art, title, cost, status, and action areas;
  no offer appears as a single placeholder slab.
- [ ] **AC2 -- Shop actions are separated**: Buy, Refresh, Ready, and read-only
  cost/status chips are visually and semantically distinct.
- [ ] **AC3 -- Auction featured card is dominant**: Auction phase centers visual
  hierarchy on the featured card with enough space for readable title/art/rules
  text at 1366x768.
- [ ] **AC4 -- Bid increments are explicit**: Bid buttons show explicit text
  increments and never render fallback glyphs or ambiguous icon-only controls.
- [ ] **AC5 -- Auction status chips are grouped**: Highest bid, current leader,
  refund/reserved gold, available gold, and timer are grouped near the action
  cluster without becoming interactive buttons.
- [ ] **AC6 -- Timer urgency is numeric plus visual**: Auction timer displays a
  readable number and calm/warning/critical stage styling near the CTA row.
- [ ] **AC7 -- Hand occlusion controlled**: Shop/auction panels do not cover an
  active hand fan; if the hand is inactive, it is visibly dimmed and still
  spatially understandable.
- [ ] **AC8 -- Hover inspect compatible**: Shop offer cards and the auction
  featured card can open the shared hover inspect/glossary state when that
  primitive exists.
- [ ] **AC9 -- QA snapshot exposes roles**: Snapshot/debug output identifies card
  product bounds, featured card bounds, CTA bounds, chip roles, button roles,
  and timer urgency stage.
- [ ] **AC10 -- Evidence captured**: Browser/WASM captures show DraftShop,
  DraftAuction idle, leader, losing, and low-timer states at minimum and target
  viewports.
- [ ] **AC11 -- No release/proxy claim**: Completion notes preserve dev-only
  proxy boundaries and final-art accepted-risk posture.

---

## Worker Contract

1. Worktree slug: `work/s19-ui-shop-auction-card-product-layout`.
2. Activate `liv-bevy-018` before touching Bevy/Rust files.
3. Read PROMPT 1265/1266/1267 and Shop/Auction Stories 016-020.
4. Preserve server-authoritative auction/economy behavior.
5. Run targeted shop-auction UI integration tests plus browser/WASM evidence.
6. Do not copy Krosmaga assets and do not modify sprint/session state.

