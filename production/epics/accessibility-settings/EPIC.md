# Epic: Accessibility Settings

> **Layer**: Presentation
> **GDD**: `design/accessibility-requirements.md` + `design/ux/settings-accessibility.md`
> **Architecture Module**: `client/src/ui/settings/` and client-side preference resources
> **Status**: Ready - Sprint 6 accessibility foundation and photosensitivity audit stories drafted
> **Stories**: Stories 001 and 003 created 2026-05-05; follow-up stories listed for QA-COND-0005 closure planning

## Overview

Accessibility Settings owns the client-side Settings / Accessibility surface and
the local preference foundation required to close or disposition the remaining
QA-COND-0005 Standard-tier accessibility rows after GSS-008.

GSS-008 already implemented the server-authoritative PLACEMENT timer-extension
authority path. This epic does not replace that authority. It exposes the
already implemented multiplayer-safe values in UI, persists local preference
requests, and provides shared resources that later presentation stories consume
for colorblind modes, reduced motion, UI scale, input remapping, help/tutorial
persistence, brightness/audio controls, and browser accessibility evidence.

The epic is a Sprint 6 gate-remediation epic, not broad feature expansion.
QA-COND-0005 remains Open until all blocking accessibility rows are either
implemented and evidenced, reclassified, or accepted as risk by the producer.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|------------------|-------------|
| [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md) | Client Settings may emit C2S intent messages, but the server remains authoritative for multiplayer phase duration and game state. | HIGH |
| [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md) | Settings is a bevy_ui presentation surface that must compose through the client presentation layer without duplicating Lightyear drains or violating Bevy 0.18 UI patterns. | HIGH |
| [ADR-023: Placement Timer Accessibility Authority](../../../docs/architecture/adr-023-placement-timer-accessibility-authority.md) | PLACEMENT timer multiplier values are extension-only multiplayer-safe session settings: 1x, 1.5x, 2x, 3x; the active value is neutral, server-authoritative, and frozen at SessionReady. | HIGH |

## Source Requirements

| Source | Requirement |
|--------|-------------|
| `design/accessibility-requirements.md` | Standard-tier rows for colorblind mode, UI scaling, motion reduction, full input remapping, tutorial persistence, PLACEMENT timer extension, brightness/gamma, independent volume controls, and final browser evidence. |
| `design/ux/settings-accessibility.md` | Settings opens from title/lobby/safe contexts; unsafe phases queue a pause/settings request; Accessibility category exposes colorblind mode, reduced motion, PLACEMENT timer multiplier, menu UI scale, and HUD UI scale; preferences persist when storage is available. |
| `design/gdd/game-session-system.md` via GSS-008 | PLACEMENT timer multiplier negotiation is already implemented by GSS-008 and must be surfaced without reimplementing server authority locally. |
| `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md` | QA-COND-0005 can close only after Standard-tier gaps are implemented, verified, reclassified, or accepted as risk. |
| `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md` | Records remaining accessibility rows and identifies Settings / Accessibility foundation as the first implementation slice after GSS-008. |

## Scope

### In Scope

- Settings / Accessibility UI shell.
- Client preference resource and storage abstraction.
- Browser localStorage persistence path for preferences when available.
- Runtime fallback when storage is unavailable.
- Keyboard and focus baseline for the Settings panel.
- Neutral PLACEMENT timer selector UI for 1x, 1.5x, 2x, and 3x.
- Menu UI scale and HUD UI scale preference foundations.
- Colorblind mode and reduced-motion preference foundations.
- Follow-up story structure for remaining QA-COND-0005 rows.

### Out of Scope

- Reimplementing GSS/RSM timer authority from GSS-008.
- Full colorblind palette application and color-only gameplay backup audit.
- Full reduced-motion consumers across HUD, Hand UI, Shop/Auction UI, Board
  Rendering, and Card Animations.
- Full keyboard/mouse input remapping.
- Help/tutorial prompt registry and reset/replay system.
- Brightness/gamma rendering calibration and audio bus implementation.
- Final browser/WASM text-size, contrast, layout, and QA-COND-0005 closure
  evidence.

## Dependency Map

| Dependency | Existing Surface | Accessibility Settings Use |
|------------|------------------|-----------------------------|
| Game Session System Story 008 | `C2SSetPlacementTimerMultiplier`, `S2CSessionSettingsUpdated`, `PlacementTimerMultiplier`, frozen snapshot field | Settings sends allowed pre-SessionReady timer requests and displays neutral effective room/session value. |
| Presentation Layer Stories 001-002 | `PresentationPlugin`, `PresentationSet`, `CurrentClientPhase`, shared economy view | Settings composes as a client UI surface and reads phase/session state without duplicate S2C drains. |
| HUD / Hand UI / Shop-Auction UI | Existing presentation sub-plugins | Later stories consume UI scale, reduced-motion, colorblind, and focus preference resources. |
| Browser/WASM target | `web-sys` already present in `client/Cargo.toml` for wasm32 | Story 001 can implement localStorage-backed persistence without adding a new crate; Storage feature may need to be enabled. |

## Current Implementation Gaps

- No `client/src/ui/settings/` module exists.
- No general client accessibility preference resource exists.
- No persistent local preference abstraction exists.
- No Settings panel root, category navigation, focus model, or status footer exists.
- `SessionSettingsView` exists for neutral GSS-008 effective timer state, but no UI selector exposes player timer requests.
- `web-sys` is present with `Window`, but the wasm localStorage path may need the `Storage` feature added during implementation.
- No photosensitivity warning or flash-frequency audit evidence exists for A11Y-BS-03.

## Stories

| # | Story | Type | Status | QA-COND-0005 Impact | ADR |
|---|-------|------|--------|---------------------|-----|
| 001 | [Settings / Accessibility Foundation and Preferences](story-001-settings-accessibility-foundation-and-preferences.md) | UI | Ready | Reduces risk; does not close QA-COND-0005 alone | ADR-002, ADR-021, ADR-023 |
| 002 | Colorblind Modes and Color-Only Backups | UI / Visual | Planned | Closes colorblind and color-as-only indicator sub-rows when implemented and evidenced | ADR-021 |
| 003 | [Photosensitivity Warning and Flash Audit](story-003-photosensitivity-warning-and-flash-audit.md) | Config/Data | Ready | Supplies A11Y-BS-03 warning/audit evidence or producer reclassification after audit; does not close QA-COND-0005 alone | ADR-002, ADR-021 |
| 004 | Input Remapping and Hold Audit | UI | Planned | Closes or dispositions full input remapping and hold-to-press rows | ADR-002, ADR-021 |
| 005 | Help and Tutorial Persistence | UI | Planned | Closes tutorial persistence row | ADR-021 |
| 006 | Browser Accessibility Evidence and QA-COND-0005 Closure | UI / Config/Data | Planned | Direct closure story only after all implementation and producer dispositions exist | ADR-021, ADR-023 |
| 007 | Reduced Motion Consumers and Gameplay-Critical Visual Backups | Visual/Feel | Planned | Closes reduced-motion and visual-audio backup rows after preference foundation and audit disposition | ADR-021 |

## Definition of Done

This epic is complete when:

- Settings / Accessibility exists and is keyboard-operable from supported entry
  contexts.
- Accessibility preferences persist across browser refresh when storage is
  available and fail gracefully when unavailable.
- PLACEMENT timer selector exposes only 1x, 1.5x, 2x, and 3x for multiplayer
  Standard tier and never attributes the effective value to a player.
- Colorblind, reduced-motion, UI scale, input-remapping, tutorial, brightness,
  audio, and browser-evidence rows have implementation, evidence, producer
  reclassification, or accepted-risk disposition.
- Photosensitivity warning and flash-frequency audit evidence exists at
  `production/qa/evidence/accessibility-photosensitivity-warning-flash-audit-2026-05-05.md`,
  and A11Y-BS-03 has either warning implementation evidence or producer
  reclassification after audit.
- QA-COND-0005 is updated only after the closure evidence states that no
  unverified Standard-tier blocker remains.

## Sprint 6 Notes

Story 001 is the recommended first implementation slice because the remaining
Standard-tier accessibility work needs a shared preference foundation before
consumer systems can safely apply color, motion, scale, input, help/tutorial,
video, or audio preferences. Story 003 can run as an audit/readiness slice in
parallel with implementation planning because its mandatory deliverable is the
exact flash-audit evidence path and a producer disposition for A11Y-BS-03. Both
stories should run through `/story-readiness` before any `/dev-story` work
begins.
