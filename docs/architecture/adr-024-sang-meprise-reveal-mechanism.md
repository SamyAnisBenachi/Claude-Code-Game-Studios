# ADR-024: Sang Méprise Reveal Mechanism — Recipient Scope, Lifecycle, and Client Cache Contract

## Status

Accepted

## Date

2026-05-18

## Last Verified

2026-05-18

## Decision Makers

- User (Sprint-17 orchestrator)
- technical-director (ADR-001 author; ObjectiveIdentity unicast precedent)
- network-programmer (`shared/src/protocol.rs`, reconnect dispatch)
- ui-programmer (Board Rendering / OQ-BR-01 owner)

## Summary

Resolves the long-pending "Sang Méprise reveal mechanism" entry in the ADR
registry (`.claude/docs/technical-preferences.md:81`). Chooses a single
recipient model (parallel unicast of the full alive-objectives set to both
players), names the server-state mutation contract for `sang_meprise_active`
and `ReconnectTracker.sang_meprise_sent_to`, defines the client-side
`ObjectiveIdentityCache` lifecycle, and closes Board Rendering OQ-BR-01 plus
cross-review row C-R9-W8. No protocol behaviour changes in this ADR; it
unblocks the client-drain story `S18-PROTO-SANG-MEPRISE-DRAIN-001`.

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Domain** | Networking (S2C unicast) / UI (board rendering reveal overlay) |
| **Knowledge Risk** | LOW — the wire shape (`S2CSangMepriseReveal`) and the reconnect-restore field (`S2CGameSnapshot.active_sang_meprise_reveals`) already exist on `origin/main @ 1345c6b`; this ADR ratifies the existing pattern and writes the client-side rendering contract. |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`; `liv-bevy-018` (Message/Event split, `MessageReceiver<T>` drain pattern); `liv-bevy-lightyear` (`NetworkTarget::Single`, reliable-channel ordering); ADR-001 (objective-identity unicast precedent); ADR-002 (client-server authority); ADR-008 (Lightyear channel config — `ReliableChannel`); ADR-011 (reconnect-snapshot evidence); ADR-018 (keyword system — `liv-bevy-018` skill activation pattern); ADR-021 (presentation layer architecture). |
| **Post-Cutoff APIs Used** | `#[derive(Message)]` + `MessageReceiver<T>` (Bevy 0.17+); `app.add_message::<T>()` registration (Bevy 0.17+); `NetworkTarget::Single` / `single(peer_id)` helper (Lightyear 0.26). |
| **Verification Required** | None new — the producer side at `server/src/core/session/reconnect.rs:473-493, 998-1006` and the snapshot field at `shared/src/protocol.rs:842` are already verified by `tests/integration/session/result_acknowledgement_*.rs` and `tests/integration/session/game_over_teardown_test.rs:228-230`. Drain-side verification is the responsibility of `S18-PROTO-SANG-MEPRISE-DRAIN-001` once that story lands. |

> **Note**: Knowledge Risk is LOW. If the project ever rolls back from the
> 0.17+ Message API or replaces Lightyear, re-validate this ADR against the
> new transport contract before reintroducing the drain.

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-001 (objective-identity unicast — Accepted; this ADR supersedes ADR-001 §5's "unicast to the opponent only" sub-clause for Sang Méprise specifically, see Migration Plan); ADR-002 (client-server authority — Accepted); ADR-008 (Lightyear channel config — Accepted; binds `S2CSangMepriseReveal` to `ReliableChannel`); ADR-011 (reconnect-snapshot — Accepted; `S2CGameSnapshot.active_sang_meprise_reveals` is the snapshot-restore field). |
| **Enables** | `S18-PROTO-SANG-MEPRISE-DRAIN-001` (client-side `MessageReceiver<S2CSangMepriseReveal>` drain + `ObjectiveIdentityCache` population + RESOLUTION-end clear); future Combat-Resolution / Class-System story that adds the **live-fire** server producer (out of scope here — see Non-Claims). |
| **Blocks** | Removal of the `S2CSangMepriseReveal` allowlist row at `tests/invariants/protocol_completeness_test.rs:296-309`. The row stays until `S18-PROTO-SANG-MEPRISE-DRAIN-001` lands; this ADR's acceptance is the gating artifact named in the rationale text. |
| **Ordering Note** | This ADR is design-only. It is safe to Accept before any code, test, or sprint-status change. The follow-on drain story must NOT open until this ADR is Accepted. The deeper Class-System live-fire producer (setting `sang_meprise_active` on Sang Méprise play) is governed by its own Class System / Combat Resolution epic and is not gated on the drain story landing — the drain story works against the existing reconnect-restore producer plus integration-test fixtures. |

## Context

### Problem Statement

The Sang Méprise reveal mechanism is referenced across five GDDs and one
ADR, but three of those sources disagree about who receives the message,
and a sixth (`board-rendering.md` OQ-BR-01) leaves the audio-suppression
signal undefined. The protocol message `S2CSangMepriseReveal` and its
snapshot-restore counterpart `S2CGameSnapshot.active_sang_meprise_reveals`
have shipped on `origin/main`, and a live producer exists in the reconnect
path (`server/src/core/session/reconnect.rs:481-485`). However, the
client-side drain (`MessageReceiver<S2CSangMepriseReveal>`) does not exist —
it is the only Sprint-13 `S2CSangMepriseReveal` orphan allowlisted in
`tests/invariants/protocol_completeness_test.rs:296-309` under Story 008's
Path C deferral. The allowlist rationale names this ADR as the gating
artifact for safe drain-side implementation. Without this ADR being
Accepted, an implementer attempting to write the drain must guess between
three incompatible recipient models and would have no published rule for
either the client cache lifecycle (`ObjectiveIdentityCache` in
`board-rendering.md:75`) or the audio-suppression signal (OQ-BR-01).

The cost of not deciding now is sprint drift: the ADR has been pending
since Sprint 13 (Story 008, 2026-04-30 era), the `S2CSangMepriseReveal`
allowlist row has carried a "pending ADR" rationale for three sprints, and
the candidate drain story `S14-PROTO-SANG-MEPRISE-DRAIN-001` was never
authored. Sprint 18 is being drafted (`1345c6b plan(s18): draft Sprint 18
plan (PROMPT 1285)`) and a re-slugged `S18-PROTO-SANG-MEPRISE-DRAIN-001`
needs an Accepted ADR to be implementation-ready.

### Current State

Three contradictory recipient statements coexist:

| Source | Quote / paraphrase | Implication |
|---|---|---|
| `design/gdd/network-protocol.md:146` | `S2CSangMepriseReveal \| Reliable \| **Unicast (opponent)**` — "Decided in `objective-system.md` Open Question 6 (Option B: targeted unicast)." | Exactly one recipient: the player who did NOT play Sang Méprise. |
| `design/gdd/class-system.md:123` and CS-5 worked example at `:243-262` | "Server unicasts identity of every alive objective (both players) **to both players** for current RESOLUTION." CS-5 worked example explicitly shows the full alive-objectives `reveal_set` (size 0–10) sent via separate parallel unicasts to Player A and Player B. | Both players receive the full reveal payload. |
| `design/gdd/lanes-and-lies-gdd.md:377, 713-714` | "Reveals all 5 objectives of all teams (real and fake status **visible to everyone**) for the duration of this resolution phase." | Both players see all alive objectives' identities for the RESOLUTION. |
| `docs/architecture/adr-001-objective-identity-unicast.md:59` | "targeted unicast `S2CSangMepriseReveal { identities: Vec<(LaneId, bool)> }` to the **opponent** only." | Same as NP GDD; opponent-only. |
| `design/gdd/objective-system.md:228` | "targeted reliable unicast to the **opponent** only (Option B — see OQ5)." | Same as NP GDD and ADR-001. |

The live reconnect-restore implementation at
`server/src/core/session/reconnect.rs:998-1006` (`sang_meprise_reveal_message`)
is per-recipient: it builds a payload using `active_sang_meprise_pairs`
(`:1034-1040`) which calls `active_sang_meprise_reveals` (`:1008-1032`),
both of which iterate `session_opponents(recipient)` and the full
`HiddenObjectives` for each opponent. The current shape can support either
the opponent-only model OR the both-players model — the wire payload
already carries the full alive-objective set, and `ReconnectTracker
.sang_meprise_sent_to` (`server/src/core/session/state.rs:151`) is a
per-recipient set that is checked before sending. There is no production
code path that **sets** `sang_meprise_active` or inserts into
`sang_meprise_sent_to`; the only producers are integration-test fixtures
(`tests/integration/session/result_acknowledgement_*.rs`) — confirming the
PROMPT 1299 finding that the server live-fire path is a separate, larger
Class-System / Combat-Resolution epic.

OQ-BR-01 (`design/gdd/board-rendering.md:877-879`) remains formally OPEN.
The `ObjectiveIdentityCache` resource is defined in
`board-rendering.md:75` "for the Sang Méprise audio-suppression branch"
but the delivery mechanism for the suppression signal is undefined and the
cache has no consumer wired in `client/src/`. The Card-Animations GDD
(referenced from cross-review row `C-R9-W8` in
`design/gdd/gdd-cross-review-2026-04-30-r9.md:135-137`) contains zero
matches for "Sang Méprise" or "surprise" and does not document the
reveal-suppression contract at all.

### Constraints

- **Server-authoritative reveal**: per ADR-002, the client cannot derive
  identity from any source other than authoritative S2C messages. The
  cache must be populated only by `S2CSangMepriseReveal` arrival or by
  `S2CGameSnapshot.active_sang_meprise_reveals`.
- **ADR-001 isolation invariant** (`design/gdd/board-rendering.md:271`):
  Board Rendering MUST NOT query any "identity-like" component for
  standing-objective rendering. The `ObjectiveIdentityCache` resource is
  read ONLY by the audio-suppression branch.
- **No protocol behaviour change in this ADR**: the allowlist row at
  `tests/invariants/protocol_completeness_test.rs:296-309` must remain
  until `S18-PROTO-SANG-MEPRISE-DRAIN-001` implementation lands. This ADR
  must NOT touch the allowlist, MUST NOT add a drain receiver in
  `client/src/`, and MUST NOT modify the server live-fire path.
- **Reconnect semantics already shipped**: `S2CGameSnapshot
  .active_sang_meprise_reveals: Option<Vec<ObjectiveReveal>>`
  (`shared/src/protocol.rs:842`) is consumed by the reconnect flow at
  `server/src/core/session/reconnect.rs:222-224`. CS Edge Case
  (`class-system.md:484`) names this as resolving the former OQ-CS-2. Any
  new contract MUST preserve this restore path verbatim.
- **`liv-bevy-018` skill activation**: the future drain story will run
  inside `client/src/`, importing `bevy` and (via Lightyear) `lightyear`.
  Both `liv-bevy-018` and `liv-bevy-lightyear` skills are MANDATORY for
  that story per `docs/engine-reference/bevy/VERSION.md` routing rules.

### Requirements

- Single, unambiguous recipient model for `S2CSangMepriseReveal` that
  matches the card-design fantasy ("everyone knows for this round").
- Single, unambiguous server-state mutation contract specifying when
  `sang_meprise_active` is set, when `sang_meprise_sent_to` is
  populated, and when both clear.
- Snapshot-restore contract reaffirmed without schema change.
- Client cache lifecycle that respects ADR-001 isolation invariant.
- Audio-suppression signal answer that closes OQ-BR-01.
- Cross-review row C-R9-W8 closure wording deliverable to
  `card-animations.md` via a follow-up GDD edit (the wording is published
  here; the actual `card-animations.md` edit is out of scope per task
  Do-Not-Modify list — see Migration Plan §6).
- A named follow-on story slug compatible with the current Sprint plan.
- Non-claims explicitly recorded so this ADR does not get cited as
  evidence for behaviour it does not enable.

## Decision

### Part 1 — Recipient scope: parallel unicast of the full alive-objectives reveal_set to BOTH players

`S2CSangMepriseReveal` is delivered as **two separate unicasts**, one per
session player, each carrying the **same full alive-objective `reveal_set`**
(both players' alive objectives, `is_fake` per slot).

```rust
// Server, on Sang Méprise play during RESOLUTION sub-step 1 (live-fire,
// future story — pseudocode for the contract, NOT implemented in this ADR):
let reveal_set: Vec<(u8, bool)> = active_sang_meprise_pairs(world, /* any */ recipient);
for recipient in session_players(world) {
    sender.send::<S2CSangMepriseReveal, ReliableChannel>(
        S2CSangMepriseReveal { identities: reveal_set.clone() },
        server,
        &single(peer_for(recipient)),
    );
    reconnect_tracker.sang_meprise_sent_to.insert(recipient);
}
sang_meprise_active = true;
```

**Why this resolves the three-way contradiction:**

- **Master GDD** (`lanes-and-lies-gdd.md:713-714`) is authoritative on
  card semantics: "visible to everyone." Both players seeing the same set
  is the player-facing truth.
- **Class System** (`class-system.md:243-262`) supplies the worked
  example: `reveal_set` is `{ alive_slots(A) ∪ alive_slots(B) }` and is
  sent to both players via parallel unicasts. This is canonical and is
  explicitly noted as "the formula is authoritative; prior session text
  suggesting a directional per-player vector was an error" (`:262`).
- **Network Protocol GDD** (`network-protocol.md:146`) says "Unicast
  (opponent)" — this is now stale and must be amended to "Parallel
  unicast (both players)". The amendment is published here; the actual
  edit lives outside this ADR's allowed file set and will be carried by
  the follow-up GDD-touch story (see Migration Plan §5).
- **ADR-001** (`adr-001-objective-identity-unicast.md:59`) says "the
  opponent only." This ADR **supersedes that one sub-clause** for Sang
  Méprise specifically — see Migration Plan §1. ADR-001's primary
  decision (do not replicate `ObjectiveIdentity` as a component; use
  targeted unicast for hidden identity) remains Accepted; only the Sang
  Méprise §5 sub-clause is narrowed to "opponent only" → "both players,
  full reveal_set."
- **Objective System GDD** (`objective-system.md:228`) carries the same
  stale "opponent only" wording — same Migration Plan §5 amendment.

**Why parallel unicast instead of true broadcast:** parallel unicast
(N=2 `NetworkTarget::Single` sends in 1v1; N=4 in 2v2) keeps the wire
shape identical to the existing reconnect-restore path
(`reconnect.rs:481-485` already uses `single(*peer_id)`), preserves the
per-recipient bookkeeping in `ReconnectTracker.sang_meprise_sent_to`
(needed for "did this player receive it before disconnect?" answers),
and means the future Dé du Chateux single-lane reveal (NP-4) can reuse
the same recipient model with a narrower payload without introducing a
broadcast/unicast dichotomy. Bandwidth cost is negligible — at most
10 entries × ~3 bytes ≈ 30 bytes per recipient per RESOLUTION.

### Part 2 — Send trigger and sub-step ordering

| Trigger | Where | Action |
|---|---|---|
| Sang Méprise card is committed at PLACEMENT and resolves at RESOLUTION sub-step 1 | Live-fire producer (future Class-System story; NOT implemented in this ADR) | (a) Set `sang_meprise_active = true`; (b) for each player in session, `sender.send::<S2CSangMepriseReveal, ReliableChannel>(reveal_set, single(peer))` AND `ReconnectTracker.sang_meprise_sent_to.insert(player)`. Performed once, atomically, before any other sub-step 1 effect that depends on identity visibility. |
| Both players play Sang Méprise in the same RESOLUTION (idempotency) | Live-fire producer | The second resolution is a **no-op for reveal dispatch**: if `sang_meprise_active == true`, do NOT re-send the message and do NOT re-insert into `sang_meprise_sent_to` (already present is fine). The second card's mana cost is paid; reveal state is set-once-per-RESOLUTION. This matches `class-system.md:451` and the existing reconnect-tracker shape (idempotent `HashSet::insert`). |
| Reconnect mid-RESOLUTION while `sang_meprise_active` | `reconnect.rs:222-224` calling `sang_meprise_reveal_message(world, player_id)` at `:998-1006`, dispatched via `:481-485` | The reconnecting player receives the **same `reveal_set`** that the original live unicast carried — built fresh from `active_sang_meprise_pairs` so destroyed slots are correctly excluded by the time of reconnect. `sang_meprise_sent_to` was already populated when the live unicast fired, so the reconnect branch correctly fires. No double-send: live unicast happened before disconnect; reconnect replays it. |
| RESOLUTION ends (RSM exits sub-step 6 or transitions to next phase) | Server | `sang_meprise_active = false`; `ReconnectTracker.sang_meprise_sent_to.clear()` for ALL players in the session. This MUST happen as part of the same RSM transition that clears `current_phase`. Cleanup MUST run before any next-round message is queued. Matches `class-system.md:610` (CS-AC-14a) and `objective-system.md:101` (Rule 6 batching). |
| Session teardown / game over | `server/src/core/session/system.rs:1641` (already implemented: `.remove(player)`) | Per-player removal of `sang_meprise_sent_to` entries is already wired and need not change. |

### Part 3 — Server state mutation contract

```
sang_meprise_active : bool                              [Resource owned by Objective System]
    set true  : sub-step 1 of RESOLUTION, on first Sang Méprise resolution this RESOLUTION
    set false : RESOLUTION-end (RSM transition out of sub-step 6, or next-phase transition)

ReconnectTracker.sang_meprise_sent_to : HashSet<PlayerId>   [server/src/core/session/state.rs:151]
    insert(P) : at the moment S2CSangMepriseReveal is unicast to P (live OR reconnect-restore)
    clear()   : at RESOLUTION-end, AS PART OF the same write that sets sang_meprise_active = false
    remove(P) : on session teardown (already wired at session/system.rs:1641)
```

**Invariant**: `sang_meprise_active == true` IFF `sang_meprise_sent_to`
is non-empty AND the current phase is RESOLUTION. Violating this is a
server bug; it should be asserted in any new test that touches the
reveal path. The `clear()` at RESOLUTION-end is essential — without it,
the next round's reconnects would replay a stale reveal.

The live-fire producer is the only path that can set
`sang_meprise_active = true`. The reconnect-restore path MUST NOT mutate
either flag — it only reads `sang_meprise_sent_to` to decide whether to
re-send; if a player was originally a recipient, they remain one across
the reconnect cycle. This is already how `sang_meprise_reveal_message`
behaves (`:998-1006`).

### Part 4 — Snapshot-restore contract (reaffirm; NO wire change)

`S2CGameSnapshot.active_sang_meprise_reveals: Option<Vec<ObjectiveReveal>>`
(`shared/src/protocol.rs:842`) remains the canonical reconnect-restore
field. The reconnect handler at `server/src/core/session/snapshot.rs:78-81`
(per PROMPT 1299) populates it via the existing
`active_sang_meprise_reveals(world, recipient)` helper at
`reconnect.rs:1008-1032`. CS Edge Case at `class-system.md:484` is the
human-facing closure of OQ-CS-2; this ADR re-affirms that closure.

The client snapshot consumer (future drain story) MUST:

- When `snapshot.active_sang_meprise_reveals` is `Some(reveals)` →
  populate `ObjectiveIdentityCache` with every `(player_id, lane, is_fake)`
  entry. Treat this as semantically equivalent to having received the
  live `S2CSangMepriseReveal` message.
- When `snapshot.active_sang_meprise_reveals` is `None` → leave
  `ObjectiveIdentityCache` empty for this RESOLUTION. The board renders
  in the "unrevealed" state, matching `class-system.md:562` ("Client must
  render gracefully without the overlay (objectives appear as unknown
  state)").
- The snapshot path and the live-message path MUST converge on the same
  cache state. If both fire in the same session (unicast arrives before
  snapshot, or after), the union of identities is correct because the
  cache key is `(PlayerId, Lane)` and the value is the same `bool` in
  both code paths.

### Part 5 — Client `ObjectiveIdentityCache` lifecycle

```rust
// client/src/presentation/board_rendering/...  (future S18 drain story)

#[derive(Resource, Debug, Default)]
pub struct ObjectiveIdentityCache {
    pub identities: HashMap<(PlayerId, Lane), bool>,  // (player, lane) -> is_fake
}
```

**Lifecycle rules (authoritative — implementer guidance for the future
drain story):**

| Event | Cache mutation |
|---|---|
| `S2CSangMepriseReveal` arrives via `MessageReceiver<S2CSangMepriseReveal>` in `client/src/` | Insert/overwrite every entry in `identities`. The wire payload carries the full alive-objectives set; treat each entry as authoritative. |
| `S2CGameSnapshot.active_sang_meprise_reveals: Some(...)` arrives via snapshot consumer | Insert/overwrite every entry in `identities` using the `(player_id, lane, is_fake)` tuples from the snapshot field. Same semantics as live arrival. |
| `S2CPhaseChanged` to any phase other than RESOLUTION | Clear `identities` (full `HashMap::clear`). This implements the "cleared at RESOLUTION end" rule from `objective-system.md:228` and `class-system.md:261` and CS-AC-14a. |
| `S2CGameSnapshot.active_sang_meprise_reveals: None` arrives (and the snapshot's `phase` is NOT RESOLUTION OR is RESOLUTION but with no active reveal) | Clear `identities`. Reconnect mid-non-RESOLUTION should never inherit a stale cache. |
| RESOLUTION animation drains to completion and the client transitions out of RESOLUTION render state (per `board-rendering.md` Rule 9 / `BoardRenderSet::ResolveStateMachine`) | The phase-change handler above is the cache-clear authority. No separate animation-completion clear is required. |

**Read scope (ADR-001 isolation invariant — STRICT):**

- The ONLY consumer that may read `ObjectiveIdentityCache.identities` is
  the audio-suppression branch defined in Part 6.
- The board renderer (`Rule 12` in `board-rendering.md:269`) MUST NOT
  read this cache when deciding standing-objective sprite, glyph, or HP
  bar fill — all standing objectives render identically.
- AC `BR-19` (ComponentId set equality across all standing objective
  entities) must remain green after the drain story lands. If the drain
  story violates the isolation invariant, that AC will catch it.
- A separate Sang Méprise "reveal overlay" UI element (per
  `class-system.md:559-562`) MAY read the cache to render the per-slot
  real/fake glyph, but it MUST be a distinct presentation surface from
  the standing-objective renderer and MUST hide itself on cache-empty
  state.

### Part 6 — Audio-suppression / OQ-BR-01 answer

**Decision: The `ObjectiveIdentityCache` IS the suppression signal. No
new wire field, no new replicated component, no new S2C message.**

The audio-suppression branch in the Board Rendering reveal pipeline
(`design/gdd/board-rendering.md:103` reference from
`objective-system.md`; rendered at the moment `ObjectiveDestroyed`
fires its reveal beat) MUST:

```
on ObjectiveDestroyed { lane, was_fake } during RESOLUTION reveal:
    let already_known = ObjectiveIdentityCache
        .identities
        .get(&(defender_player_id, lane))
        .map(|cached_is_fake| *cached_is_fake == !was_fake_inverted_to_is_fake(was_fake))
        .unwrap_or(false);
    if already_known {
        // Surprise sting suppressed; standard reveal tween still plays
        // (the visual 500ms hold + reveal animation is still required
        // per objective-system.md Rule 6 reveal-moment contract — only
        // the audio sting and any "surprise" particle layer are gated).
    } else {
        // Default path: play surprise sting + reveal tween.
    }
```

(The exact predicate `cached_is_fake == is_fake` reduces to a presence
check because the cache only stores entries during RESOLUTION while
Sang Méprise is active — see the lifecycle in Part 5. An implementer
who prefers to gate on `identities.contains_key(&(player, lane))` is
correct; the boolean equality check above is shown for clarity.)

**Why this answer:** every required input is already in the cache as a
side-effect of the reveal-mechanism contract. Adding a separate
`SangMepriseActive` replicated component or an `is_already_known: bool`
field in `S2CResolutionEvent::ObjectiveDestroyed` would duplicate the
cache content on the wire and create a second source-of-truth that
could go out of sync (the cache might be cleared on phase change while
a still-in-flight `ObjectiveDestroyed` event carries `is_already_known
= true`). The cache is already RESOLUTION-scoped and per-player; using
it as the signal is the simplest correct answer.

**Closes OQ-BR-01.** The cross-doc owner field for OQ-BR-01 (Network
Protocol GDD + Keyword System GDD) is satisfied by this ADR: NP GDD
contributes the wire format (no change), KS GDD contributes nothing new
(suppression is a Board Rendering / Card Animation concern, not a
keyword runtime state). The closure publishes here; the actual GDD edit
to flip `(OPEN)` → `(RESOLVED — ADR-024)` lives outside this ADR's
allowed file set and is covered by the cleanup story (Migration Plan §5).

### Part 7 — Card-animations cross-review C-R9-W8 closure wording

The ADR resolves cross-review row C-R9-W8 in
`design/gdd/gdd-cross-review-2026-04-30-r9.md:135-137`. The required
addition to `design/gdd/card-animations.md` is:

> **Edge Case — Sang Méprise active during ObjectiveDestroyed reveal.**
> When `ObjectiveIdentityCache` (per `design/gdd/board-rendering.md:75`
> and ADR-024 Part 5) contains an entry for `(defender_player_id,
> destroyed_lane)`, the destruction reveal animation is split:
> - The standard 500ms hold + reveal tween (per
>   `objective-system.md` Rule 6 reveal-moment contract) STILL PLAYS in
>   full. The visual beat is unconditional.
> - The "surprise" audio sting and any associated "surprise" particle
>   layer ARE SUPPRESSED. The attacker already knew the identity; the
>   surprise cannot fire twice.
> - When the cache has no entry (Sang Méprise not active, or the cache
>   was cleared by a phase transition), play the standard
>   surprise-included reveal.
>
> See ADR-024 §6 for the suppression-signal rationale and §5 for cache
> lifecycle.

The actual insertion into `card-animations.md` is outside this ADR's
allowed file set per the task Do-Not-Modify list (`card-animations.md`
sits under `design/gdd/` but is not in the explicit allowed-modification
list). The wording above is the contract the cleanup story must
faithfully copy.

### Part 8 — Future test surface and follow-up story slug

The follow-on story that removes the `S2CSangMepriseReveal` allowlist row
is named:

```
S18-PROTO-SANG-MEPRISE-DRAIN-001
```

(superseding the Story-008-era placeholder `S14-PROTO-SANG-MEPRISE-DRAIN-001`,
which named a sprint that ended before the ADR was authored. Sprint 14
through Sprint 17 closed with the row still allowlisted; Sprint 18 is the
first sprint plan that exists after this ADR's authoring. The S14 slug in
`production/epics/lightyear-protocol-verification/story-008-protocol-orphan-drain.md`
and in the allowlist `follow_on` field stays as historical context — the
S18 slug is the live name going forward.)

Acceptance criteria the future story must satisfy:

1. A `MessageReceiver<S2CSangMepriseReveal>` is wired in `client/src/`
   under a presentation-layer system (per ADR-021), draining inside
   `BoardRenderSet::ReadMessages` (per `board-rendering.md:280`).
2. On drain, `ObjectiveIdentityCache.identities` is populated per Part 5.
3. The snapshot consumer also populates `ObjectiveIdentityCache` from
   `S2CGameSnapshot.active_sang_meprise_reveals` per Part 5.
4. `S2CPhaseChanged` to non-RESOLUTION clears the cache per Part 5.
5. The audio-suppression branch in Card Animations / Board Rendering
   reads the cache per Part 6 and the C-R9-W8 wording in Part 7.
6. The allowlist row at
   `tests/invariants/protocol_completeness_test.rs:296-309` is removed.
7. Integration test under `tests/integration/board_rendering/` (or
   equivalent presentation crate test directory) injects an
   `S2CSangMepriseReveal` into the client `MessageReceiver` and asserts
   `ObjectiveIdentityCache` mutation per Part 5, AND injects a
   `S2CPhaseChanged` non-RESOLUTION and asserts the cache is cleared.
   Test name (per `.claude/rules/test-standards.md`):
   `test_sang_meprise_reveal_populates_cache_on_resolution_drain` and
   `test_phase_change_to_placement_clears_sang_meprise_cache`.
8. Test naming MUST follow `[system]_[scenario]_[expected_result]`
   pattern; tests MUST follow the arrange / act / assert structure; tests
   MUST NOT depend on any other test or shared mutable state.

The future story does NOT need to implement the server live-fire
producer. The drain works end-to-end against the reconnect-restore
producer plus integration-test fixtures (which is how the current
`sang_meprise_sent_to` is populated in tests). The live-fire path is a
separate, larger Class-System / Combat-Resolution / spell-effect epic.

## Alternatives Considered

### Alternative A: Opponent-only unicast (status quo per ADR-001 §5, NP GDD :146, OS GDD :228)

- **Description**: keep `S2CSangMepriseReveal` as a single unicast to the
  player who did NOT play Sang Méprise. Caster gets nothing on the wire.
- **Pros**: matches existing ADR-001 text and three GDD lines verbatim;
  smallest wire footprint (one unicast instead of two).
- **Cons**: contradicts the master GDD ("visible to everyone") and the
  CS-5 worked example. Forces the caster's client to "know it played the
  card" via a side channel (which `S2CResolutionEvent` doesn't currently
  carry for spell effects per NP-5) to render the reveal overlay on its
  own board. The caster's `ObjectiveIdentityCache` would not be
  populated by the live message — only by snapshot replay if the caster
  reconnects. This is an asymmetry that future implementers would have
  to special-case (each Sang Méprise consumer needs an "am I the caster?"
  branch).
- **Estimated Effort**: equal (no code change needed today; only the
  client drain is downstream).
- **Rejection Reason**: contradicts authoritative card semantics in the
  master GDD and the CS-5 worked-example formula. Adopting it would
  freeze a known-stale design and require a future override.

### Alternative B: Parallel unicast to both players, full reveal_set (chosen)

- **Description**: see Decision §1.
- **Pros**: matches master GDD and CS-5 verbatim. Preserves existing
  wire shape (`single(*peer_id)` reused). Symmetric — both clients use
  the same code path. Uniform with the reconnect-restore path. Extensible
  to future per-player narrower reveals (Dé du Chateux NP-4) by changing
  the payload size, not the dispatch shape.
- **Cons**: requires a Network Protocol GDD edit (`:146` line) and an
  Objective System GDD edit (`:228` line) to amend the "opponent only"
  wording. Carried by the cleanup story (Migration Plan §5), not by this
  ADR.
- **Estimated Effort**: zero code change today; the future drain story is
  the same regardless of recipient model.
- **Rejection Reason**: n/a (chosen).

### Alternative C: True broadcast via `NetworkTarget::All`

- **Description**: send one `S2CSangMepriseReveal` to all peers using a
  broadcast target instead of two unicasts.
- **Pros**: theoretically fewer dispatch calls (1 vs 2 in 1v1).
- **Cons**: requires forking the dispatch shape from
  `single(*peer_id)` to `NetworkTarget::All` only for this one message
  type. Breaks symmetry with the reconnect-restore path which is
  inherently per-recipient. Breaks the per-recipient bookkeeping in
  `ReconnectTracker.sang_meprise_sent_to` (which one HashSet entry would
  correspond to a broadcast? all of them? would `clear()` semantics
  change?). In team modes (2v2/3v3), the reveal is intentionally
  team-aware via the per-recipient flag — broadcast loses that
  granularity if a future card adds team-scoped reveals. Bandwidth
  savings (one fewer 30-byte message) are negligible.
- **Estimated Effort**: low but disruptive — every reconnect-restore
  invariant test would need to be re-validated against the new dispatch
  shape.
- **Rejection Reason**: trades a real architectural invariant
  (per-recipient bookkeeping uniform across live + restore) for an
  imperceptible bandwidth win.

### Alternative D: Audio-suppression via a new `S2CResolutionEvent` field

- **Description**: add `is_already_known: bool` to
  `S2CResolutionEvent::ObjectiveDestroyed` and let the client read THAT
  instead of the cache.
- **Pros**: explicit per-event signal; no dependency on cache state
  having survived intervening events.
- **Cons**: duplicates information already present in the cache and on
  the server (which knows `sang_meprise_active`). Adds a wire-protocol
  bit to every `ObjectiveDestroyed` event (4× per RESOLUTION worst case).
  Creates a second source-of-truth that can desync (cache cleared by
  phase change while in-flight event carries stale `true`). Forces a
  protocol amendment now, before the live-fire producer exists, with no
  way to integration-test it against the reconnect-restore path.
- **Estimated Effort**: high (NP GDD amendment + wire protocol change +
  server producer + integration tests + migration of any pending
  reconnect snapshots).
- **Rejection Reason**: the cache is already the right surface; this
  would be over-engineering.

## Consequences

### Positive

- The three-way recipient contradiction is closed with a single,
  GDD-aligned answer.
- The future drain story is implementation-ready: the recipient model,
  cache lifecycle, snapshot convergence, and audio-suppression signal
  are all named.
- OQ-BR-01 closes without a wire protocol change and without a new
  replicated component.
- Cross-review row C-R9-W8 has a publishable closure wording for
  `card-animations.md`.
- The pending-ADR registry shrinks by one entry, and the
  `S2CSangMepriseReveal` allowlist row gets a clear "implementation
  story is `S18-PROTO-SANG-MEPRISE-DRAIN-001`" follow-on instead of
  "story file authoring pending."
- The reconnect-restore path needs no changes — already correct.

### Negative

- A follow-up GDD cleanup story is needed to bring
  `network-protocol.md:146`, `objective-system.md:228`, and
  `adr-001-objective-identity-unicast.md:59` into line with the chosen
  recipient model. That work is paperwork-only, but it is unavoidable.
- The live-fire server producer is still missing. This ADR explicitly
  does NOT solve that — it is a Class-System / Combat-Resolution epic.
  Until the live-fire path lands, the reveal can only be triggered via
  integration-test fixtures or by manually inserting into
  `ReconnectTracker.sang_meprise_sent_to`. No player-visible
  functionality changes.

### Neutral

- The `ObjectiveIdentityCache` resource type is named in
  `board-rendering.md:75` but does not yet exist in `client/src/`. The
  future drain story creates the resource and the lifecycle systems;
  this ADR commits to the contract those systems will satisfy.

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| Future drain story implements recipient model A (opponent-only) by reading the stale ADR-001 §5 / NP-GDD line | MEDIUM | Caster's client misses the live reveal; only reconnect restores it | This ADR is cited as authority in the future story's "GDD Requirements Addressed" table; ADR-024 supersedes ADR-001 §5 for Sang Méprise; control-manifest update in cleanup story. |
| `ObjectiveIdentityCache` clear-on-phase-change race with in-flight `ObjectiveDestroyed` reveal animation | LOW | Surprise sting fires for an already-known reveal | Cache clear runs in `BoardRenderSet::ReadMessages` (per `board-rendering.md:280`) before any `ObjectiveDestroyed` is consumed in the same frame; AC in future story asserts ordering. |
| Server live-fire producer is implemented later but fails to insert into `ReconnectTracker.sang_meprise_sent_to` | MEDIUM | Reconnect during active Sang Méprise drops the reveal (no restore) | This ADR's Decision §2 explicitly lists both writes as atomic per dispatch; future live-fire story PR must include a test that disconnect-immediately-after-Sang-Méprise restores via snapshot. |
| Cleanup story to amend NP GDD / OS GDD / ADR-001 §5 wording is skipped | LOW | Documentation drift; stale "opponent only" text continues to mislead readers | This ADR's Migration Plan §5 names the exact lines to amend; pending-ADR-registry entry is converted to a pointer at this ADR so the next `/architecture-review` flags the residual cross-GDD inconsistency. |
| Live-fire epic never lands, allowlist row removed prematurely | LOW | Drain wired against producer that exists only in reconnect-restore + test fixtures; an unallowlisted client would pass tests because the reconnect path can still feed it | Future story's AC #6 (remove allowlist) MUST be paired with AC #7 (integration test of cache mutation); the test fixture path is sufficient evidence per Story-008 Path C's deferral logic. |

## Performance Implications

| Metric | Before | Expected After | Budget |
|--------|--------|---------------|--------|
| Server CPU per Sang Méprise resolution | N/A (no live producer) | 2× `sender.send::<S2CSangMepriseReveal, _>` + 2× `HashSet::insert` | < 0.1ms (negligible — fewer ops than a typical placement-reveal dispatch) |
| Client CPU per `S2CSangMepriseReveal` drain | N/A (no drain) | Up to 10× `HashMap::insert` | < 0.05ms |
| Network — per RESOLUTION when Sang Méprise active | 0 bytes (no live producer) | ~60 bytes (2 recipients × ~30 byte payload) | < 1 KB / round budget (ADR-008 / technical-preferences.md) |
| Network — reconnect during active Sang Méprise | already implemented | unchanged (snapshot field already carries the data; live unicast replays once) | same as above |
| Memory — `ObjectiveIdentityCache` on client | N/A | At most 10 entries × ~16 bytes = ~160 bytes per session | < 256 MB WASM heap budget |
| Load time | unchanged | unchanged | n/a |

## Migration Plan

1. **No code or test change in this ADR.** Acceptance is the only
   action this ADR commits to.

2. **`.claude/docs/technical-preferences.md` ADR registry**: remove
   "Sang Méprise reveal mechanism" from the pending list at line 81; add
   ADR-024 to the Architecture Decisions Log table. This file IS in the
   allowed-modification list — handled in this ADR's commit.

3. **`design/gdd/board-rendering.md` OQ-BR-01**: flip from `(OPEN)` to
   `(RESOLVED — ADR-024)` and add the one-line reference to ADR-024 §6.
   This file IS in the allowed-modification list — handled in this ADR's
   commit per the task's "if the local convention permits" clause; the
   project convention is that ADRs do close GDD OQs via authoring (see
   ADR-018's OQ-NP-1/OQ-NP-5/OQ-KS-2 closure in
   `class-system.md`/`network-protocol.md`).

4. **Sprint-status / session-state / stage**: NO changes. This ADR does
   not advance the project stage; Sprint 17 remains active; Sprint 18 is
   the prospective home for the follow-on drain story but this ADR does
   not activate it.

5. **Cleanup story (out of scope here; SEPARATE future paperwork
   prompt)**: amend the following lines to match this ADR's recipient
   model:
   - `design/gdd/network-protocol.md:146` — "Unicast (opponent)" →
     "Parallel unicast (both players); see ADR-024 §1."
   - `design/gdd/objective-system.md:228` — "targeted reliable unicast
     to the opponent only (Option B — see OQ5)" → "parallel unicast to
     both players (full alive-objective `reveal_set`); see ADR-024 §1."
   - `docs/architecture/adr-001-objective-identity-unicast.md:59` — add
     a "**Superseded for Sang Méprise**: see ADR-024 §1" note
     immediately after the existing §5 paragraph. Do NOT rewrite ADR-001
     §5 — only annotate.
   - `design/gdd/card-animations.md` — insert the Edge Case wording from
     this ADR §7 (verbatim).
   - `design/gdd/gdd-cross-review-2026-04-30-r9.md` row C-R9-W8 — flip
     status from "Warning" to "RESOLVED — ADR-024".

6. **Implementation story (out of scope here; SEPARATE Sprint-18
   prompt)**: author and dispatch `S18-PROTO-SANG-MEPRISE-DRAIN-001`
   per the acceptance criteria in this ADR §8.

**Rollback plan**: If a future engine/transport change makes parallel
unicast unsafe (e.g., Lightyear ≥ 0.30 adds a per-message broadcast
primitive that supersedes the current `NetworkTarget` API), supersede
this ADR with a new ADR that ratifies broadcast — do not amend this one
in place. The reconnect-restore path is the canonical contract; it must
keep working through any supersession.

## Validation Criteria

- [x] All eight required design decisions enumerated in task PROMPT-1302
      §"Required design decisions" are answered explicitly above.
- [x] The recipient-model contradiction between `network-protocol.md:146`,
      `class-system.md:123/243-262`, and `lanes-and-lies-gdd.md:713-714`
      is resolved with reference to the authoritative source (master
      GDD + CS-5 worked example).
- [x] Server-state mutation contract names the writer, the trigger, and
      the cleanup site for both `sang_meprise_active` and
      `ReconnectTracker.sang_meprise_sent_to`.
- [x] Snapshot-restore contract reaffirms the existing
      `S2CGameSnapshot.active_sang_meprise_reveals` field; no new wire
      field; no schema change.
- [x] Client cache lifecycle is fully specified, with five distinct
      events and their cache mutations.
- [x] OQ-BR-01 has an explicit answer with rationale (Decision §6).
- [x] C-R9-W8 closure wording is publishable verbatim into
      `card-animations.md` (Decision §7).
- [x] Follow-on story slug is named (`S18-PROTO-SANG-MEPRISE-DRAIN-001`)
      with full acceptance criteria for the test surface (Decision §8).
- [x] Non-claims are recorded (see Non-Claims below).
- [x] No code changes, no tests added or modified, no Cargo invocation,
      no sprint-status / session-state / stage edits, no Sprint 18
      activation, no QA plan, no `main` push.
- [x] `S2CSangMepriseReveal` allowlist row at
      `tests/invariants/protocol_completeness_test.rs:296-309` remains
      in place; this ADR does not touch it.

## Non-Claims

This ADR makes none of the following claims:

- **No implementation**. No `MessageReceiver<S2CSangMepriseReveal>` is
  added to `client/src/`. No drain code is written. No server-side
  live-fire producer is added. No tests are added or modified.
- **No protocol behaviour change**. The wire shape of
  `S2CSangMepriseReveal`, the snapshot field
  `S2CGameSnapshot.active_sang_meprise_reveals`, the reconnect-restore
  dispatch in `server/src/core/session/reconnect.rs`, and the
  per-recipient `ReconnectTracker.sang_meprise_sent_to` set all remain
  byte-for-byte identical after this ADR.
- **No release readiness or stage advance**. The project stage remains
  `Polish`. Sprint 17 remains `active`. Sprint 18 is not activated by
  this ADR.
- **No `card-animations.md` GDD edit in this commit**. The Edge Case
  wording in Decision §7 is the contract; the file edit is carried by
  the separate cleanup story in Migration Plan §5.
- **No `network-protocol.md` / `objective-system.md` / ADR-001 textual
  amendment in this commit**. The same Migration Plan §5 cleanup story
  owns those edits.
- **No QA sign-off, gate-check verdict, smoke-check pass, or test
  evidence is implied**. This ADR is a design artifact only.
- **No claim that the deeper server live-fire (Sang Méprise spell-effect
  → set `sang_meprise_active`) is solved**. That belongs to the
  Class-System / Combat-Resolution epic and is explicitly out of scope.

## GDD Requirements Addressed

| GDD Document | System | Requirement | How This ADR Satisfies It |
|-------------|--------|-------------|--------------------------|
| `design/gdd/lanes-and-lies-gdd.md:713-714` | Master GDD | "Reveals all 5 objectives of all teams (real and fake status visible to everyone) for the duration of this resolution phase." | Decision §1 (parallel unicast of full reveal_set to both players) makes "everyone" mean exactly the session's player set; cache lifecycle in §5 enforces "duration of this resolution phase." |
| `design/gdd/class-system.md:243-262` (CS-5) | Class System | Sang Méprise reveal_set formula; unicast to both players; per-RESOLUTION lifetime; cleared at RESOLUTION end | Decision §1 ratifies the CS-5 formula as authoritative; §2 defines send trigger at sub-step 1; §3 defines server-state cleanup at RESOLUTION-end; §5 defines client cache clear on phase change. |
| `design/gdd/class-system.md:451` (Concurrency edge case) | Class System | Both players play Sang Méprise in the same RESOLUTION → second is idempotent | Decision §2 specifies the idempotent no-op for the second dispatch and the `HashSet::insert` semantics for `sang_meprise_sent_to`. |
| `design/gdd/class-system.md:484` (Disconnect edge case) | Class System | Reconnect mid-RESOLUTION while Sang Méprise active → snapshot field carries reveal; no second unicast required (closes former OQ-CS-2) | Decision §4 reaffirms the snapshot-restore contract without change; explicitly preserves the reconnect path at `reconnect.rs:222-224, 481-485, 998-1006`. |
| `design/gdd/class-system.md:559-562` | Class System | Sang Méprise reveal overlay on board for RESOLUTION lifetime; gracefully render absent state on reconnect | Decision §5 (cache lifecycle) and §6 (read scope) define how the overlay system reads the cache and how a snapshot with `active_sang_meprise_reveals = None` produces the absent state. |
| `design/gdd/class-system.md:608-612` (CS-AC-13, CS-AC-14a, CS-AC-14b) | Class System | Unicast on play; flag clears at RESOLUTION end; opponent objectives re-hidden next PLACEMENT (ADVISORY) | Decisions §1, §2, §3, §5 collectively satisfy these ACs. The future drain story's integration tests are the verification surface. |
| `design/gdd/objective-system.md:101, 103` | Objective System | Reveal animation must be a distinct visual event; surprise sting suppressed when Sang Méprise was active | Decisions §6 and §7 (the Card-Animations Edge Case wording) close this. |
| `design/gdd/objective-system.md:228` (Sang Méprise edge case) | Objective System | `S2CSangMepriseReveal` semantics; reconnect gap | Decision §1 supersedes the "opponent only" sub-clause; §4 reaffirms the reconnect-restore path; cleanup story in Migration Plan §5 amends the stale line. |
| `design/gdd/objective-system.md:244` (OS-24 edge case) | Objective System | `ObjectiveDestroyed { was_fake }` fires once even when Sang Méprise active | Out of scope for behaviour change but cited here because Decision §6 specifies that the destruction event is still authoritative — the cache only controls audio suppression, never event firing. |
| `design/gdd/board-rendering.md:75` | Board Rendering | `ObjectiveIdentityCache` resource exists for the Sang Méprise audio-suppression branch | Decision §5 defines the resource's lifecycle; Decision §6 defines its sole reader. |
| `design/gdd/board-rendering.md:271` | Board Rendering | ADR-001 isolation invariant: identity caches read ONLY by audio-suppression branch | Decision §5 "Read scope" enforces this restriction explicitly. |
| `design/gdd/board-rendering.md:877-879` (OQ-BR-01) | Board Rendering | "Sang Méprise suppression signal" — OPEN | Decision §6 answers this OQ; this ADR commit flips the OQ to RESOLVED. |
| `design/gdd/network-protocol.md:146` | Network Protocol | `S2CSangMepriseReveal` row | Decision §1 supersedes "Unicast (opponent)" → parallel unicast (both players). Cleanup story in Migration Plan §5 amends the line. |
| `tests/invariants/protocol_completeness_test.rs:296-309` | Protocol invariant | Allowlist row carries "pending ADR" rationale | This ADR's acceptance is the gating artifact named in the rationale; allowlist row stays until `S18-PROTO-SANG-MEPRISE-DRAIN-001` lands. |
| `production/epics/lightyear-protocol-verification/story-008-protocol-orphan-drain.md:501-534` | Story 008 Path C | Per-Orphan Decision: defer until reveal-mechanism ADR is Accepted | Acceptance of this ADR is the disposition Story 008 was waiting on; the follow-on slug update from `S14-` to `S18-` is recorded in Decision §8. |
| `design/gdd/gdd-cross-review-2026-04-30-r9.md:135-137` (C-R9-W8) | Cross-review | Sang Méprise + objective destruction reveal-suppression contract undefined in `card-animations.md` | Decision §7 publishes the verbatim closure wording for `card-animations.md`; cleanup story in Migration Plan §5 carries the edit. |

## Related

- **Supersedes (in part)**: `docs/architecture/adr-001-objective-identity-unicast.md`
  §5 — only the "to the opponent only" sub-clause for Sang Méprise.
  ADR-001's primary decision (do not replicate `ObjectiveIdentity` as a
  component; use targeted unicast for hidden identity) remains Accepted.
- **Depends on (Accepted)**: ADR-001, ADR-002, ADR-008, ADR-011.
- **Enabled by this ADR**: `S18-PROTO-SANG-MEPRISE-DRAIN-001` (client-side
  `MessageReceiver<S2CSangMepriseReveal>` drain).
- **Code references**:
  - `shared/src/protocol.rs:109, 692-695, 842` — wire types and channel
    registration.
  - `server/src/core/session/reconnect.rs:52-55, 195-196, 222-224,
    473-493, 998-1006, 1008-1032, 1034-1040` — reconnect dispatch
    enum, builder, sender, and helpers.
  - `server/src/core/session/state.rs:151` —
    `ReconnectTracker.sang_meprise_sent_to`.
  - `server/src/core/session/snapshot.rs:78-81` — snapshot populator.
  - `server/src/core/session/system.rs:1641` — teardown cleanup.
  - `tests/invariants/protocol_completeness_test.rs:296-309` —
    allowlist row.
- **Out-of-scope (separate epic)**: server live-fire producer for
  `sang_meprise_active = true` on Sang Méprise spell resolution
  (Class System / Combat Resolution); Dé du Chateux single-lane reveal
  (NP-4, separate message type per `class-system.md:711`).
