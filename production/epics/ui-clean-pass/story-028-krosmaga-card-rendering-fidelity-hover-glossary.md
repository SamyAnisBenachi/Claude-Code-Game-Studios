# Story 028: S19-UI-CARD-RENDERING-FIDELITY-HOVER-GLOSSARY-001 -- Card Rendering Fidelity + Hover Inspect Glossary

> **Epic**: UI Clean-Pass
> **Story ID**: `S19-UI-CARD-RENDERING-FIDELITY-HOVER-GLOSSARY-001`
> **Status**: Draft -- future Sprint 19 candidate; NOT activated
> **Layer**: Presentation / UI card primitive
> **Type**: UI + Integration
> **Sprint**: Future Sprint 19 implementation wave; depends on Sprint 18 layout/card primitive foundations
> **Authored**: 2026-05-18 by PROMPT 1280
> **Authoring source-of-truth**: local `HEAD@051b59d`; workspace had unrelated dirty code/runtime files that were not touched
> **Source reports**: PROMPT 1265, 1266, 1267
> **Estimated effort**: ~0.75d

---

## Status / No-Claim Banner

This story is a future candidate only. It does not activate Sprint 19 and does
not claim release readiness, final-art completion, Standard-tier accessibility
completion, playtest validation, full game completion, or closure of any
`PAW-TD-*-a` accept-risk row.

Krosmaga is a composition and state-feedback reference. Any Krosmaga proxy
candidate referenced by implementation remains dev-only under the provenance
boundary story `S18-PAW-KROSMAGA-DEV-PROXY-PACK-BOUNDARY-001`; this story does
not copy Krosmaga art, approve it for release, or place it in `assets/**`.

---

## Source Findings

PROMPT 1266 identifies card readability as the first Krosmaga-style
implementation priority:

- Hand, shop, auction, and modal cards need aspect-fit art.
- Card anatomy needs stable zones: cost badge, art window, title strip, rules
  text/stat area, and rarity/class strip.
- Hover/inspect should scale a card close to modal size without leaving play.
- Keyword glossary panels should appear near the inspected card and explain
  terms in context.

PROMPT 1265 shows large card inspect plus glossary panels as a state
communication system: the player can read the card, understand keywords, and
still see the board/action context. PROMPT 1267 maps the relevant dev-proxy
families to card frames, backs, tooltips, and movement/audio references, all
classified as dev-only.

---

## Scope

### In Scope

- Extend the shared card-slot/card-display primitive so all card surfaces can
  consume the same structural zones:
  cost badge, art window, title strip, rules text, stat badges, rarity/class/type
  strip, and hover target.
- Use aspect-fit image behavior for card art; never stretch source art.
- Add a hover/inspect presentation state for local readable card surfaces:
  hand fan cards, draft/grid cards, shop offer cards, auction featured card,
  and result/modal card summaries where already present.
- Render the inspected card at a large readable size in the current viewport
  without hiding the primary CTA or destroying board/hand spatial context.
- Render glossary panels for keywords/status terms found in the inspected card
  using existing keyword/GDD definitions or an existing keyword registry if
  available.
- Add QA snapshot/debug fields sufficient to prove which card is inspected and
  which glossary terms are visible.
- Add targeted Bevy UI integration tests and browser/WASM visual evidence across
  at least 1366x768 and 1920x1080.

### Out Of Scope

- New card rules, keyword mechanics, or card data semantics.
- Any server or protocol change.
- Final art production or release approval.
- Accessibility closure beyond explicit evidence gathered for this story.
- Krosmaga AP/deck/discard/fatigue semantics.

---

## Dependencies And Parallelism

| Dependency | Required posture |
|---|---|
| `S18-PAW-KROSMAGA-DEV-PROXY-PACK-BOUNDARY-001` | Must be Done before using Krosmaga proxy rows locally. |
| `S18-UI-CARD-ART-AND-LABEL-STRIP-001` | Should be Done or explicitly superseded; this story builds on the label-strip primitive. |
| Hand UI Story 015 / Shop-Auction UI Story 013 | Reuse card text/stat/keyword readability decisions where landed. |
| Keyword System docs | Source for glossary text; this story does not invent mechanics. |

This story is not file-disjoint from hand/shop/card primitive consumers. Run it
before per-surface layout stories, or explicitly split the primitive changes from
surface migrations.

---

## Acceptance Criteria

- [ ] **AC1 -- Shared card anatomy zones exist**: The card primitive exposes
  stable visual zones for cost, art, title, rules text, stats, and class/rarity
  markers without per-surface ad hoc layout.
- [ ] **AC2 -- Aspect-fit art enforced**: Hand, draft/grid, shop, auction, and
  modal/result card surfaces use aspect-fit behavior and do not stretch art.
- [ ] **AC3 -- Hover inspect is readable**: At 1366x768 and 1920x1080, inspected
  cards render large enough to read title, cost, stats, and rules text without
  navigating away from the game surface.
- [ ] **AC4 -- Glossary panels are contextual**: Keywords/status terms present on
  the inspected card produce adjacent glossary panels sourced from canonical
  keyword/status definitions.
- [ ] **AC5 -- Z-order is correct**: Hover card appears above hand/shop/auction
  base UI and below hard modal/result overlays; glossary panels appear above the
  hover card when overlapping.
- [ ] **AC6 -- CTA remains usable**: Hover inspect does not cover the current
  primary action CTA unless a hard modal is already active.
- [ ] **AC7 -- Passive/locked cards remain readable**: Disabled, passive, or
  opponent-facing states visibly dim cards while preserving enough structure for
  planning and inspection where inspection is allowed.
- [ ] **AC8 -- QA snapshot exposes inspect state**: Snapshot/debug output records
  inspected `card_id`, source surface, hover-card visibility, and glossary term
  count/keys.
- [ ] **AC9 -- Tests cover lifecycle**: Integration tests prove inspect opens on
  hover/focus, updates when the source card changes, and clears on hover/focus
  exit, phase exit, or modal takeover.
- [ ] **AC10 -- Evidence captured**: Browser/WASM screenshots demonstrate hand
  card inspect, shop/auction card inspect, and glossary panels at minimum and
  target viewports.
- [ ] **AC11 -- Provenance boundary preserved**: Any proxy art references remain
  dev-only and excluded from release claims.

---

## Worker Contract

1. Worktree slug: `work/s19-ui-card-rendering-fidelity-hover-glossary`.
2. Activate `liv-bevy-018` before touching Bevy/Rust files.
3. Read PROMPT 1265/1266/1267 and the card primitive stories before editing.
4. Keep all new state client-presentation-only.
5. Run targeted client integration tests plus browser/WASM visual evidence.
6. Do not copy Krosmaga assets and do not modify `production/sprint-status.yaml`.
