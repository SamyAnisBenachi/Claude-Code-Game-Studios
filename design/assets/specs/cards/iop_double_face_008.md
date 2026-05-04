# Card Illustration Spec - iop_double_face_008

> **Card**: Double-Face Blade / Lame Double-Face
> **Card ID**: 8
> **Asset ID**: ASSET-234
> **Source**: assets/data/cards.json
> **Status**: Needed

## Production Contract

Illustration only. Runtime card composition owns the frame, mana badge, ATK/HP badges, text, type/rarity label, hover, ghost, drag, and state overlays. DoubleFace transform state is a runtime/card-data concern; do not bake both faces as UI chrome.

| Field | Value |
|---|---|
| Category | Card Illustration |
| Canvas | 240x360 PNG-32 zoom master; 120x180 display derivative |
| Naming | `card_iop_double_face_008_art_zoom.png`; `card_iop_double_face_008_art_display.png` |
| Atlas | display derivative in `atlas_cards`; zoom loaded on demand |

## Visual Brief

An Iop blade wielder with a split-intent pose: one side restrained and tactical, the other side aggressive after a killing blow. Show the dual-face concept through mirrored lighting or a double-edged weapon, not through baked UI panels. The silhouette must still read as one Iop Blade unit.

## Generation Prompt

Ankama Wakfu style card illustration only, Iop double-face blade warrior, single character with double-edged weapon, split-intent pose, one restrained side and one aggressive side, warm orange-red Iop accents, Arcane Gold edge highlights, bold Void outlines, saturated cel shade, readable as one Blade unit, 240x360 portrait illustration, no card frame, no text, no stat badges, no rarity icon, no UI overlay, no photorealism, no split UI layout.
