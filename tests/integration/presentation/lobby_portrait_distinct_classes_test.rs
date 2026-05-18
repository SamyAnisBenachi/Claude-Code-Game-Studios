//! PROMPT 1081 — Lobby class picker no longer renders all-`?` placeholder
//! art. AUDIT-1076-06 reported every one of the 7 lobby class tiles paints
//! the same generic `?` glyph because the canonical
//! `art/ui/lobby/ui_class_portrait_*.png` files on disk are byte-identical
//! (one shared MD5 across all 7 classes plus the room-code chip and slot
//! panel — all stamped from the same placeholder PNG).
//!
//! The repair in `client/src/asset_wiring.rs` repoints each
//! `LOBBY_PORTRAIT_*_ASSET` to the class-specific frame in
//! `art/ui/hand/card_frame_{class}_default_display.png` (verified
//! all-different MD5s). This test guards the repair against silent
//! regression by asserting:
//!
//! - **AC1** — `lobby_portrait_asset(class_id)` returns a unique path for
//!   every one of the 7 `ClassId` variants (no two classes resolve to the
//!   same string).
//! - **AC2** — Each resolved path exists on disk under the project's
//!   `assets/` directory.
//! - **AC3** — The on-disk file content for the 7 paths is byte-distinct
//!   under SHA-256, so a future regression that re-stamps every class
//!   path to the same placeholder PNG fails this test immediately rather
//!   than silently restoring the all-`?` render.
//!
//! ## ADR alignment
//!
//! - **ADR-021 Presentation Layer Architecture**: read-only assertion on
//!   the path constant table; no protocol shape exercised.
//! - No new accessibility / playtest claims are made. Friend-game scope
//!   only; `QA-COND-0005` Standard-tier and `QA-COND-0006` playtest
//!   validation stay on accept-risk.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use client::asset_wiring::lobby_portrait_asset;
use shared::card::ClassId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const ALL_CLASS_IDS: [ClassId; 7] = [
    ClassId::Iop,
    ClassId::Cra,
    ClassId::Sacrier,
    ClassId::Xelor,
    ClassId::Ecaflip,
    ClassId::Sadida,
    ClassId::Neutral,
];

fn assets_root() -> PathBuf {
    // `client/Cargo.toml` is the test's CARGO_MANIFEST_DIR; the workspace
    // root lives one directory up. Assets are under `<workspace>/assets/`.
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("client crate has a parent workspace root")
        .join("assets")
}

#[test]
fn ac1_lobby_portrait_paths_are_unique_per_class() {
    test_helpers::init_test_tracing();

    let mut paths = HashSet::new();
    let mut duplicates = Vec::new();
    for class_id in ALL_CLASS_IDS {
        let path = lobby_portrait_asset(class_id);
        if !paths.insert(path) {
            duplicates.push((class_id, path));
        }
    }

    assert!(
        duplicates.is_empty(),
        "AC1: every class must resolve to a unique portrait path so the \
         lobby picker shows class-distinct art; found duplicates: \
         {duplicates:?}. Reverting to the all-`?` placeholder set would \
         re-introduce AUDIT-1076-06."
    );
    assert_eq!(
        paths.len(),
        ALL_CLASS_IDS.len(),
        "AC1: expected {} unique portrait paths (one per class), got {}",
        ALL_CLASS_IDS.len(),
        paths.len()
    );
}

#[test]
fn ac2_each_portrait_path_exists_on_disk() {
    test_helpers::init_test_tracing();

    let root = assets_root();
    for class_id in ALL_CLASS_IDS {
        let rel_path = lobby_portrait_asset(class_id);
        let full = root.join(rel_path);
        assert!(
            full.is_file(),
            "AC2: portrait path for ClassId::{:?} resolves to `{}` but \
             that file does not exist on disk at `{}`. The repair must \
             only point at existing assets — broken paths render as a \
             magenta missing-asset placeholder.",
            class_id,
            rel_path,
            full.display()
        );
    }
}

#[test]
fn ac3_portrait_files_are_byte_distinct_across_classes() {
    test_helpers::init_test_tracing();

    let root = assets_root();
    let mut digests: Vec<(ClassId, &'static str, [u8; 32])> = Vec::new();
    for class_id in ALL_CLASS_IDS {
        let rel_path = lobby_portrait_asset(class_id);
        let full = root.join(rel_path);
        let bytes = std::fs::read(&full).unwrap_or_else(|err| {
            panic!(
                "AC3: failed to read portrait file `{}` for ClassId::{:?}: {err}",
                full.display(),
                class_id
            )
        });
        let digest = sha256(&bytes);
        digests.push((class_id, rel_path, digest));
    }

    // Compare every pair; report the FIRST collision with both
    // identities so the diagnostic is actionable.
    for i in 0..digests.len() {
        for j in (i + 1)..digests.len() {
            let (lhs_class, lhs_path, lhs_digest) = &digests[i];
            let (rhs_class, rhs_path, rhs_digest) = &digests[j];
            assert_ne!(
                lhs_digest, rhs_digest,
                "AC3: ClassId::{:?} (`{}`) and ClassId::{:?} (`{}`) point \
                 at byte-identical PNGs. The repair guarantee is that \
                 every class renders distinct art; matching MD5/SHA \
                 across two classes is exactly the AUDIT-1076-06 \
                 regression that this test guards against. Re-stamp \
                 with class-specific art or repoint the constants.",
                lhs_class, lhs_path, rhs_class, rhs_path,
            );
        }
    }
}

#[test]
fn ac4_friend_game_scope_no_claim_documented_inline() {
    // Source-embedded scope guard mirroring
    // `lobby_button_dimensions_test::ac5_friend_game_scope_no_claim_documented_inline`.
    let source = include_str!("lobby_portrait_distinct_classes_test.rs");
    assert!(
        source.contains("QA-COND-0005"),
        "AC4: friend-game-scope no-claim restatement must reference QA-COND-0005"
    );
    assert!(
        source.contains("QA-COND-0006"),
        "AC4: friend-game-scope no-claim restatement must reference QA-COND-0006"
    );
    assert!(
        source.contains("accept-risk"),
        "AC4: friend-game-scope no-claim restatement must reference accept-risk"
    );
}

// ── Minimal SHA-256 (FIPS 180-4) ─────────────────────────────────────────
//
// Pulled in-file so the test crate does not need a new dev-dependency. SHA-256
// is the canonical content-distinctness oracle for asset-regression guards;
// MD5 would suffice but openly-collidable hashes flag friction in security
// reviews. Length-extension attacks are irrelevant for a content fingerprint
// of static PNGs.

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

const SHA256_H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

fn sha256(bytes: &[u8]) -> [u8; 32] {
    let bit_len = (bytes.len() as u64).wrapping_mul(8);
    let mut padded: Vec<u8> = Vec::with_capacity(bytes.len() + 64 + 8);
    padded.extend_from_slice(bytes);
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    let mut h = SHA256_H0;
    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

#[test]
fn sha256_self_check() {
    // Sanity check against a known SHA-256 vector ("abc") so AC3 trusts
    // its own digest function: a bug in the local hasher would otherwise
    // silently let two byte-identical files report as distinct.
    let digest = sha256(b"abc");
    let expected: [u8; 32] = [
        0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22,
        0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00,
        0x15, 0xad,
    ];
    assert_eq!(
        digest, expected,
        "SHA-256 self-check vector for `abc` must match"
    );
}
