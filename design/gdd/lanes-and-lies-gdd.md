# Lanes and Lies — Game Design Document

**Project:** Hackathon (1 week, solo dev + AI)
**Stack:** Bevy 0.18 (Rust) — WASM client (Trunk → Vercel) + headless server (Cargo → Railway)
**Target audience:** Hardcore gamers (FPS, LoL/Dota, WC3, Hearthstone, Slay the Spire, Krosmaga)
**Session length:** 1–2h total, 10–15 min per game
**Theme:** Fantasy (Ankama/Dofus/Wakfu aesthetic — see `design/art-references/`)

---

## 1. Overview

Lanes and Lies is a real-time multiplayer card/lane-pushing game where players build decks through competitive auctions and simultaneous hidden-placement combat. Each player controls a team of units across 5 parallel lanes, pushing toward the opponent's hidden objectives. The core mechanic is an open ascending auction for powerful cards: players watch the price rise in real time and must decide when to stop bidding, creating genuine bluff and economic pressure. Combat resolves simultaneously — every player places cards in secret, then all lanes resolve at once. Victory requires destroying 2 of the opponent's 3 real objectives while protecting your own, with 2 decoy fake objectives providing misdirection and bonus rewards for whoever attacks them.

---

## 2. Player Fantasy

The player should feel like a **cunning tactician in an information war** — never certain what the opponent knows, always forcing imperfect reads, and watching their economic advantage compound through smart auctions and precise combat.

**The four feelings that define success:**

1. **"I read them."** — Bidding exactly high enough to win an auction the opponent wanted, or stopping just before being trapped into an unwanted purchase. The auction should feel like a negotiation where knowing when to stop is the skill.
2. **"I fooled them."** — The opponent wasted two rounds attacking a fake objective and got stronger for it, but you know exactly where your real objectives are and can now end the game.
3. **"My deck came online."** — The TFT weighting handed you a third Gobball that triggered your LEADER synergy and turned a losing lane into a wall. Archetype building should feel satisfying without being mandatory.
4. **"Zero idle time."** — Even when the opponent is resolving their turn, you are watching the board, watching their gold, watching the price rise on the auction card. Spectating should feel like active intelligence gathering.

**How the three system fantasies layer into this single identity:** The **auction** is the singular tense moment — a 20-second negotiation that punctuates every third round and crystallises the "I read them" feeling into a concrete gold stake. The **class system** is the silent rhythm underlying every other round — your Xelor reserve growing, your Sadida seeds spreading, your Sacrier threatening sacrifice — an authorship layer that makes "my deck came online" feel personal and repeatable. The **prism system** is the standing income that funds both: a persistent spatial commitment (hold lane 2+4 for reserve, hold lane 1+5 for strike power) that makes "I fooled them" extend beyond fake objectives to the entire economic map. All three sub-fantasies serve the same top-level identity: a cunning tactician who out-reads their opponent across multiple timescales simultaneously.

## Game Pillars

| Pillar | Definition | Design Test |
|---|---|---|
| Simple surface | Every rule has at most one exception. A new player can place units on round 1 without reading a manual. | When two rule formulations achieve the same outcome, choose the one that requires fewer words to explain to a first-time player. |
| Deep emergence | Classes, families, card synergies, and objective positioning create 10+ viable strategies without being explicitly taught. | When adding a mechanic, ask: does this increase strategic depth without increasing the rules surface? If not, cut it. |
| No idle spectating | Players always have an active decision available or are watching live information that affects their next decision. Every phase gives you something meaningful to do or observe. | When a player has no cards to play, they should still be reading the board, watching opponent gold, tracking prisms, or timing the auction. If they have nothing to do at all, add a decision. |
| Auction as signature | The open ascending auction — watching the price climb in real time and deciding when to drop — is the mechanic no other game has. It is the commercial identity of this game. | When choosing between two draft system designs, prefer the one that puts more tension on the auction moment. The shop is scaffolding; the auction is the feature. |

> **Canonical pillar names (4):** Simple surface · Deep emergence · No idle spectating · Auction as signature.
> "Krosmaga foundation" is a sourcing constraint, not a design pillar — it does not gate design decisions.

## Anti-Pillars

What this game is NOT. These statements prevent scope creep and clarify the design boundary.

| Anti-Pillar | Why it is excluded |
|---|---|
| Not a chess-like pure strategy game | Perfect information removes the bluff and economic pressure that define the core fantasy. Hidden objectives and real-time auction dynamics are load-bearing — remove them and the game becomes a different genre. |
| Not a collectible deck-builder | Players build their hand from a shared pool each match, not from a personal owned collection between sessions. There is no deckbuilding meta outside of the game session. Progression is within-match and within-round, not across sessions. |
| Not a real-time twitch game | Placement is simultaneous and timed but deliberate (10-second window). Reaction speed is not a skill axis — reading information, bidding strategy, and resource management are. The game punishes recklessness, not slowness. |
| Not a passive spectator experience | No phase exists where a player has nothing to do or observe. Idle watching is an explicit failure state for the design, not a feature. Any mechanic that produces forced downtime violates the "No idle spectating" pillar and must be redesigned. |

**Art direction:** Ankama studio style — vibrant cel-shaded 2D illustration, bold clean outlines, rich saturated colors, Wakfu animated series aesthetic, French fantasy cartoon. Reference images in `design/art-references/`: `slide1_title_board_epic.png`, `slide3_board_gameplay.png`, `slide4_objectives_real_fake.png`, `slide6_auction_bidding.png`, `slide7_classes_lineup.png`. **Visual authority:** The reference images are the authoritative visual target — they were generated from this text description and are consistent with it. For `/art-bible` authoring, treat the images as the primary source and this text as commentary.

---

## 3. Detailed Rules

### 3.1 Game Modes

All modes share the same rules. Team size and lane configuration are parameters.

| Mode | Players | Board |
|---|---|---|
| 1v1 | 2 | 5 horizontal lanes, Team A (bottom) vs Team B (top) |
| 2v2 | 4 | 5 horizontal lanes, Team A vs Team B; max 2 units per lane per team side |
| 3v3 | 6 | 5 horizontal lanes, Team A vs Team B |
| 1v1v1 | 3 | 5 horizontal lanes between each pair of teams (advanced mode — 3-player auction dynamics apply) |
| 2v2v2 | 6 | 5 lanes: 2 toward Team A, 2 toward Team B, 1 central lane with a capture point |

**Gold is individual in all modes.** Kill rewards go to the player whose unit dealt the killing blow. Objective destruction rewards go to the player whose card or unit dealt the final damage.

**2v2v2 central capture point:** The team that has a living unit on the central cell at end of RESOLUTION receives a passive bonus (amount TBD during development). All teams contest it. Central lane uses standard objective rules.

### 3.2 Board Structure

- 5 lanes per game (horizontal)
- Each lane: 8 cells total (4 on your side, 4 on opponent's side)
- Cell 1 = your spawn, Cell 8 = opponent's objective cell
- Units spawn on Cell 1 of their controller's side
- Units advance automatically toward Cell 8 (opponent side) by their MP value each round
- Maximum **1 Minion per player per lane** at any time
- In 2v2: maximum **2 Minions per team per lane** (1 per player on the team)
- Traps, Structures, and Field cards do not count against the Minion slot limit

**Spawn range — where Minions can be placed:**

Minions can only be placed within your current spawn range. Spawn range expands when you (the attacker) destroy your opponent's fake objectives:

| Fakes you have destroyed | Your spawn range (all lanes) |
|---|---|
| 0 | Row 1 only (Cell 1 of your 4 cells) |
| 1 | Row 1 or Row 2 (Cell 1 or 2) |
| 2 | Row 1, 2, or 3 (Cell 1, 2, or 3) |

- Spawn expansion is **global** — it applies to all your lanes simultaneously, not just the lane where the fake was destroyed
- You CHOOSE which row to spawn in (within your available range); spawning deeper has strategic tradeoffs
- **Structures** are exempt: they can be placed on any of your 20 cells regardless of spawn range

### 3.3 Objectives (Hidden)

Each player (or team) has **5 objectives** at the far end of their 5 lanes.

- **3 real objectives** and **2 fake objectives** — visually identical to the opponent
- Each objective: **5 HP, 0 AR, cannot be healed**
- Assignment is random at game start; only the owner knows which is real and which is fake
- The opponent must discover by attacking them

**Win condition:** You **LOSE** when **2 of your own REAL objectives** are destroyed. Fake objectives destroyed do NOT count toward this threshold.

**Destruction rewards — any objective (real or fake):**
- Attacker receives **+3 gold**

**Destruction rewards — fake objectives only (additional):**
- Attacker's units in that lane permanently spawn 1 cell further into the board (expanded spawn range)
- Attacker receives a random bonus: **+1 permanent mana cap** OR **1 free card pick** (solo, outside auction)
  - "+1 permanent mana cap" = attacker's `mana_cap` increases from 10 to 11 permanently (or from 11 to 12 if they already claimed one fake)
  - "1 free card pick" = attacker draws 1 additional card outside the auction system at the next draft phase

**Cards that reveal objective information ("double-tranchant" cards):**
Some powerful cards have self-detrimental effects including:
- Reveals one of YOUR objectives to the opponent (you choose which one)
- Destroys one of YOUR OWN fake objectives
- Opponent learns whether a specific one of your objectives is real or fake

### 3.4 Economy — Dual Currency

**Two independent resources: Mana (combat) and Gold (draft).**

#### Mana — Playing Cards in Combat

Every player has **two mana pools**:

**Current-round mana** — resets to 0 at the start of each DRAFT phase (all classes):
- Gained: `mana = min(round_number, mana_cap)` at the start of each round
- Default `mana_cap = 10` (reached at round 10)
- Used to play: Units, Spells, Traps, Structures, Passives/Auras, Orders
- Mana cap can be permanently increased by +1 from fake objective destruction reward (max cap = 12 if both fakes claimed)

**Reserve mana** — persists between rounds (all classes):
- Gained from: playing the "+1 reserve" spell card received from collecting Prism Lane 2/4 (see Section 3.4 Prisms), and from Xelor-specific spells (see Section 3.10)
- Can be spent at any time to pay card costs, from either pool (player chooses)
- Reserve is independent of the mana cap — it does not reset each round

Xelor's uniqueness is NOT that he is the only class with a reserve — all classes have one. Xelor's uniqueness is that his class spells (Gelure, Rollback, Miss Nuit, Garde-Temps) are designed entirely around growing and spending the reserve aggressively.

#### Gold — Draft Economy

| Source | Amount | Timing |
|---|---|---|
| Baseline income | +2 | Start of each DRAFT phase |
| Interest (see formula) | +0, +1, or +2 | Start of DRAFT, before spending |
| Kill reward | +1 per kill | Immediately when unit dies |
| Objective destroyed | +3 | Immediately when objective HP reaches 0 |

- **Starting gold:** 5 (one-time only, at draft initial)
- **Shop costs:** Common=1g, Uncommon=2g, Rare=3g, Epic=4g, Manual refresh=1g
- **Auction starting bid:** Rare=3g, Epic=4g, Legendary=5g (Common and Uncommon never appear at auction)
- **Legendary (= Krosmaga "Infinite"):** auction-only, neutral only, 1 copy per player pool
- **Hand limit:** 10 cards maximum. Purchase is rejected (gold not deducted) if hand is at 10 when the server processes the transaction.

#### Prisms

2 prisms per lane — 1 per player, located at each player's own spawn cell (absolute cell 1 for Player A, absolute cell 8 for Player B). Total: 10 prisms per 1v1 game.

A prism is collected when that player's own unit ends the standard movement sub-step at the player's spawn cell. Each player's 5 prisms respawn independently when that player has collected all 5 of their own prisms.

| Lane | Reward |
|---|---|
| Lane 1 (edge) | Spell card added to hand: "1 damage to a chosen objective" (costs 3 mana to play) |
| Lane 2 | Spell card added to hand: "Add +1 mana to your reserve" (player decides when to play it) |
| Lane 3 (center) | Draw 1 card (random, from the shop pool) |
| Lane 4 | Spell card added to hand: "Add +1 mana to your reserve" (player decides when to play it) |
| Lane 5 (edge) | Spell card added to hand: "1 damage to a chosen objective" (costs 3 mana to play) |

**Prisms do not grant gold.** Prisms grant mana/cards/spells only.

**Team modes:** Each player tracks their own 5 prisms independently. In 2v2, each player on a team has their own prisms at their own spawn cell; teammates do not share prism collection.

### 3.5 Draft — Card Acquisition

#### Draft Initial (Pre-Game, Once Only)

- 9 random cards displayed (mix of player's chosen class + neutral)
- Budget: **5 gold** (use it or lose it)
- Purchase up to budget; unpurchased cards disappear

#### Personal Shop (Each Round)

- 3 cards shown per round
- Each slot: **50% chance class card**, **50% chance neutral card**
- TFT weighting: the more cards of a type/family you own, the more likely that type appears in the relevant slot
- Auto-refreshes at the start of each DRAFT phase (previous unsold cards disappear)
- Manual refresh: **1 gold**, usable any time during DRAFT phase before PLACEMENT begins
- Refresh does not reset during the same DRAFT phase; buying a card does not trigger a refresh

#### Auctions (Shared, Scheduled Rounds)

Auctions occur at rounds: **1-3, 1-6, 2-3, 2-6, 3-3, 3-6...** (notation = Stage-Round within stage; each stage = 6 rounds; pattern continues indefinitely while the game lasts).

- Cards offered: Rare, Epic, and Limited only (more powerful than personal shop)
- Cards are neutral only (no class-specific cards at auction)
- **Starting price**: Rare=3g, Epic=4g, Legendary=5g
- **Bid increment** = +1 gold minimum per bid
- **Timer** = 20 seconds; resets to (current_timer + 5 seconds, capped at 20s) on each accepted bid
- **Winner** = player whose bid is accepted by the server when the timer reaches 0; pays their bid
- **Current price and current leader** are visible to all players during the auction
- **Tie-break** (two bids received at the same server tick at the same price): first received by the server wins at that price
- **No bids placed**: card is removed from the auction pool for this game
- **Hand full**: if auction winner has 10 cards when the auction resolves, the won card is discarded and gold is paid (you chose to bid; the purchase is binding)

**Bluff mechanic:** Because the current price and leader are visible, players can bid strategically to drive up the price for an opponent they know wants the card, then stop bidding before becoming the last bidder. The risk: if you stop and no one else bids, the last bidder (your opponent) wins at whatever price they stopped at. If you bid too aggressively and your opponent drops, you are the last bidder and must pay. Reading opponent gold totals and desire informs when a bluff bid is worth the risk.

#### Hand Size

Maximum 10 cards in hand at all times. Purchase attempts when hand is full are rejected server-side; gold is not deducted.

### 3.6 Combat

#### Round Flow

1. **DRAFT** — Personal shop purchases + auction (if auction round)
2. **PLACEMENT** — **10 seconds**, simultaneous secret selection of cards to play this round
3. **RESOLUTION** — Simultaneous reveal; all effects resolve; units advance and fight

#### Placement Rules

- Each player secretly selects cards to play this round during the 10-second timer
- Cards are not revealed until RESOLUTION begins
- Timer is a single shared countdown for all players; the server begins RESOLUTION when all players have submitted OR when the timer expires
- **Timeout fallback:** If a player's placement is not received when the timer expires, they are treated as having played zero cards that round (their existing board state is unchanged)
- Players may place at most 1 unit per lane (their personal slot limit)

#### Resolution Order

All 6 sub-steps are **global passes** — each step executes across all lanes simultaneously before the next step begins. Lane order (left to right, Lane 1–5) is tie-breaking within a sub-step only (e.g., two units claiming the same cell both resolve in lane order).

1. **Apply placement effects (global):** All played cards take effect simultaneously across all lanes. APPEARANCE triggers fire immediately when played units enter the board. If an APPEARANCE trigger kills a unit, that unit's DEATH trigger fires after all APPEARANCE effects resolve (sequential chain — see Section 5).
2. **CHARGE X bonus movements (global):** Units with the movement keyword CHARGE X advance their bonus X cells across all lanes simultaneously. (This sub-step only affects units with a numeric CHARGE X value — distinct from the combat keyword CHARGE "can act this round.")
3. **FIRST STRIKE attacks (global):** All FIRST STRIKE units deal damage simultaneously across all lanes.
4. **Remove dead units (global):** Units reduced to 0 HP across all lanes are removed; DEATH triggers fire.
5. **Standard movement (global):** All remaining units advance by their MP value across all lanes simultaneously.
6. **Standard combat (global):** Units at the same cell fight; units at the opponent's far edge deal damage to the objective.

Cross-lane triggers (CHANGE LANE, Strich auto-switch) apply after the sub-step that caused them, before the next sub-step begins.

#### Simultaneous Effects

All effects triggered in the same resolution sub-step resolve **simultaneously with no priority** — both effects apply. Exception: DEATH trigger chains are sequential (a DEATH trigger from unit A fires; if that trigger kills unit B, unit B's DEATH trigger fires after, not simultaneously).

#### Unit Combat

- Units occupying the same cell fight each other
- `net_damage = max(0, ATK_attacker − AR_defender)`
- Damage cannot go negative (does not heal)
- FIRST STRIKE and FIRST STRIKE: both units attack simultaneously (mutual FIRST STRIKE = simultaneous damage, no advantage)
- Dead units are removed before standard movement

#### Objective Damage

- At the end of sub-step 6, any attacker unit occupying **Cell 8** (the opponent's far-edge cell) deals its ATK value as damage to the objective in that lane
- The unit **remains** at Cell 8 and attacks the objective again the following round unless killed
- `objective.hp -= attacker_unit.ATK`
- "Dofus" and "objectif" are synonymous terms throughout all card text

#### RPS Type System

Each unit card has one of four types: **Blade, Arcane, Shield, Neutral**

Triangle: **Blade > Arcane > Shield > Blade** (Neutral has no advantage or disadvantage)

When the attacker's type beats the defender's type:
- `ATK_attacker += 1` for this combat
- `AR_attacker += 1` for this combat (absorbs 1 more incoming damage)

Type assignment status: most cards are currently **Neutral** (unassigned). When a card is explicitly assigned a type (Blade/Arcane/Shield), the RPS rules apply. Cards will be assigned types progressively; unassigned = Neutral until updated.

ARMOR-PIERCING interacts with RPS: ARMOR-PIERCING treats the defender's AR as 0, including any bonus AR gained from RPS type advantage.

### 3.7 Keywords

#### Timing Triggers

| Keyword | When it fires |
|---|---|
| APPEARANCE | Unit enters play |
| DEATH | Unit is destroyed (HP = 0) |
| FINAL BLOW | Unit kills another unit |
| COUNTERATTACK | Unit receives damage in combat |
| INJURED | Unit has missing HP (persistent state, re-checked each sub-step) |
| START OF TURN | Beginning of each round's DRAFT phase |
| END OF TURN | After RESOLUTION completes |

#### Combat Keywords

| Keyword | Effect |
|---|---|
| FIRST STRIKE | Attacks before standard combat; can kill before retaliation |
| CHARGE | Can act the round it is played (no summoning sickness) |
| RANGE 1-X | Attacks at distance without advancing; X = maximum range in cells |
| WALL | Stationary; MP = 0 |
| BODYGUARD | On entry: controller selects one friendly unit currently on the board. That chosen unit cannot be targeted by opponent spells or Order cards while this BODYGUARD unit is alive. RANGE attacks bypass BODYGUARD (see Section 5). Units reaching Cell 8 via movement still damage objectives normally — BODYGUARD only blocks spell/Order targeting. |
| IRREMOVABLE | Cannot be displaced, returned to hand, or repositioned |
| UNTARGETABLE | Cannot be targeted by spells or order cards |
| RESISTANCE X | Takes X fewer damage per hit (minimum 0) |
| VULNERABILITY X | Takes X additional damage per hit |
| SILENCE | Loses all keywords and effects |
| STUN | Cannot act this round |
| ARMOR-PIERCING | Treats defender's AR as 0 for the attacker's outgoing damage only. The attacker's own AR (including any RPS type-advantage bonus) is unaffected. |
| SHIELD | Absorbs the next attack (negates damage once) |
| LEADER | Boosts units of the same family (see individual card for bonus) |
| OUTNUMBERED | Effect triggers if this unit's controller has fewer units on the board than the opponent |
| NECRO | Extension=518 keyword — **not in pool, removed** |

#### Movement Keywords

| Keyword | Effect |
|---|---|
| CHARGE X | Advances X additional cells this round |
| REPEL X | Pushes target X cells toward their own side |
| ATTRACT X | Pulls target X cells toward the caster |
| TELEPORT | Repositions a unit to a specified cell |
| CHANGE LANE | Moves to an adjacent lane; Strich triggers this automatically when an enemy unit enters play in its current lane |

### 3.8 Card Types

| Type | Description |
|---|---|
| Minion | Has ATK/HP/MP/AR + keywords; spawns within your current spawn range, advances automatically by MP each round |
| Spell | Instant effect; costs mana; consumed on use |
| Trap | Placed face-down on any of your 20 cells (not spawn-restricted); triggers when an enemy unit enters that cell; fake traps (1 mana, no effect) are pure bluff |
| Structure | 0 ATK, has HP; placeable on **any** of your 20 cells regardless of spawn range; provides continuous effects each round until destroyed |
| Field | Played once; permanent lane-wide effect; visible to opponent; costs mana; maximum 1 Field per lane per player. Original design — not from Krosmaga Extension=1. |

**Card type sourcing:** Minion and Spell cards come from Krosmaga Extension=1 (~315 cards total). Trap, Structure, and Field cards are original designs for this game — they do not exist in Extension=1 and must be authored separately.

Draw effects ("pioche 1 carte aléatoire") draw from the shop pool randomly — there is no fixed deck.

### 3.9 Classes (6 Classes, Extension=1 Only)

All classes are public information — your opponent can see which class you chose. Class cards appear in your personal shop only (never at auction). All non-class cards (neutrals) can appear at auction.

#### IOP — 24 cards | Aggro Rush

**Identity:** Fastest units, CHARGE native, buff synergies.

**Key signature cards (Krosmics):**
- *Authority* — targeted unit charges to the opponent's objective cell
- *Felida* — FINAL BLOW: adds Jump to hand
- *Heure de Gloire* — spends all remaining mana; +1 ATK+1 AR per mana spent
- *Appel à la Baston* — draws 4 cards; keeps units, discards spells

**Key units:** Chuck Maurice (FIRST STRIKE 1/1), Archille (FIRST STRIKE 5/4), Sono Sino (other friendly units charge 1 at start of turn)

**Key spells:** Charge (targeted unit charges 2), Intimidation (all friendly units charge 2), Ravage (+2 ATK + FIRST STRIKE), Compulsion (+2 ATK +2 AR)

#### CRA — 25 cards | Control Range

**Identity:** RANGE attacks, direct damage, repel/push.

**Key signature cards (Krosmics):**
- *Criblage* — deals damage to a dofus equal to the number of Cra units in play
- *Harcèlement* — 1 damage to all enemy dofus
- *Flèche Destructrice* — destroys a unit
- *Guy Yomtella* — RANGE 1-6

**Key units:** Clara Byne (RANGE 1-2, 1 damage on APPEARANCE), Fantôme Crâ (RANGE 1-2 + UNTARGETABLE), Betty Boubz (RANGE 1-3, COUNTERATTACK: repels attacker 2)

**Note:** "Each Cra in play" = each Cra UNIT on the board. Scales with your board presence.

#### SACRIER — 25 cards | Masochist/Sacrifice

**Identity:** INJURED bonuses (stronger when damaged), BODYGUARD, self-sacrifice.

**Key signature cards (Krosmics):**
- *Fulgurance* — swaps the positions of 2 friendly units
- *Sang Méprise* — reveals all dofus (real and fake) to both players this round
- *Punition* — sacrifice one of your dofus to deal 3 damage to each alive enemy dofus; the sacrificed dofus loses 5 HP (destroyed if real). **Self-inflicted dofus destruction counts toward your own loss condition.**
- *Jet le Pied Volant* — COUNTERATTACK: charges 1 cell when any of your units or dofus takes damage

**Key units:** Edass (INJURED: +2 ATK + FIRST STRIKE), Dureden Taillair (INJURED: +2 ATK + 1 MP), Bould Erdash (BODYGUARD 7/6)

**Note (team modes):** Punition sacrifices one of your team's dofus, not a teammate's.

#### XELOR — 25 cards | Tempo/Reserve

**Identity:** Reserve mana specialist — the class that most aggressively exploits the universal reserve mechanic via spells and cross-round accumulation.

**Context — Reserve is universal:** All players have a reserve (see Section 3.4). Lane 2/4 prisms add +1 to reserve for any class. Xelor's uniqueness is not the reserve itself — it is the class spells that fill the reserve far beyond what prisms allow, and the Krosmic cards that spend massive reserve amounts.

**Unique class mechanics:**
- Reserve has no maximum cap for any class — organic board pressure is the intended limiter
- Standard mana still accrues each round normally; Xelor chooses what to spend from reserve vs. current mana
- **Spending rule:** All Xelor cards cost from **current-round mana** unless the card text explicitly says "from reserve" or "dépense la réserve." Garde-Temps specifically costs 20 reserve mana.

**Xelor-specific reserve sources** (beyond the universal Lane 2/4 prisms):
- *Gelure* — transfers all current mana to reserve (spell; converts current mana that would otherwise reset to 0)
- *Miss Nuit* (Krosmic) — +1 to reserve each time the opponent plays a card
- Various other Xelor spells that interact with the reserve pool

**Key signature cards (Krosmics):**
- *Rollback* — spends entire reserve; friendly units charge by that many cells
- *Garde-Temps* (20 reserve mana!) — destroys a dofus
- *Dévouement* — transforms a unit into a friendly Momie
- *Miss Nuit* — +1 reserve each time opponent plays a card

**Key units:** Instantina (2 or 5 damage if 5+ reserve), Dente le Remonteur (free to play if 5+ reserve), Radoris Montrouge (spends reserve for ATK+AR)

**Key spells:** Sinistro (attaches to dofus, 1 damage/turn, destroyed if dofus takes damage), Gelure (transfers current mana to reserve), Sablier (steals 1 mana from opponent's current mana), Xelorium (steals ALL opponent's current mana for this round and adds to reserve)

**Note:** Xelorium steals all opponent **current mana** only. The opponent's reserve is untouched — they can still play cards by spending from their reserve. Xelorium is a powerful tempo play but not a complete lockout if the opponent has accumulated reserve mana.

#### ECAFLIP — 24 cards | Luck/Chaos

**Identity:** 1d6 rolls, coin flips, maximum variance.

**Key signature cards (Krosmics):**
- *Chacha/Bow Meow* — transforms a unit into a friendly Chacha Noir
- *Craps* — deals 8 OR 4 damage distributed among enemy dofus (coin flip)
- *Miranda* — APPEARANCE: takes control of adjacent enemy units until she dies
- *Defhi Croquets* — FIRST STRIKE + charges 1d6 cells

**Key units:** Chatar (APPEARANCE: +2 ATK OR 2 self-damage, coin flip), Shava Shavien (DEATH: returns to your hand OR to opponent's hand, coin flip), Karla Blondie (gains 1d6 ATK)

**Key spells:** Bluff (swaps positions of 2 of your dofus), Dé du Chateux (1d6 damage; reveals dofus if 3 or less), De Ecaflip (1d6; returns card to your hand if 3 or less), Bond du Félin (teleports 1d6 cells)

**All 1d6 rolls are server-seeded and broadcast.** Client-side RNG is not used.

#### SADIDA — 25 cards | Setup/Nature

**Identity:** Seeds placed on board → triggered by unit movement; two-phase play (setup then convert).

**Seed placement:** A Seed is placed on a specific cell of your 4×5 grid. Effects: +1 AR if a friendly unit walks over it; 1 damage if an enemy unit walks over it.

**Key signature cards (Krosmics):**
- *Pollinisation* — 3 damage to enemy units + places a Seed on each cell where a unit died
- *Savoir Sadida* — transforms a unit into a friendly Graine
- *Sacrifice Poupesque* — sacrifices your dolls; each deals 1 damage to the enemy dofus in its lane
- *Sylvine Folherbe* — APPEARANCE: adds Madoll + Sac de Graines + Buisson to hand

**Tokens generated by Sadida cards:**
- *Graine/Seed* — +1 AR to friendly units / 1 damage to enemy units that walk on it
- *La Gonflable* — heals your other units in its lane for 2 HP after all movement
- *La Folle/Madoll* — reduces your spell costs by 1 mana while in play
- *La Sacrifiée* — DEATH: 1 damage to enemy units in its lane

**Key spells:** Sac de Graines (add 2 Seeds to field), Ronce (2 damage + places a Seed on death), Graines de Folie (converts all Seeds to Madolls), Tremblement de Terre (4 damage + adjacent)

### 3.10 Card Pool

**Source:** Krosmaga Extension=1 (Extension=518 excluded).

| Category | Count |
|---|---|
| Iop | 24 class cards |
| Cra | 25 class cards |
| Sacrier | 25 class cards |
| Xelor | 25 class cards |
| Ecaflip | 24 class cards |
| Sadida | 25 class cards |
| **Total class** | **148** |
| Neutrals | ~150 cards |
| **Total pool** | **~298 cards** |

**Excluded classes:** Eniripsa, Sram, Enutrof, Feca, Huppermage (~114 cards) — not in pool.

**Rarities and shop costs:**

| Rarity | Copies in pool | Shop cost | Notes |
|---|---|---|---|
| Common | Multiple | 1g | Class and neutral cards |
| Uncommon | Multiple (fewer) | 2g | Class and neutral cards |
| Rare | 4 (per player pool) | 3g | Class and neutral cards |
| Epic (= Krosmaga "Krosmic") | 1 | 4g in shop | Class-specific signature cards; powerful class identity cards |
| Legendary (= Krosmaga "Infinite") | 1 | 5g starting bid | **Auction only** — neutral only; 3-level evolution deferred for hackathon |

**Important:** Epic cards appear in the personal shop (class-specific cards only). Legendary cards appear **only at auction** and are neutral. Common and Uncommon cards never appear at auction.

**Neutral families included (Extension=1):** Arachnée, Bow Meow, Chafer, Craqueleur, Gobball, Blibli, Jelly, Drheller, Tofu, Wabbit/Cawwot, Strich, Rat, Larva, Scaraleaf, Piwi, Vigilante, Vampire, Moogrr, Boowolf, Monk, Plant, Snapper, Bandit, Midgins, Krosmics divers.

**Notable family archetypes:**

| Family | Archetype | Key mechanic |
|---|---|---|
| Chafer | Resurrection | DEATH: spawns Decrepit Chafer if another Chafer is in play |
| Craqueleur | Armor | RESISTANCE 1 native; stacks with LEADER bonus |
| Gobball | Family buff | LEADER +ATK/AR; APPEARANCE synergies |
| Tofu | Rush | CHARGE; scales with number of Tofus in play |
| Boowolf | Evolution | FINAL BLOW: transforms into next evolution form |
| Strich | Lane chaos | Automatically changes lane when an enemy unit enters play in its lane |
| Arachnée | Return to hand | Units return to hand under specific conditions |
| Bow Meow | Swarm | Fast tokens MP=6; mass generation |
| Jelly | Spawn synergy | +ATK+AR when another Jelly enters play |
| Scaraleaf | Armor stack | +AR accumulated each round |
| Drheller | Draw engine | DEATH: draw a card |

---

## 4. Formulas

All formulas use these conventions: `R` = current round number; `cap` = mana_cap (default 10); `g` = gold held.

### 4.1 Mana Ramp

**Current-round mana (all classes — Hearthstone model):**
```
current_mana(R) = min(R, mana_cap)
default mana_cap = 10
```
At the start of each round: mana bar fills to `min(R, 10)`.
Unspent mana is discarded at end of round — it does NOT carry over (all classes, including Xelor).
Xelor's Gelure converts current mana to reserve BEFORE the round ends, which is the only way to "save" current mana.

Example: Round 3 → 3 mana available. Round 12 → 10 mana (capped). After claiming 2 fake objectives (if both random rewards yielded mana cap) → 12 mana.

**Mana cap increase from fake objective reward:**
```
mana_cap += 1  (permanent, applied immediately when reward is claimed)
minimum mana_cap = 10 (base)
maximum mana_cap = 12 — only achievable if BOTH fake destruction rewards randomly yielded
                        mana cap +1 (each is a 50/50 random draw; 12 is not guaranteed)
```

**Reserve mana (all classes — universal mechanic):**
```
// Sources available to ALL classes:
reserve += 1  (by playing the "+1 reserve" spell card received from Prism Lane 2/4)

// Sources available to XELOR ONLY (via class spells):
reserve += current_mana_transferred  (via Gelure — converts current mana to reserve)
reserve += 1  (per opponent card played, via Miss Nuit)

// No maximum cap on reserve — organic board pressure is the intended limiter

// Spending:
// Any card can be paid from reserve OR current mana (player chooses)
// Exception: cards with explicit "costs from reserve" text MUST use reserve
// (e.g., Garde-Temps: costs 20 reserve mana specifically)
```

### 4.2 Gold Economy

```
gold_at_start_of_round = gold_previous + baseline + interest_bonus
baseline = 2 (every round)
interest_bonus = min(floor(gold_held_at_previous_resolution / interest_threshold_gold), interest_max_bonus)
gold_held_at_previous_resolution = gold balance after RESOLUTION of the prior round

combat_gold = kill_gold + objective_gold
kill_gold = kills_dealt_this_round × 1
objective_gold = objectives_destroyed_this_round × 3  (any objective, real or fake)
```

**Starting gold:** 5 (one-time, before round 1 only).

**Example gold trace — round 3:**
- End of round 2 RESOLUTION: player holds 8 gold
- `interest_bonus = min(floor(8/5), 2) = min(1, 2) = 1`
- Start of round 3 DRAFT: 8 + 2 (baseline) + 1 (interest) = **11 gold available**

### 4.3 Interest Formula

```
interest_bonus = min(floor(g / interest_threshold_gold), interest_max_bonus)
where g = gold held at the END of RESOLUTION (before the next round's DRAFT begins)
      interest_threshold_gold = GameConfig field, default 5 (divisor; determines bracket spacing)
      interest_max_bonus = GameConfig field, default 2 (ceiling bonus)

interest is added at the START of the next DRAFT phase, before any shop spending
```

| Gold held | Interest |
|---|---|
| 0–4 | +0 |
| 5–9 | +1 |
| 10+ | +2 (maximum) |

### 4.4 Auction Mechanics

```
// Only Rare, Epic, and Legendary appear at auction (never Common or Uncommon)
starting_price(rarity) = { Rare: 3, Epic: 4, Legendary: 5 }
minimum_bid = current_price + 1
timer_on_new_bid = min(current_timer + 5, 20)  // resets up to 20s
winner = player with most recent accepted bid when timer reaches 0
winner_pays = their_bid_amount
tie_break (same price, same server tick): first received by server wins
no_bid_outcome: card removed from pool for this game
hand_full_on_win: won card discarded; gold still paid
```

### 4.5 Combat Damage

```
net_damage = max(0, ATK_attacker − AR_defender)
```

Damage cannot go negative. All modifiers (RPS, RESISTANCE, VULNERABILITY, buffs) are applied before this formula.

**Full modifier stack (applied in order):**
1. Apply SILENCE (strip keywords)
2. Apply STUN (unit cannot attack)
3. Apply type advantage ATK bonus (+1 if applicable)
4. Apply RESISTANCE/VULNERABILITY to attacker's base ATK
5. Determine ARMOR-PIERCING (if yes: AR_defender = 0 for this attack; no other modifiers are affected)
6. Apply type advantage AR bonus (+1 to attacker's AR if applicable — applies regardless of ARMOR-PIERCING)
7. `net_damage = max(0, ATK_modified − AR_modified)`

### 4.6 RPS Type Advantage

```
triangle: Blade > Arcane > Shield > Blade
Neutral: no advantage, no disadvantage

if attacker.type beats defender.type:
    combat_ATK_attacker += 1
    combat_AR_attacker += 1  // absorbs 1 more retaliation damage
// these bonuses are for this combat only; they do not change the card's base stats
```

**Type assignment status:** All cards default to Neutral until explicitly assigned. Type field exists in card data and will be populated progressively.

### 4.7 Objective Damage

```
// at end of each RESOLUTION (sub-step 6):
for each lane:
    if any attacker_unit.position == lane.cell_8:
        objective.hp -= attacker_unit.ATK
    if objective.hp <= 0:
        objective.destroyed()
        award_gold(attacker_player, 3)
        if objective.is_fake:
            apply_fake_rewards(attacker_player, lane)
        else:
            check_loss_condition(defending_player)

// unit stays at cell_8 and attacks again next round unless killed
```

```
// loss condition check:
if count(real_objectives_destroyed(defending_player)) >= 2:
    defending_player.loses()
// fake objectives destroyed do NOT count toward this threshold
```

**Spell-based objective damage (direct targeting — bypasses Cell 8 requirement):**
```
// For spells, prism spell cards, or card effects that directly target an objective:
objective.hp -= damage_value
if objective.hp <= 0:
    objective.destroyed()
    if attacker_player != defending_player:  // no reward for self-destroy
        award_gold(attacker_player, 3)
        if objective.is_fake:
            apply_fake_rewards(attacker_player, lane)
        else:
            check_loss_condition(defending_player)
    else:
        check_loss_condition(defending_player)  // self-destroy still counts toward loss
// attacker_player = controller of the card that dealt the damage
// No cell position requirement for spell-based damage
```

### 4.8 Shop TFT Weighting

```
// for each of the 3 shop slots:
roll = random(0, 1)
if roll < 0.5:
    slot_type = "class"
else:
    slot_type = "neutral"

// within class slot:
for each card_id X in player's class:
    base_weight = 1 / total_class_card_ids
    weight(X) = base_weight + (0.10 × total_acquired(X))
    weight(X) = min(weight(X), 0.65)  // cap at 65%
normalize all weights to sum to 1
roll card from weighted distribution

// total_acquired(X) = cumulative copies of X purchased via distribute(), never reset
// For class slot: X = individual card_id. For neutral slot: X = family name.
// See card-data-pool.md Formula 2 for full specification.
```

---

## 5. Edge Cases

**Xelorium (steals all opponent current mana):**
The opponent has 0 current mana for this round and cannot play any card. This is a deliberate high-impact play; the "no idle spectating" pillar applies at the strategic level (watching the board is still active) but the opponent cannot act with cards this round. Xelorium costs Xelor significant reserve; both sides accept the tradeoff.

**FIRST STRIKE vs FIRST STRIKE:**
Both units attack simultaneously. Neither has priority. Both damage values are calculated from pre-combat stats, then applied simultaneously. If both die, both DEATH triggers fire.

**Simultaneous effects — no priority:**
If Unit A's DEATH trigger and Unit B's DEATH trigger both fire at the same moment, both resolve fully. If A's trigger kills B (creating a chain), B's DEATH trigger fires after A's trigger completes.

**Win condition and fake objectives:**
Destroying 2 FAKE objectives does NOT cause the defender to lose. Only 2 REAL objectives destroyed triggers defeat. A player can have both fakes destroyed and still be in the game — the attacker received two sets of fake bonuses (expanded spawn ×2 + mana cap or card pick ×2) but has NOT won.

**Punition (Sacrier) self-sacrifice:**
When Punition destroys one of your own dofus (you chose):
- **If fake:** loss condition NOT advanced. No reward — you are not the "attacker." The dofus is removed with no bonus.
- **If real:** loss condition advances by 1 (counts toward the 2-real threshold). No +3 gold is awarded — you are the defending player, not an attacker. Gold rewards only apply when an opponent destroys your objective.

**Double-tranchant cards that destroy your own fake:**
A self-inflicted fake destruction gives no expanded spawn or mana reward (the card's cost is a negative effect, not a reward trigger). Real objectives self-destroyed via double-tranchant DO count toward your loss condition.

**"Sang Méprise" (Sacrier Krosmic — reveals all dofus):**
Reveals all 5 objectives of all teams (real and fake status visible to everyone) for the duration of this resolution phase. Information resets to hidden the following round. Does not change the objectives — just reveals them.

**Hand full during auction win:**
The auction winner's card is discarded. Gold is still paid. This is a binding commitment — a player who bids on an auction with a full hand pays and loses the card. (Skill expression: manage hand size before auction rounds.)

**Mana cap at 12 for Xelor:**
Xelor's mana_cap applies to current-round mana gain. A Xelor player with mana_cap=12 gains 12 current mana on round 12+. This also provides 12 mana to transfer to reserve via Gelure, accelerating reserve accumulation. Intended — Xelor rewards controlling fakes.

**Team modes — placement with 2 units per lane:**
Each player on a team independently controls their own unit slot per lane. Player A submits their unit for lane 3; Player B independently submits their unit for lane 3. The team limit of 2 units per lane per side is enforced server-side: if both submit for the same lane and the combined count would exceed 2, the second submission (by server arrival order) is rejected and gold is refunded.

**Disconnection:**
Disconnection = **immediate defeat**. Grace period: 30 seconds from connection drop detection. If reconnected within 30 seconds, game resumes normally. In team modes (2v2, 3v3): **one disconnection forfeits the entire team.** (30s is intentional for WASM/browser — OS interrupts and tab switches routinely cause 3–6s connection gaps.) The team's units remain on board until a new round begins, then are removed.

**Placement timeout (no submission received):**
If a player's placement packet is not received by server when the timer expires, they are treated as playing zero cards that round. Existing board state (units already deployed from previous rounds) continues as normal. No refund or compensation.

**No-bid auction:**
If the auction timer expires with zero bids placed, the card is removed from the pool and does not appear again this game.

**Prism Lane 2/4 spell cards:**
Collecting a Lane 2/4 prism adds a spell card to your hand: "Add +1 mana to your reserve." Playing this card (at any time during DRAFT phase, at any round) adds +1 to your reserve. This applies to ALL classes — the +1 goes to the reserve, not to current-round mana.

**RANGE units and BODYGUARD:**
RANGE attacks can bypass BODYGUARD if the RANGE reaches the target unit's cell directly. BODYGUARD blocks attacks targeting adjacent units from non-RANGE attackers only. RANGE attackers that target the BODYGUARD unit itself are resolved normally.

---

## 6. Dependencies

| Dependency | Status | Notes |
|---|---|---|
| Krosmaga Extension=1 card data | Required | Stats base (ATK/PV/MP/AR), keywords per card |
| Card type field (Blade/Arcane/Shield/Neutral) | Partially needed | Defaults to Neutral; RPS system inactive for untyped cards |
| `krosmaga-cards-reference.md` | In repo | Reference for card text; `design/gdd/krosmaga-cards-reference.md` |
| Socket.io session/room system | Required before combat | Manages player connections, placement submission, auction events |
| Server-side RNG (for Ecaflip dice, shop rolls, fake objective reward randomization) | Required | All randomness is server-seeded |
| Art assets | Blocking for UI | Art direction: `design/art-references/`; style: Ankama/Dofus/Wakfu cel-shaded |
| Legendary card 3-level evolution system (Krosmaga "Infinite") | Optional/deferred | Simplify to single-level cards for hackathon; TBD |
| 2v2v2 central capture point bonus value | Deferred | Explicitly TBD; implement capture mechanic but assign bonus value during dev |

---

## 7. Tuning Knobs

All values below are configurable. Ranges indicate safe bounds for playtesting before exceeding the intended design.

| Parameter | Default | Safe Range | Gameplay Impact |
|---|---|---|---|
| `mana_cap` | 10 | 6–14 | Higher = more cards playable per round; dramatically changes tempo |
| `gold_baseline_per_round` | 2 | 1–4 | Core economy pacing; affects interest threshold timing |
| `interest_max_bonus` | +2 | +1 to +3 | Higher = stronger hoard incentive; lower = less snowball |
| `interest_threshold_gold` | 5 | 5–10 | Divisor in `floor(g / interest_threshold_gold)` — bracket spacing. At 5: interest fires at 5g, 10g. At 10: only at 10g+. Do not set below 5 (below 5 removes the miser/gambler tension). |
| `objective_hp` | 5 | 3–8 | Lower = faster games; higher = more comeback potential |
| `fake_count` | 2 of 5 | 1–3 | More fakes = more bluff; fewer = more direct information |
| `objective_gold_reward` | 3 | 2–5 | Higher = more snowball from first destruction |
| `kill_gold_reward` | 1 | 0–2 | 0 = remove combat gold loop; 2 = stronger snowball |
| `starting_gold` | 5 | 3–8 | Higher = more initial draft choice; lower = more constraint |
| `auction_timer_seconds` | 20 | 10–30 | Affects bluff depth; shorter = more pressure |
| `auction_timer_reset_seconds` | +5 | +3 to +10 | How much each bid extends the timer |
| `placement_timer_seconds` | 10 | 5–20 | Shorter = more reflex; longer = more deliberation |
| `shop_weight_per_card` | 10% | 2%–15% | Higher = more archetype focus; lower = more variety. 10% creates felt archetype recognition by round 5-6. |
| `shop_weight_cap` | 65% | 50%–80% | Activates at ~7 owned copies; prevents scripted late-game shop. |
| `fake_objective_spawn_advance` | 1 cell | 1–2 | How far attacker's spawn advances in that lane per fake |
| `xelor_sablier_steal` | 1 | 1–3 | Mana stolen from opponent per Sablier cast |

---

## 8. Acceptance Criteria

All criteria below must pass before the game is considered feature-complete. **BLOCKING** criteria are gates for sprint completion. **ADVISORY** criteria are quality gates for the polish pass.

### Economy

| # | Criterion | Type |
|---|---|---|
| E1 | Given round R, any player's current mana equals `min(R, mana_cap)` at the start of DRAFT phase (mana refills at DRAFT start, before any spending). | BLOCKING |
| E2 | Unspent current mana is discarded at end of each round for ALL classes (including Xelor). Only reserve mana persists. | BLOCKING |
| E3 | A player with 10 gold at end of RESOLUTION receives exactly +2 interest at start of next DRAFT. | BLOCKING |
| E4 | A player with 9 gold at end of RESOLUTION receives exactly +1 interest. | BLOCKING |
| E5 | A player with 4 gold at end of RESOLUTION receives exactly +0 interest. | BLOCKING |
| E6 | Baseline +2 gold is added at the start of each DRAFT phase before any purchases. | BLOCKING |
| E7 | +1 gold is awarded to the killing player immediately when their unit kills an enemy unit. | BLOCKING |
| E8 | +3 gold is awarded to the attacking player immediately when an objective is destroyed (any objective, real or fake). | BLOCKING |
| E9 | Purchasing a card when hand size is exactly 10 is rejected; gold is not deducted. | BLOCKING |
| E10 | Starting gold of 5 is granted once at game start, before round 1. | BLOCKING |
| E11 | Interest is applied before any draft spending in the round it is granted. | BLOCKING |

### Auction

| # | Criterion | Type |
|---|---|---|
| A1 | Auction fires at rounds 1-3, 1-6, 2-3, 2-6 (and continues the Stage.Round 3/6 pattern). | BLOCKING |
| A2 | Auction starting price equals rarity cost (Rare=3, Epic=4, Legendary=5). Common and Uncommon cards never appear at auction. | BLOCKING |
| A3 | No bid can be accepted at less than `current_price + 1` gold. | BLOCKING |
| A4 | The auction timer resets to `min(current_timer + 5, 20)` on each accepted bid. | BLOCKING |
| A5 | The current price and current leader are visible to all players throughout the auction. | BLOCKING |
| A6 | When the timer reaches 0, the current leader wins and pays their bid; gold is deducted. | BLOCKING |
| A7 | If the auction timer expires with zero bids, the card is removed from this game's pool. | BLOCKING |
| A8 | A bid received after timer=0 is rejected; gold is not deducted. | BLOCKING |
| A9 | If the auction winner has 10 cards in hand, the won card is discarded and gold is still paid. | BLOCKING |
| A10 | Simultaneous bids at the same price are resolved in server-arrival order; one player wins. | BLOCKING |

### Combat & Win Condition

| # | Criterion | Type |
|---|---|---|
| C1 | `net_damage = max(0, ATK_attacker − AR_defender)` for all combat interactions. | BLOCKING |
| C2 | Two FIRST STRIKE units facing each other deal damage simultaneously (neither applies damage before the other). | BLOCKING |
| C3 | A unit at Cell 8 (far edge) at end of RESOLUTION deals ATK damage to the objective in that lane. | BLOCKING |
| C4 | A unit at Cell 8 is NOT removed after attacking the objective; it persists and attacks again next round. | BLOCKING |
| C5 | Objectives have exactly 5 HP and 0 AR. They cannot be healed. | BLOCKING |
| C6 | Destroying a fake objective does NOT advance the loss condition. Destroying a real objective advances it by 1. | BLOCKING |
| C7 | The defending player loses exactly when 2 of their real objectives have been destroyed. Not 1, not 3. The server broadcasts `S2CGameOver { loser, round, reason: GameOverReason::ObjectivesDestroyed }` on the reliable channel. See `round-state-machine.md` Rule 14. | BLOCKING |
| C8 | Destroying a fake objective: attacker receives expanded spawn in that lane + a **randomly-assigned** bonus (server-determined: mana cap +1 OR free card pick, 50/50). The player does not choose which bonus they receive. | BLOCKING |
| C9 | Expanded spawn persists for the remainder of the game (attacker's units in that lane spawn 1 cell further). | BLOCKING |
| C10 | ARMOR-PIERCING treats defender AR as 0 for the attacker's outgoing damage. The attacker's own AR (including any RPS type-advantage AR bonus) is unaffected. | BLOCKING |
| C11 | Type advantage gives the advantaged attacker +1 ATK and +1 AR for that combat only. | ADVISORY |
| C12 | Neutral-type units receive no type advantage or disadvantage. | ADVISORY |

### Multiplayer Sync

| # | Criterion | Type |
|---|---|---|
| M1 | When all players submit placement, RESOLUTION begins immediately (does not wait for timer). | BLOCKING |
| M2 | When the placement timer expires, RESOLUTION begins with whatever placements were received; missing players play zero cards. | BLOCKING |
| M3 | In 2v2, a team cannot have more than 2 units per lane per side; the second submission for an already-full lane is rejected and gold is refunded. | BLOCKING |
| M4 | A player disconnected for >30 seconds is declared defeated; their team forfeits in team modes. | BLOCKING |
| M5 | All Ecaflip 1d6 rolls are computed server-side; clients receive the result, not the seed. | BLOCKING |
| M6 | Xelor reserve changes (Gelure transfer, Miss Nuit trigger, Sablier steal, Xelorium steal) are broadcast to all clients immediately. Xelorium steals current mana only — opponent reserve is unchanged and broadcast separately. | BLOCKING |
| M7 | Gold balances are broadcast to all players and visible in the UI during auctions (enabling informed bluff decisions). | BLOCKING |

### Shop & Draft

| # | Criterion | Type |
|---|---|---|
| D1 | Personal shop refreshes at the start of each DRAFT phase; prior unsold cards are removed. | BLOCKING |
| D2 | Manual refresh (1 gold) can only be triggered during DRAFT phase, before PLACEMENT begins. | BLOCKING |
| D3 | Each shop slot independently rolls 50% class / 50% neutral. | BLOCKING |
| D4 | Shop weighting increases probability for card types the player owns more of; weight is capped at 65% per type. | ADVISORY |

### Prism System

| # | Criterion | Type |
|---|---|---|
| P1 | Collecting a Lane 1 or Lane 5 prism adds the "1 damage to a chosen objective" spell card to the collecting player's hand. | BLOCKING |
| P2 | Collecting a Lane 2 or Lane 4 prism adds the "+1 mana to reserve" spell card to the collecting player's hand (not an immediate mana injection). | BLOCKING |
| P3 | Collecting a Lane 3 prism draws 1 card randomly from the shop pool into the collecting player's hand. | BLOCKING |
| P4 | Playing the "+1 reserve" spell card (from Lane 2/4 prism) adds exactly +1 to the player's reserve mana and can be played at any point during the DRAFT phase. | BLOCKING |
| P5 | Each player's 5 prisms respawn independently when that player has collected all 5 of their own prisms. The opponent's collection pace does not affect the player's respawn cycle. | BLOCKING |
| P6 | Playing the "1 damage to dofus" spell card deals exactly 1 damage to a player-chosen objective (real or fake), bypassing lane position. Costs 3 mana. | BLOCKING |

### Stress Test (Advisory)

| # | Criterion | Type |
|---|---|---|
| S1 | 6-player 2v2v2 game completes a full round (DRAFT+PLACEMENT+RESOLUTION) in under 5 seconds of server processing time after all placements are received. | ADVISORY |
| S2 | Shop generation for a full 298-card pool completes in under 100ms per player per round. | ADVISORY |
| S3 | No observable gold desync between client and server after 10 full rounds in a 1v1 game. | ADVISORY |

---

## Open Questions (TBD During Development)

1. **2v2v2 central capture bonus value:** Mechanic is implemented; bonus amount requires playtesting to determine.
2. **Limited card evolution levels:** Simplify to single-level for hackathon? Decision deferred.
3. **Auction card count per auction round:** Currently 1 card per auction. Consider 1 per N players in larger modes.
4. **Balance pass:** Mana costs, gold costs, stat values — all from Krosmaga base; will need adjustment after first playtests.
5. **RPS type assignment:** Type field exists; most cards are Neutral. Full assignment pass is post-hackathon work.
6. **Disconnect grace period — RESOLVED:** Confirmed at 30s for WASM/browser target. Rationale in RSM GDD Rule 13.
