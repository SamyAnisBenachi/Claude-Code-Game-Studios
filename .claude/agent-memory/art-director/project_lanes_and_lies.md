---
name: Project — Lanes and Lies game overview
description: Core facts about the game that inform all AD decisions
type: project
---

1v1 online card/lane-push game. Players draft decks through open-ascending auctions, then simultaneously place cards hidden in 5 lanes. Combat resolves via a 6-step deterministic algorithm each round. Victory = destroy 2 of 3 real objectives (3 are fake — hidden information core mechanic).

**Tech:** Bevy 0.18 / Rust, WASM client (Vercel), headless server (Railway). bevy_tweening for animations.

**Art style:** Vibrant Ankama/Wakfu cel-shaded 2D — bold Void outlines, saturated local color, chibi 1:1.5 head-body ratio on board units, Krosmaga-derived card frame anatomy.

**Art bible:** `design/art/art-bible.md` — complete, 9 sections, AD sign-off pending.

**Key palette constants:**
- ATK = Orange `#E07020` (globally reserved)
- HP = Teal `#2AA8C4` (globally reserved)
- Damage numbers = Crimson Slate `#8B1A2F`
- Arcane Gold `#F5C842` = objectives/rewards
- Void `#0D0D14` = outlines

**Game pillars:** Simple surface · Deep emergence · No idle spectating · Auction as signature

**Why:** Shapes every AD decision around instant readability, no ambiguity, and making RESOLUTION feel theatrical.

**How to apply:** Always cross-reference pillar alignment when proposing visual systems. "No idle spectating" is the dominant pillar for combat/RESOLUTION decisions.
