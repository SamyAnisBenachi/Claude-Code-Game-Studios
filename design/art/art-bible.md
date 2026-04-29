# Art Bible — Lanes and Lies

> **Status**: All 9 Sections Complete · Art Director Sign-Off Pending
> **Last Updated**: 2026-04-29
> **Art Director Sign-Off**: Pending (lean mode — AD-ART-BIBLE skipped)
> **Visual Authority**: Reference images in `design/art-references/` are authoritative. Krosmaga card frames (`krosmaga-screenshot-1.jpg`, `krosmaga-screenshot-2.jpg`) are the canonical source for card anatomy. Text description is commentary.
> **References**: `design/art-references/` — 8 Ankama/Wakfu concept slides + 10 Krosmaga in-game screenshots + character sprite sheet

---

## 1. Visual Identity Statement

> **One-line visual rule:**
> Every asset must read as a bold, hand-illustrated game piece from the Ankama universe: confident black outlines, fully saturated local color, and enough visual weight to hold its own at 64px — never photorealistic, never muted, never decorative noise for its own sake.

### Supporting Visual Principles

#### Silhouette First
Every unit, objective, and UI element must be identifiable by shape alone, before color or detail is read.

**Design test:** When choosing between a pose with a clear weapon silhouette and a more dynamic pose that loses the weapon read at small scale, choose the clear silhouette.

**Pillar served:** Simple surface — a new player placing cards on round one cannot afford to misidentify a unit. Instant recognition at board scale is a rule, not a preference.

#### Color as Information
Saturated hues carry functional meaning — faction identity, team side, objective state, auction urgency — so color choices are never arbitrary.

**Design test:** When choosing an accent color for a UI state or unit border, check whether that hue already belongs to a faction or team. If it does, either reuse the meaning deliberately or pick a hue that is neutral to the faction palette.

**Pillar served:** No idle spectating — every idle moment is active intelligence gathering. Color must let a player parse the full board state in under one second without reading text labels.

#### Structured Luminance Hierarchy
The brightest point in any scene or screen is the most actionable element: the auction card in contest, the revealed objective, the unit currently resolving combat.

**Design test:** When two elements compete for brightness — for example an auction card and the background glow of the board — the element requiring a player decision wins. Background atmosphere steps down one stop.

**Pillar served:** Auction as signature — the contested card must visually dominate the screen the moment bidding opens. Luminance hierarchy is what makes the auction feel like a spotlight, not a menu.

---

## 2. Mood & Atmosphere

Each game state has a distinct emotional register. The palette system and lighting shifts (defined in Section 4) serve these targets directly. States must never feel interchangeable at a glance.

*Thermal arc: warm (LOBBY) → bright (DRAFT_INITIAL) → cool under pressure (DRAFT_AUCTION / PLACEMENT) → warm burst (RESOLUTION) → outcome-dependent (GAME_OVER). DRAFT_AUCTION and PLACEMENT use different tension languages — frequency vs. subtraction — so they don't feel like the same state twice.*

---

### LOBBY — *Anticipation*
**Emotional target:** Anticipation edged with appraisal — sizing up a threat before the first move.
**Lighting character:** Neutral-warm ambient; soft rim lighting on character portraits; low contrast, no drama yet. Late-afternoon tavern light.
**Atmosphere:** Composed · Watchful · Weighted · Deliberate
**Energy level:** Measured
**Mood carrier:** Character portraits lit with a single warm key light from below-left — a classic intimidating-subject angle. The opponent's class reveal uses a short desaturate-then-resaturate flash, making the reveal feel like a cold assessment.

---

### DRAFT_INITIAL — *Possibility*
**Emotional target:** Genuine excitement at an open hand of options — the moment before a plan crystallizes.
**Lighting character:** Bright, even, saturated. Full-daylight Wakfu field energy. High chroma, no shadows.
**Atmosphere:** Expansive · Hopeful · Vivid · Unhurried
**Energy level:** Contemplative
**Mood carrier:** The 9 cards are revealed with a staggered fan-in from a shuffled stack, each card catching a brief specular highlight on arrival — the classic tactile pleasure of a fresh draw.

---

### DRAFT_AUCTION — *Pressure*
**Emotional target:** The controlled panic of a public negotiation where every second costs.
**Lighting character:** Cool-to-neutral; higher contrast than DRAFT_INITIAL; background desaturates slightly as the bid climbs, isolating the price counter.
**Atmosphere:** Electric · Escalating · Exposed · Competitive
**Energy level:** Frenetic
**Mood carrier:** The gold counter pulses with a warm-gold bloom on each bid increment, contrasting against the cooling background — the money is the only warm thing left in the frame. Pulse frequency increases as bids climb toward the upper range.

---

### DRAFT_SHOP — *Calculation*
**Emotional target:** The quiet satisfaction of a craftsman arranging their tools.
**Lighting character:** Warm, intimate, low ambient; individual cards lit from above like a shopkeeper's display lamp. Low energy contrast after the auction.
**Atmosphere:** Quiet · Deliberate · Grounded · Restorative
**Energy level:** Contemplative
**Mood carrier:** Background slightly desaturated and vignette-darkened at edges, focusing all visual weight on the available cards — compositional closure after the open-space auction.

---

### PLACEMENT — *Dread*
**Emotional target:** The committed dread of sealing a choice you cannot take back.
**Lighting character:** Cool-dominant; shadows deepen; rim light on placed units is blue-white — clinical, isolated. Highest shadow-to-highlight contrast in the game.
**Atmosphere:** Tense · Secretive · Irreversible · Silent
**Energy level:** Tense
**Mood carrier:** Each card placed triggers a brief desaturate-on-arrival — color drains as the unit locks in, reinforcing finality. The opponent's side of the board is a solid dark shape: no information, no relief.

---

### RESOLUTION — *Spectacle*
**Emotional target:** The visceral satisfaction (or horror) of watching your plan execute without your help.
**Lighting character:** Dynamic — each lane activates in sequence with a warm burst of contrast, then settles. Back to full Wakfu saturation for combat hits. No sustained cool palette.
**Atmosphere:** Theatrical · Inevitable · Kinetic · Revealing
**Energy level:** Charged
**Mood carrier:** Lane-activation uses a sequential spotlight sweep left to right — a stage-reveal composition. Combat outcomes read via color: warm gold/orange for wins, cool blue-grey for losses. Damage numbers are large, bold, red — clean floating numerals, no particle effects. The player is an audience; make it worth watching.

---

### GAME_OVER — *Clarity*
**Emotional target:** Resolute finality — whether victory or defeat, the fog of war lifts completely.
**Lighting character:** Objective destruction uses a single high-contrast burst (overexposed bloom) then settles into full, even, unambiguous light. All hidden information revealed simultaneously.
**Atmosphere:** Conclusive · Unambiguous · Open · Earned
**Energy level:** Settled
**Mood carrier:** Fake objectives reveal their markers with an iris-open wipe — the deception is anatomized. Win state: warm gold fill light, saturated. Loss state: desaturated with a cool-blue grade. Both feel *final*, not harsh.

---

## 3. Shape Language

### 3.1 Unit Silhouette Philosophy

**Pillar: Simple surface** — a unit must be readable at ~48×64px on the board.

All units are built on the **triangle principle**: every silhouette resolves to a dominant geometric read from 3 meters away. The Krosmaga character sprite sheet confirms this across hundreds of units — warriors upward-thrust, archers lateral-draw, sorcerers raised-staff — each producing a distinct triangle orientation before any detail is visible.

| Type | Silhouette Rule | Dominant Shape |
|---|---|---|
| Minion | Humanoid with clear weapon gesture | Tall narrow triangle — upward or forward thrust |
| Structure | Squat, wide, anchored to ground plane | Inverted trapezoid — broad base, stable |
| Trap | Low-profile, ambiguous | Near-flat horizontal bar — reads as nothing until triggered |
| Field | Spread, area-covering, no central mass | Wide shallow oval — environmental, not a unit |
| Spell (card only) | Burst / radial energy | Circle or starburst — never humanoid |

Traps deliberately break the silhouette rule: they must fail to read as a threat at a glance. This is a Simple surface exception that serves the hidden-placement mechanic.

---

### 3.2 Card Frame Anatomy

**Source of truth:** Krosmaga Extension=1 card frames (see `design/art-references/krosmaga-screenshot-2.jpg` and `krosmaga-screenshot-1.jpg`).

Cards are **portrait-orientation with straight or very slightly rounded corners**. All stat badges are embedded gem shapes, not overlaid text.

| Element | Position | Shape | Color |
|---|---|---|---|
| Mana cost | Top-left | Diamond/gem badge | Class color background (red = Iop, blue = neutral, gold = Legendary) |
| ATK | Top-right | Orange diamond | Orange — always |
| HP | Below ATK, top-right | Teal/blue gem | Teal — always |
| MP | Small badge near stats | Small secondary gem | Secondary class color |
| Family label | Bottom strip | Narrow text band | Neutral parchment |
| Card border | Full edge | Straight-sided portrait | Class color |
| Art | Center fill | Full-bleed illustration | — |
| Keyword text | Bottom third | Clean sans-serif on neutral bg | Dark text on parchment |

**Color rule for stats:** ATK is always orange. HP is always teal. These two colors are owned by these stats globally — they must not be reused for any other semantic meaning in the UI system.

---

### 3.3 Board and Environment Geometry

**Pillar: Deep emergence from Simple surface** — the grid must feel like a strategic arena, not a spreadsheet.

The board uses a **neutral surface with strong directional language**:

- **Cell geometry:** Square tiles with shallow inset shadows — clean and uniform so unit color has no competition from the floor. The board surface is the stage; characters are the actors.
- **Movement arrows:** Red directional arrows are embedded in each cell, pointing toward the opponent's end. This is a core readability element confirming the lane-push direction at a glance, not explained by text.
- **Lane channels:** Subtle dividers between lanes — enough to show where one lane ends and another begins without creating visual noise.
- **Board edge stripe:** Player side has a color-coded stripe at the near edge (red in Krosmaga; adapted for Lanes and Lies per Section 4 player palette).
- **Objectives:** Egg or stone shapes on simple pedestals at the far end of each lane — vertically taller than units so they read as landmarks, not combatants.
- **Environmental framing (concept upgrade):** The concept slides show the board embedded in an organic Wakfu arena with stone archways, ivy, and torchlit borders. This is the visual upgrade target over Krosmaga's floating white platform — the functional board logic is identical; the framing makes it feel like a place.

---

### 3.4 UI Shape Grammar

**Pillar: No idle spectating** — UI must surface information before the player asks for it.

Two-register language: world shapes are organic/curved (Wakfu aesthetic); strategic-information shapes are geometric/angular (information-war fantasy).

- **HUD elements** (gold, mana, round counter): flat angular chips — horizontal pill shapes with beveled ends. Distinct from world geometry so they read as interface, not world object.
- **Reserve mana:** Blue diamond shape (matching Krosmaga's AP RESERVE diamond) — a deliberate visual echo since the 298 cards assume this iconography.
- **Auction panel:** Contested card centered and enlarged (~2× board card size) with a radiating gold particle frame during active bidding. Price counter uses the largest numeral on screen. Nothing else competes.
- **Objective status dots (HUD):** Circular, matching the pedestal motif. Five dots per side. During RESOLUTION, they pulse with the lane's outcome color. Circles carry no directional implication — they are status, not action.
- **class figurines:** Large 3D figurine on a circular pedestal OUTSIDE and BESIDE the board, 3–4× unit scale. HP counter displayed on the figurine base. These are identity objects and health indicators, not playable units.

---

### 3.5 Visual Hierarchy by Phase

**DRAFT_AUCTION:** Auction card is the hero shape — largest element, brightest, centered. Price counter is the largest numeral on screen. All other elements at 60–70% opacity.

**RESOLUTION:** Board is the hero. Lane channels brighten, combat animations are the dominant motion, objective dots pulse to full saturation. The player's eye is pulled to the end of contested lanes — exactly where the win condition lives.

**Constant across all phases:** Player gold counter is always visible, never occluded, never animated except when the value changes. It is the one resource affecting every decision in every phase.

---

## 4. Color System

### 4.1 Primary Palette

| Name | Hex | Role | Rationale |
|---|---|---|---|
| **Arcane Gold** | `#F5C842` | Objectives, rewards, premium UI chrome | Warm Wakfu gold — both real and fake objectives share it, preserving the deception. Gold = significance, not safety. |
| **Ink Blue** | `#1A2D5A` | Board surface, UI backgrounds, depth | Cool anchor opposite Gold. Keeps the warm/cool split defining the Ankama palette. |
| **Ivory** | `#F7F0DC` | Text, readouts, card parchment background | Warm white over Ink Blue; easier on WASM browser screens than pure `#FFF`. |
| **Crimson Slate** | `#8B1A2F` | Damage numbers, combat indicators, Sacrier class | Deep enough to read on gold backgrounds. Reserved for "things that hurt." |
| **Auction Amber** | `#E87C1E` | Auction escalation track, warning state | Sits between Gold and Red on the hue wheel — communicates rising heat without stealing from class colors. |
| **Void** | `#0D0D14` | Hard outlines, card silhouettes | Near-black with a cool blue cast. Keeps outlines from reading as flat black; reinforces the Ankama cel-shade convention. |
| **Prism White** | `#EEF4FF` | Prism units, magical effects, highlight rim | Cool-white with a slight blue tint. Reserved for high-value magical events so it always draws the eye. |

**Stat colors are global constants — never reused for other meanings:**
- **ATK = Orange** (`#E07020` — distinct from Auction Amber by hue)
- **HP = Teal** (`#2AA8C4`)
These own their colors across every card and every piece of UI.

---

### 4.2 Semantic Color Vocabulary

Define meanings before players import assumptions from other games.

| Color | In this game means... | Does NOT mean |
|---|---|---|
| Gold | "Game object of significance — objective, reward, bid token" | "Safe", "real", "better than blue" |
| Blue (Ink Blue) | "Neutral board space, my information zone, Xelor class" | "Cold", "bad", "player 2 always blue" |
| Red / Crimson | "Combat event just occurred", "Sacrier class", "damage number" | "Fake objective", "losing", "enemy" |
| Amber / Orange | "Auction is active and escalating" | "Iop class" (Iop uses more saturated/redder orange, distinguished by saturation) |
| ATK Orange | "This number is ATK" | Any other UI meaning |
| HP Teal | "This number is HP" | Any other UI meaning |
| Prism White | "Magical resolution, prism unit, phase transition" | "Victory", "player A side" |
| Void (near-black) | "Outline, containment, spatial boundary" | "Evil", "enemy" |

---

### 4.3 Player Side Colors

Player identity is communicated through **unit base ring color** and **side-panel chrome** — not board tile color. Board tiles stay neutral so neither player "owns" the ground.

**1v1:**

| Player | Base Ring | Board Edge Stripe | UI Chrome |
|---|---|---|---|
| Player A | Sky Blue `#3A8EDB` | Left / bottom | Left panel |
| Player B | Terracotta `#D45C22` | Right / top | Right panel |

**Colorblind backup (built in from day one):**
- Player A units: **circle** base ring
- Player B units: **diamond** base ring
Shape carries identity; color reinforces it.

**2v2:** Two teams share a tinted chrome (Team Warm = amber-gold tint, Team Cool = blue-silver tint). Individual players distinguished by a small player number glyph (P1–P4) on their unit bases.

---

### 4.4 Class Color Identity

Canonical Wakfu/Dofus class colors — these match player expectations from the source games and must not be reassigned.

| Class | Primary Color | Accent | Hex | Note |
|---|---|---|---|---|
| **Xelor** | Dark Purple-Blue | Clockface silver | `#2E1B6E` | Darkest class; purple differentiates from Ink Blue by hue |
| **Sacrier** | Blood Red | Bone white | `#8B1A2F` | Shares Crimson Slate — combat indicators use Sacrier's color intentionally |
| **Iop** | Warm Orange-Red | Flame yellow | `#E05A00` | More saturated and redder than Auction Amber; players learn the saturation difference |
| **Eniripsa** | Soft Pink-Violet | Healing green | `#C45FA0` | Only class using pink — immediately distinct |
| **Cra** | Forest Green | Quiver brown | `#2A6B3C` | Deep green — the one class using green; firmly "archer precision", not "objective is real" |

---

### 4.5 Auction Escalation Color Track

The auction UI uses a **temperature ramp** on the bid value display and the panel border. Communicates rising stakes through color and glow intensity — without text.

| Bid Range | Frame Color | Glow | Reading |
|---|---|---|---|
| 1–3 gold | Ink Blue | None | Cheap, low stakes |
| 4–6 gold | Auction Amber `#E87C1E` | Soft warm pulse | Interest rising |
| 7–9 gold | Deep Amber `#C45A00` | Medium pulse | Contested |
| 10+ gold | Crimson-Amber `#9C2000` | Strong pulse, frame flicker | High stakes, tense |

The ramp moves Blue → Amber → toward Crimson. It stops short of full red — full `#FF0000` is reserved for Sacrier combat events and damage numbers only.

---

### 4.6 Colorblind Safety

Three semantic color pairs require shape/icon backup because they carry critical game information:

| Pair | Risk | Backup |
|---|---|---|
| Player A (Blue) vs Player B (Terracotta) | Protanopia/Deuteranopia | Unit base shape: Circle vs Diamond |
| Sacrier (Red) vs Cra (Green) | Deuteranopia: classic red-green confusion | Class icon always shown on unit base; color is secondary |
| Auction Amber escalation vs default Blue UI | Tritanopia | Bid number always displayed in text; pulse animation reinforces escalation independent of color |

---

## 5. Character Design Direction

### 5.1 Chibi Proportion System

Board units use a **1:1.5 head-to-body ratio** — head one unit tall, body (torso + legs) 1.5 units tall. At a 64px canvas this places the head at ~26px: enough room for two distinct eye shapes and a readable mouth without a pixel budget for fine detail.

Rules:
- **Limbs are suggestion, not anatomy.** Arms are short, thick, and terminate in a mitten-hand or oversized weapon grip — no finger articulation at board scale.
- **Weapon or accessory oversized by 20–30%** relative to body. The weapon is the identity signal; it must outlast the character in a thumbnail.
- **No neck.** Head sits directly on shoulders. Neck detail disappears at 64px.
- **Eye size is the emotional register.** Large eyes convey the Wakfu "bright world" tone. Angry-tilted vs. wide-open is the full emotion vocabulary at this scale.

### 5.2 Class Visual Distinguishability

Color is the first signal (Section 4.4). These shape and attitude cues are the second — readable even when color is stripped:

| Class | Silhouette Cue | Accessory Language | Pose Attitude |
|---|---|---|---|
| **Xelor** | Clock dial halo behind head, widening the silhouette laterally | Oversized pocket watch, bandaged limbs | Hunched, conspiratorial — leaning forward with raised finger |
| **Sacrier** | Broad upper body, tattoo scarring visible as dark shapes | Anchor chain or heavy iron weapon, no armor | Aggressive, weight forward, chin down |
| **Iop** | Tall spiky hair adding 10–15px to vertical silhouette | Large two-handed sword taller than body | Wide stance, chest out, weapon planted |
| **Eniripsa** | Wings visible above shoulder line, asymmetric upper halo | Staff with oversized healing orb, medical satchel | Weight on back foot, one hand raised |
| **Cra** | Bow held laterally — strong horizontal cross-bar | Oversized quiver protruding above shoulder | Feet apart, side-on stance; drawn bow is always dominant |

**The test:** At 32px (half board scale), each class must be distinguishable by silhouette alone. Hat/hair toppers and held objects are the primary differentiation layer.

### 5.3 Card Art vs. Board Sprite Relationship

Card illustration and board sprite depict **the same character at two rendering registers**, not two different angles.

- Card art shows the same costume and weapon as the board sprite, with full painterly detail.
- Pose angle: card art = 3/4 front view, slightly downward-looking. Board sprite = front-facing (flattest read, best silhouette).
- Color identity must be **immediately mappable** — a player who has seen a card in hand must recognise the unit on board.
- **One-way fidelity rule:** Detail flows from card art down to sprite, never upward. A costume element that does not survive sprite simplification is cut from the card art design at concept stage.
- Card art and sprite design must occur **in parallel review**, not sequentially — the sprite silhouette test gates the card art approval.

### 5.4 class figurine Direction

Figurines stand outside the board at 3–4× unit scale. They are identity totems and HP gauges, not combatants.

- **Heroic adult proportions** (1:5 to 1:6 head-to-body ratio) — the scale break from chibi board pieces signals "this is the class archetype, not a unit."
- **3D pedestal:** Circular base with class symbol inset as raised emblem. HP counter on the pedestal face — large numeral, always legible.
- **Idle animation:** Slow 5–8s looping micro-animation (cape flutter, clock-hands ticking, wings breathing) with amplitude under 4px.
- **Damage response:** Brief forward-lean on HP loss. No screen shake — drama is localised to the figurine.
- Each pose embodies class fantasy: Xelor points at a watch face; Sacrier arms open, chin raised; Iop sword raised; Eniripsa holds a glowing orb outward; Cra has bow drawn skyward.

### 5.5 Expression and Pose Philosophy

**Register: exaggerated confidence, not neutral competence.**

Every idle pose communicates a point of view — the Sacrier is daring you to hit it; the Cra has already calculated the shot. This serves the "information war" fantasy.

- **No neutral stances.** Feet together, arms at sides is forbidden. Every idle pose has a lean, raised element, or weight shift.
- **Exaggeration threshold:** If the gesture does not read as a clear emotional beat at 64px, push it further.
- **Motion language:** Attack animations use smear frames (1–2 frames of stretched limb before impact) — the Ankama signature. Fast wind-up, instant hit, slow recover.
- **Traps are the exception:** Trap units use a deliberately boring, neutral ground-level pose. The one place "interesting pose = bad design."

---

## 6. Environment Design Language

### 6.1 Board Surface

Board surface is **dark cobblestone** — irregular, hand-laid stone slabs in a near-black charcoal tone (~`#1A1A22`), lightly cracked at the edges. Stone reads as permanent and arena-like, not as a gameplay grid.

Cells are defined by **blue neon lane lines**, not tile borders. Each lane is connected by a continuous glowing chain of diamond nodes running from the near-side objective to the far-side objective. Lane lines are the primary wayfinding system for push direction; the stone beneath is neutral backdrop.

**Lane dividers:** Subtle 1–2px darkening groove between lanes — enough to prevent units from appearing to float across lanes, not enough to create a cage-grid aesthetic.

**Board edge stripe:** Player A (bottom) gets a Sky Blue `#3A8EDB` stone inlay trim along the near row; Player B (top) gets Terracotta `#D45C22`. The inlay is part of the geometry, not an overlay.

**Movement arrows:** Red directional arrows embedded in each board cell, pointing toward the opponent's objective end. Core readability element — confirms lane-push direction at a glance without text.

### 6.2 Environmental Framing

The board is **embedded in a cobblestone forest arena** — not floating in void. A ring of weathered stone architecture frames all four sides.

| Element | Role |
|---|---|
| Torch posts (2–3 per side) | Warm point-light sources; define near/far boundary; establish arena depth |
| Stone arch corners | Anchor the board as a built space; block sightlines beyond the board |
| Ivy / foliage border | Organic softness against the structured stone; the Wakfu world bleeding in |
| Player faction banners (1 per side) | Sky Blue and Terracotta per Section 4.3. Identity without text. |
| Crystal accents (corner posts) | Blue ambient-light fill to balance warm torches |
| Background tree canopy | Distant silhouettes above arches at low contrast — implies world scale; never competes with board |

**Lighting split:** Warm torchlight on the near horizontal plane. Cool Ink Blue ambient from background and crystals. This warm/cool split mirrors Section 4 and reinforces the board as the lit stage.

**Technical constraint:** Environment framing must be **atlased and tiled** — not a single full-bleed painterly scene. A single 2048×2048 full-resolution environment illustration consumes the entire environment memory budget (see Section 8, TC-4).

### 6.3 Objective Design

Objectives are **teardrop-flame shapes on squat stone pedestals** — three-tier cylindrical cobblestone, approximately 1.5× unit height.

**Critical rule — visual identity parity:** Real and fake objectives must be **pixel-identical in silhouette, scale, pedestal geometry, and placement**. The only difference is the presence or absence of the flame animation. Players learn over multiple games which positions tend to be real — they never learn by looking at the art.

| State | Visual |
|---|---|
| Real objective | Animated flame in Arcane Gold `#F5C842`, inner swirl glyph, low warm glow halo |
| Fake objective | Stone-grey inert teardrop, same geometry, no glow, no animation. A small `?` glyph floats just above the tip — reads as "unknown," not "empty" |
| Destroyed | High-contrast Prism White burst, then pedestal cracks and dims to inert stone permanently |

### 6.4 Prism Cell Treatment

Prisms occupy specific cells (lanes 1/5, 2/4, lane 3). Visual: a **six-pointed star inset** etched into the cobblestone surface.

| Prism State | Visual |
|---|---|
| Available | Star fills with soft Prism White `#EEF4FF` glow, slow pulse (1 cycle / 2s) |
| Occupied (unit on it) | Glow suppressed to 30%; star outline remains under unit base |
| Consumed | Star fades to dark stone etch; permanently inert for that round |

The star is part of the floor geometry — no floating icon, no text label.

### 6.5 Prop Density Rules

**Target: medium-sparse.** Props frame and atmosphere; they must never intrude on the board play area.

| Zone | Density | Rule |
|---|---|---|
| Board interior (5×8 grid) | Zero props | Nothing except units, prism stars, and lane lines |
| Board edge (1-cell border) | Minimal | Lane endpoints and objective pedestals only |
| Arena ring (outside board) | Moderate | Torch posts, banners, crystals — no more than 10–12 total |
| Background (arches, foliage, sky) | Low-contrast only | All background elements at 40–60% opacity; must fail the "can I mistake this for a game object?" test |

**Silhouette rule:** No environmental prop may share a silhouette with any unit type. Run the triangle silhouette test (Section 3.1) on every new prop.

### 6.6 Board Theming

**Default theme: Cobblestone Forest** (defined by this section). Structurally, the environmental framing layer is visually decoupled from board surface and gameplay data — swapping it produces a new theme without touching board logic.

**Constraints for any alternate theme:**
1. Board surface stays dark and low-chroma (player attention stays on units)
2. Lane lines stay blue — recoloring requires AD approval (gameplay readability risk)
3. Objective pedestal geometry stays constant — only surface material changes
4. Player banner colors stay tied to player-side identity (Section 4.3)
5. Warm/cool lighting split is required in all themes

*Candidate themes (V2+):* Desert ruin (orange sandstone, braziers — confirmed by `unnamed.jpg`), Winter fortress (frost crystals, snow arches), Celestial observatory (star-field backdrop).

---

## 7. UI / HUD Visual Direction

*Merged from Art Direction (AD) and UX Alignment (UX). Conflicts resolved inline.*

### 7.1 HUD Layout Philosophy

Perimeter-ring layout: all persistent information lives at screen edges; board center is unobstructed.

| Element | Position | Rationale |
|---|---|---|
| Player gold counter | Top-left | Gold drives every decision; first number the eye must find |
| **Opponent gold counter** | **Top-right, equal visual weight to player gold** | Opponent gold is PUBLIC during DRAFT_AUCTION — both values are tactical information during the auction. Must render at identical size and weight. *(UX: never subordinate opponent gold during auction)* |
| Mana bar (current) | Bottom-left cluster | Consulted only during Placement; adjacent to hand |
| Reserve mana (blue diamond) | Bottom-left, right of mana bar | Distinct by shape (diamond) from mana bar (bar). Direct echo of Krosmaga AP RESERVE — familiar to source-game players |
| Objective status dots | Top-center strip, 5 per player mirrored | Horizontal strip: my 5 : their 5. **Dots must behave identically between real and fake slots before destruction** — any size, pulse, or animation difference leaks fake/real identity. *(UX flag)* |
| class figurine | Outside board, left (own) / right (opponent) | Identity landmarks flanking the board; HP on figurine base |
| Phase label + timer | Top-center, below objective dots | **Persistent text label always present** — phase transitions must not rely on animation alone. *(UX: motion-only phase signals are prohibited)* |

**Phase-specific elements** (fade in/out at fixed slot, never slide from off-screen):
- DRAFT_AUCTION: auction panel replaces board entirely (see 7.3)
- PLACEMENT: hand fully lit; board cells must be **visually distinct from RESOLUTION state** — different cell treatment, never reuse the same cell art for both phases *(UX flag)*
- RESOLUTION: all HUD dimmed to 70%; board is hero

### 7.2 Hand Card Display

**Row layout, not a fan.** At 10 cards, a fan makes stat badges unreadable.

- Cards at 75% of board-card scale in the hand tray
- **Hover:** scales to 100%, lifts 12px, full card front including keyword text visible. Adjacent cards compress slightly.
- **At 3–4 cards:** Row stays left-anchored; no ghost frames for empty slots
- **At 10 cards:** Cards compress to 90% spacing. ATK orange diamond and HP teal gem must remain legible at maximum overlap — these badges have a minimum-size floor
- **Selected card:** Lifts to full board-card scale with a gold outline pulse — visually committed before dropped on a lane

### 7.3 Auction Panel Design

The auction panel **replaces the board entirely** during DRAFT_AUCTION. The board disappearing is the signal that normal strategic time has stopped.

**Panel anatomy:**

| Element | Treatment |
|---|---|
| Contested card | 2× board-card scale, centered, full card art at maximum resolution |
| Gold particle frame | Radiates from card border outward; density and color follow Section 4.5 escalation ramp |
| **Price counter** | **Largest numeral on screen.** Arcane Gold with Void outline. Size ≥ 2× any other number on screen |
| **Both player gold totals** | **Side-by-side, equal weight**, positioned at same visual level — both are active tactical information *(UX: opponent gold must not be subordinated during auction)* |
| Leading bidder indicator | Winning player's class icon pulsing at 1Hz beside price counter |
| Timer countdown | Below card, large bold digits; transitions white → Amber → Crimson-Amber as time compresses. The only timer that communicates urgency, not just duration. |
| Bid input | Bottom-center, angular chip — matches HUD chip language (Section 3.4) |
| Bid history strip | Right of card, narrow column, desaturated — context only, not focal point |

### 7.4 Typography Direction

**Single display sans-serif, two weights.** No serif — Ankama earns personality through outlines and color, not letterform ornamentation.

| Read | Weight | Size | Color |
|---|---|---|---|
| Auction price | Heavy | 3× base | Arcane Gold + Void outline |
| Damage numbers | Heavy | 2.5× base, floating | Crimson Slate |
| Player / opponent gold | Heavy | 2× base | Arcane Gold |
| Timer countdown (auction) | Heavy | 2× base | Temperature-ramped (Section 4.5) |
| Phase label | Regular | 1.25× base | Ivory |
| Card keyword text | Regular | 1× base (floor) | Dark on parchment |

**Rule:** No resource number (gold, HP, ATK, mana) in a weight lighter than Heavy. Resource numbers are action inputs.

*Typeface: TBD — candidate directions include Barlow Condensed Heavy, Rajdhani, or a custom Ankama-adjacent display sans. Requires a type review before production fonts are locked.*

### 7.5 Iconography Style

**Outlined with a flat interior fill.** 2px Void outline. Ties icons to the card art cel-shade style without the detail complexity that breaks down at 24px.

| Category | Treatment | Scale |
|---|---|---|
| Class icons | 3-color flat + Void outline | 32–48px |
| Stat gems (ATK diamond, HP gem) | 2-color gradient fill, no outline — gem shapes, not glyphs | 18–24px |
| Phase icons | Single-color + Void outline, high contrast | 20px |
| Objective status dots | Solid fill, no outline | 12px |
| Reserve mana diamond | Blue gradient + soft Prism White inner glow | 28–36px |

**Rule:** If an icon cannot be identified at 24px in context, it becomes a text label.

### 7.6 Animation Feel

**Fast translate with a single weighted settle.** Not smooth eases (reads laggy), not elastic bounces (reads toylike). Animations confirm state change; they do not entertain.

| Event | Style | Duration |
|---|---|---|
| Card played to board | Fast translate + 1-frame scale pop on land | 80–120ms |
| Auction panel open | Board contracts to zero scale, panel expands from card center | 200ms ease-out |
| Bid increment | Price counter ticks up, gold bloom pulse | 60ms per tick |
| Phase transition label | Fade-in 80ms · hold 600ms · fade-out 80ms | 760ms total |
| Damage number | Appears at full size, translates +40px upward, fades | 500ms |
| Auction timer final 5s | Per second: step toward Crimson-Amber + scale 110%→100% | Per second |

**Anti-patterns:**
- Sliding panels from off-screen (player must track origin — cognitive overhead)
- Looping idle animations on HUD elements (only reserve mana diamond may pulse, at ≤5% opacity delta)
- Scale-bounce on card hover (bounce reserved for confirmation events only)

### 7.7 UX Constraints (from UX Alignment review)

1. **PLACEMENT staged disclosure:** card selection → lane highlight → cell highlight → mana input. Never show the mana split input before a lane is selected.
2. **Mana pools must use distinct container shapes** (not only color) — reserve mana must carry a loop/cycle glyph reinforcing "carries forward." Never display a combined mana total.
3. **Browser focus trap during PLACEMENT:** keyboard focus must not escape the game container during the 10-second timer. Accidental browser-chrome focus is a game-affecting bug.
4. **Timer contrast:** the PLACEMENT countdown must have a semi-opaque background — never rendered directly over animated board cells.
5. **Pointer targets:** bid input and PLACEMENT submit button minimum 44×44 CSS px (browser zoom safety).

---

## 8. Asset Standards

*Merged from Art Direction (AD) and Technical Constraints (TA). One conflict resolved: alpha export — TA decision prevails (straight alpha; Bevy handles conversion).*

### 8.1 Sprite Resolution Tiers

All sprites authored at the largest tier and downsampled — never upscaled.

| Tier | Canvas | Use |
|---|---|---|
| **Board** | 64 × 96 px | Units on board, objectives, trap markers |
| **Card Display** | 120 × 180 px | Cards in hand, shop row, auction panel standard |
| **Card Zoom** | 240 × 360 px | Hand-hover magnification, full card detail |

**Gate:** Every design must pass the visual identity rule (Section 1) at the 64×96 board tier before any higher-tier rendering is produced.

**Card art canvas:** 240×360 px with a **20 px safe zone** inset on all edges for stat badge overlays. The center field is the unconstrained painterly area. Card Display and Board tiers are derived from this canvas via UV crop — not separately scaled.

**Card Zoom sprites** are loaded on-demand and released after the hover ends. They are not atlased. *(TA: 300+ individual card handles cannot be pre-loaded; dynamic load + evict is required.)*

### 8.2 Texture Atlas Organization

Organized by asset category and phase frequency to minimize GPU texture swaps.

| Atlas | Contents | Max Size |
|---|---|---|
| `atlas_units` | All unit board sprites (64×96), all animation frames | 2048×2048 |
| `atlas_cards` | All Card Display sprites (120×180), card frames, stat badges | 2048×2048 |
| `atlas_ui_hud` | HUD chips, icons, objective dots, auction chrome | 1024×1024 |
| `atlas_board` | Board tiles, lane dividers, movement arrows, edge stripes | 1024×1024 |
| `atlas_vfx` | Combat flash frames, bid-pulse ring, damage number glyphs | 1024×1024 |

Units packed into `atlas_units` regardless of class — class identity is carried by sprite color, not atlas separation.

**TA constraint — 2048×2048 maximum:** WebGL2 guarantees `MAX_TEXTURE_SIZE` of 2048×2048 on all spec-compliant implementations. 4096 is not safe on older iOS WebGL2 or low-end mobile hardware.

**TA constraint — power-of-two dimensions:** All atlas sheets must be POT (512, 1024, or 2048). Non-POT textures disable WebGL2 mipmapping.

**TA constraint — 2px padding:** Minimum 1px transparent gutter between sprites; 2px is the safe production value. Prevents GPU texture bleeding at non-integer scale factors.

**TA constraint — even dimensions:** Sprite frame width and height must be even integers. Odd dimensions cause half-pixel sampling artifacts on WASM WebGL2.

### 8.3 Color Profile and Export Format

- **Format:** PNG-32 (RGBA). No WebP for atlased sprites — outline edges corrupt under lossy compression at board scale. *(Both AD and TA agree.)*
- **Alpha:** Export **straight alpha**. Bevy 0.18 handles the conversion internally. *(TA prevails over AD's premultiplied-alpha preference.)*
- **Color profile:** Strip ICC profiles on export. WASM browsers honor sRGB implicitly; embedded profiles add bytes with no visual benefit.
- **Working space:** sRGB, 8-bit per channel.

> ⚠️ **Post-cutoff flag (TA):** Bevy's image feature flags may have changed between 0.14 and 0.18. Verify `ImagePlugin` configuration and available image formats against 0.18 release notes before assuming any non-PNG format loads natively.

### 8.4 Outline Technique

Outlines are **baked into the sprite** at paint time. No procedural outline pass for primary identity outlines.

*Rationale: procedural outlines at 64px produce sub-pixel rounding artifacts that corrupt Ankama line-weight intention.*

| Context | Line Weight | Color |
|---|---|---|
| Board unit (64×96) | 1–2 px | Void `#0D0D14` |
| Card Display (120×180) | 2 px | Void `#0D0D14` |
| Card Zoom (240×360) | 3–4 px outer; 1 px for inner details | Void `#0D0D14` |

A secondary GPU outline pass is permitted for selection-state highlight only (unit selected for placement). The primary identity outline is always baked.

### 8.5 File Naming Convention

Format: `[category]_[name]_[variant]_[size].[ext]`

| Prefix | Applies to |
|---|---|
| `char_` | Unit and character sprites |
| `card_` | Card frame and card art |
| `ui_` | HUD, panel, button, icon elements |
| `env_` | Board tiles, lane elements, environment |
| `vfx_` | Combat flash, bid pulse, transition frames |

Examples: `char_xelor_idle_board.png` · `card_iop_warrior_art_zoom.png` · `ui_btn_bid_default_hud.png` · `env_tile_neutral_board.png`

Variant terms: `idle` `attack` `hit` `death` `default` `hover` `active` `disabled` `loop` — lowercase, no hyphens.

### 8.6 Illustration Pipeline

1. **Sketch** — Structure and silhouette rough. Gate: silhouette reads at 64×96 in grayscale. AD approves before line pass.
2. **Line pass** — Clean Void outlines at Card Zoom canvas (240×360). Bold outer contour first; secondary detail lines at reduced weight.
3. **Flat color** — Fill local colors using Section 4 palette. No gradients yet. Gate: ATK orange and HP teal absent except on stat badges; no faction palette conflicts.
4. **Rendering pass** — Cel-shade: one highlight stop up, one shadow stop down from flat. No airbrush. Highlights lean warm; shadows lean cool.
5. **Outline QA** — Verify outline reads as Void at all three tiers. Downscale to 64×96 and confirm Section 1 silhouette rule passes.
6. **Export** — Flatten to PNG-32 straight alpha. Name per 8.5. Deliver to Technical Artist for atlas packing.

### 8.7 Memory Budget

Total WASM heap: 256 MB. Art allocation target: **~96 MB**.

| Category | Max Sheets | Approx. Budget |
|---|---|---|
| Unit sprites | 2 × 2048×2048 | ~32 MB |
| Card art | 2 × 2048×2048 | ~32 MB |
| UI elements | 1 × 2048×2048 | ~16 MB |
| Environment / board | 1 × 2048×2048 | ~16 MB |
| Engine + code heap (reserved) | — | ~160 MB |

**TA flag:** Environment framing (stone arches, ivy, torches from Section 6.2) must be **tiled and atlased** — a single full-resolution painterly scene would consume the entire environment budget in one asset.

### 8.8 Prohibited Asset Patterns

| Pattern | Why Prohibited |
|---|---|
| One texture file per unit | Destroys sprite batching — N units = N draw calls, blows the 12ms render budget |
| Non-POT atlas dimensions | Disables WebGL2 mipmapping; increases GPU memory |
| Atlas larger than 2048×2048 | Not safe on all WebGL2 targets (iOS WebGL2, low-end mobile) |
| Individual full-resolution card art PNGs at load time | 300+ card handles cannot be pre-loaded; use atlased Card Display + on-demand Card Zoom |
| Sprites with odd-number pixel dimensions | Half-pixel UV offsets cause shimmer at non-integer zoom |
| Embedded ICC color profiles in PNG | Ignored by Bevy's decoder; adds file size for zero visual benefit |
| WebP or KTX2 without confirmed Bevy 0.18 feature flags | Do not deliver non-PNG formats until `ImagePlugin` feature configuration is verified |

---

## 9. Style Prohibitions

### 9.1 Style Prohibitions

**No photorealism or semi-photorealism.**
Subsurface scattering, specular normal maps, ambient occlusion bakes, and film-grain post-processing flatten the read of bold outlines at board scale. At 48–64px unit size, photorealistic texture becomes mud.

**No grimdark desaturation or atmospheric gray washes.**
The thermal arc (Section 2) deliberately modulates saturation to signal game state. A globally desaturated base palette collapses that communication system. "Gritty" is not an Ankama register.

**No generic Western fantasy rendering — Hearthstone warm-glow or Magic chiaroscuro.**
Both rely on heavy ambient occlusion shadows and oil-painting textures that erase the Ankama cel-shade identity. If a card could appear in Hearthstone's library without looking out of place, it is wrong for this game.

**No flat vector minimalism (mobile-idle or hyper-casual aesthetic).**
Krosmaga uses confident outlines with interior detail — not filled shapes with no stroke variation. Flat vector reads as "free browser game," undermining the auction's premium feel.

### 9.2 Color Prohibitions

**Never use green to indicate "real" vs. "fake" objective.**
Green is owned by Cra class. Using it to distinguish real from fake would leak hidden information through color before the reveal — breaking the core deception mechanic. This is the game's most dangerous color-confusion vector.

**Never use red to indicate fake-objective team or "losing" state.**
Red is owned by combat events and Sacrier class. A red-tinted objective pedestal reads as "damaged" or "Sacrier-affiliated," not "deceptive."

**Never reuse ATK orange (`#E07020`) or HP teal (`#2AA8C4`) for any non-stat purpose.**
These are globally reserved constants (Section 4.1). Using either on a button, hover state, or particle creates ambiguity — the player will look for an ATK or HP value that does not exist.

**Never reach full `#FF0000` outside Sacrier combat events and damage numbers.**
The auction escalation track stops short of pure red intentionally. Full red is an alarm register; using it in environmental art or UI chrome desensitises its damage signal.

**Never use Cra forest green (`#2A6B3C`) to communicate "safe," "confirmed," or "go."**
Green-as-safe is a global UI convention players carry from every other game. It must never appear in a safety-signalling role in Lanes and Lies — the color is locked to Cra class identity.

### 9.3 Structural Prohibitions

**Never occlude the gold counter.**
The player's current gold balance affects every decision in every phase. Any panel, animation, or overlay that covers it — even briefly — is prohibited.

**Never give the opponent's board side visual texture or information during PLACEMENT.**
The opponent's side must be a solid dark mass during placement. Any unit silhouette, shimmer, or particle that leaks through the fog violates the hidden-placement mechanic. The "committed dread" mood (Section 2) depends on that side being genuinely opaque.

**Never let two elements compete for the brightest point in any scene.**
The Structured Luminance Hierarchy (Section 1) requires a single visual apex. When two elements share luminance — auction card and board background glow, for example — the element requiring player decision wins. Background steps down.

**Never use directional arrows for anything other than lane-push direction.**
The red movement arrows in board cells (Section 3.3) are the primary language for "units advance this way." Repurposing arrow iconography for tooltips, UI navigation, or other game actions creates false reads.

**Never animate objective status dots differently between real and fake slots.**
Any pulse, scale, or animation difference between dots before an objective is destroyed leaks fake/real identity. Dots must be visually identical in behaviour until the `ObjectiveDestroyed` broadcast.

### 9.4 IP / Reference Prohibitions

**Never use Krosmaga card art verbatim.**
The 298 Extension=1 cards are the mechanical and thematic source pool — not an asset library. Card art must be original work inspired by the Ankama aesthetic. Reproducing Ankama's illustrations is copyright infringement.

**Never reproduce Krosmaga's specific stat-badge vectors or frame chrome at 1:1.**
The card frame anatomy (Section 3.2) was derived from observing Krosmaga's design language. That observation may drive proportions and color logic — it may not drive copy-paste of the actual badge geometry or border artwork.

**Never copy Hearthstone's golden card or legendary treatment.**
Foil swirls, animated card portraits, and particle-loop legendary frames are Blizzard's visual signature. The auction mechanic is Lanes and Lies's premium differentiator — it must not signal "special card" through someone else's vocabulary.

**Any Wakfu/Dofus character used as reference must be substantially transformed.**
Concept art from Ankama's public releases may inform proportions, silhouette vocabulary, and color range. A character design may not trace or closely derive from a specific published Ankama piece — it must be a new design belonging to the same aesthetic lineage, not a reproduction of it.
