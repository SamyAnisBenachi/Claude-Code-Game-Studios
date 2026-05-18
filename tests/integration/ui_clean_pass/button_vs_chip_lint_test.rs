//! S18-UI-LAYOUT-CONTRACT-DOC-AND-LINT-001 (PROMPT 1188 / Lane K).
//!
//! Conservative lint over `client/src/ui/**` and
//! `client/src/presentation/**` enforcing the structural invariants
//! ratified in `docs/ux/global-ui-layout-contract.md` §6 (button vs
//! status chip) and §5 (`Overflow::visible()` justification).
//!
//! The lint is **allowlisted-baseline by design**: existing violations
//! recorded against the audit at `reports/PROMPT-1180-global-ui-layout-
//! system-deep-audit.md` are entered into the baseline so the current
//! branch is not blocked, and only *new* violations fail.
//!
//! Rules implemented (per contract §10):
//!
//! - **L1** every `.spawn((` tuple containing the `Button` marker must
//!   also contain `BackgroundColor(` (or a documented helper-builder
//!   site in `BUTTON_BG_BUILDER_ALLOWLIST`).
//! - **L2** every `.spawn((` tuple containing the `Button` marker must
//!   also contain `Interaction::` initialiser.
//! - **L3** every `Overflow::visible()` occurrence in `client/src/ui/**`
//!   or `client/src/presentation/**` must (a) live on a justification
//!   comment matching `// AC: ` on the same statement or one of the
//!   two preceding lines, OR (b) be entered explicitly in the
//!   `OVERFLOW_VISIBLE_BASELINE` allowlist. The strip-primitive module
//!   `client/src/ui/design_tokens/strips.rs` is excluded by path
//!   because strip primitives are the only legitimate consumer (the
//!   `HandBar` parent intentionally lets the hand fan extend past the
//!   strip — see contract §5).
//!
//! Scope guards:
//!
//! - The lint is intentionally conservative (false-positive-avoidance);
//!   advisory rule **L4** (status-chip-styled-as-button) is documented
//!   in the contract but **NOT implemented here** — the static
//!   detection surface is too brittle to ship without per-surface
//!   refinement. It is reserved for a Sprint 18+ follow-on lane.
//! - The lint does NOT do AST parsing. It does line-windowed text
//!   scans over a single spawn tuple, bounded by paren depth from the
//!   opening `.spawn((` to the matching closing `));`. This mirrors
//!   the existing `ui_clean_pass/strips_test.rs::ac3_*` grep-guard
//!   style and the `strips_test.rs::ac7_*` `_GAP_PX` guard.
//! - The lint does NOT modify production code. PROMPT 1188 forbids
//!   any write to `client/src/**`.
//!
//! No protocol change, no server change, no shared change.
//! See `docs/ux/global-ui-layout-contract.md` for the binding contract
//! and `reports/PROMPT-1180-global-ui-layout-system-deep-audit.md`
//! §6 Lane K for the lane-level acceptance shape.

use std::fs;
use std::path::{Path, PathBuf};

#[path = "../../test_helpers.rs"]
mod test_helpers;

// ─── Roots scanned by the lint ─────────────────────────────────────────

/// Production-code roots the lint walks for `Button` and
/// `Overflow::visible()` occurrences. Test files, design-token modules,
/// and the strip primitives are intentionally excluded — see the
/// `is_path_excluded` predicate below.
const SCAN_ROOTS_RELATIVE_TO_CLIENT: &[&str] = &["src/ui", "src/presentation"];

/// File paths (relative to `client/`) excluded from the L1 / L2 button
/// lint. These modules either (a) author the strip primitives that are
/// the documented exception, or (b) host marker structs only — no
/// spawn sites.
fn is_path_excluded(rel_path: &str) -> bool {
    // The strip primitive module declares the one legitimate
    // `Overflow::visible()` consumer per contract §5.
    rel_path.replace('\\', "/") == "src/ui/design_tokens/strips.rs"
}

// ─── L1 / L2 allowlist ─────────────────────────────────────────────────

/// Builder helper functions that supply `BackgroundColor` outside the
/// spawn tuple. If a `Button` spawn tuple invokes one of these
/// builders, the lint accepts the tuple as L1-conformant even when
/// `BackgroundColor(` is not present in the same tuple text.
///
/// Empty by default. Wave-2 lanes (PROMPT 1194 / 1196 / 1197 / 1199)
/// may introduce a `xxx_button_node()` builder that bundles the
/// background; in that case the lane adds the helper name here.
const BUTTON_BG_BUILDER_ALLOWLIST: &[&str] = &[];

/// Baseline violations carried over from the PROMPT 1180 audit. Each
/// entry is `(file_rel_to_client, line_of_button_marker, rule)`. The
/// lint records a baseline-listed site as `BASELINE` rather than
/// `VIOLATION`, and panics with `STALE` if the recorded site no
/// longer violates the rule.
///
/// The baseline records the L1 violations the lint observes at
/// `origin/main@efb698e` (the PROMPT 1180 audit cut). Each entry
/// corresponds to a real audit finding and is owned by a Wave-2 lane
/// that will either supply the missing `BackgroundColor` directly or
/// register a builder helper in `BUTTON_BG_BUILDER_ALLOWLIST`.
///
/// Sites listed:
///
/// - `src/ui/hand/mod.rs:3362 / :3503 / :3774` — hand-fan slot
///   parents (audit F-01 / F-02). The slot's visible background is
///   painted by the child `HandCardFrame` chrome, not by the slot
///   parent. Owned by Wave-2 Lane C (PROMPT 1192 — card-art +
///   label-strip primitive) which will either move the background to
///   the parent or register the chrome builder as an allowlisted
///   helper.
/// - `src/ui/lobby.rs:1637` — `LobbyConfirmClassButton` spawn
///   (audit L-03; UI-1129-08). The confirm CTA is the headline
///   button-styled-as-text symptom. Owned by Wave-2 Lane E
///   (PROMPT 1194 — lobby panel overflow + confirm CTA).
/// - `src/ui/shop_auction/mod.rs:4434 / :4514 / :4537 / :4615 / :4634`
///   — draft / shop / auction control buttons (audit S-03 / S-05).
///   Background is currently supplied by a sibling builder
///   (`draft_initial_ready_button_node()`, `bid_button_node()`)
///   rather than `BackgroundColor` on the tuple. Owned by Wave-2
///   Lane H (PROMPT 1197 — shop / auction paint + bid label).
const BUTTON_NO_BG_BASELINE: &[(&str, usize)] = &[
    ("src/ui/hand/mod.rs", 3362),
    ("src/ui/hand/mod.rs", 3503),
    ("src/ui/hand/mod.rs", 3774),
    ("src/ui/lobby.rs", 1637),
    ("src/ui/shop_auction/mod.rs", 4434),
    ("src/ui/shop_auction/mod.rs", 4514),
    ("src/ui/shop_auction/mod.rs", 4537),
    ("src/ui/shop_auction/mod.rs", 4615),
    ("src/ui/shop_auction/mod.rs", 4634),
];

/// L2 baseline. The single site is the QA snapshot button — a
/// dev-overlay button that relies on Bevy 0.18's Required Components
/// to inject `Interaction` automatically rather than declaring it
/// explicitly. PROMPT 1140 ratified this overlay as the QA snapshot
/// affordance; the lint surfaces it once for future hardening but
/// does not block the current branch.
const BUTTON_NO_INTERACTION_BASELINE: &[(&str, usize)] = &[
    ("src/presentation/qa_snapshot.rs", 1787),
];

// ─── L3 allowlist ──────────────────────────────────────────────────────

/// Baseline `Overflow::visible()` sites that lack the `// AC: `
/// justification comment. PROMPT 1180 audit RC-2 / H-01 / H-02
/// identified these two HUD strip sites as the only non-strip-primitive
/// consumers. PROMPT 1196 (Lane G — HUD top-strip wrap + opp class
/// repair) clears both entries; the lint reports them as `STALE` on
/// lane landing so the migrating worker remembers to delete them.
const OVERFLOW_VISIBLE_BASELINE: &[(&str, usize)] = &[
    ("src/ui/hud/mod.rs", 2806),
    ("src/ui/hud/mod.rs", 2816),
];

// ─── Path resolution ────────────────────────────────────────────────────

fn client_crate_root() -> PathBuf {
    // CARGO_MANIFEST_DIR resolves to client/ because this integration
    // test bin lives in client/Cargo.toml.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn normalise_rel(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn collect_rs_files(root: &Path, into: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(root) {
        Ok(e) => e,
        Err(err) => panic!("lint: failed to read dir {}: {err}", root.display()),
    };
    for entry in entries {
        let entry = entry.expect("lint: dir entry");
        let path = entry.path();
        let file_type = entry.file_type().expect("lint: file_type");
        if file_type.is_dir() {
            collect_rs_files(&path, into);
        } else if file_type.is_file()
            && path.extension().is_some_and(|ext| ext == "rs")
        {
            into.push(path);
        }
    }
}

/// Yields `(relative_path_str, full_text)` for every `.rs` file under
/// `client/src/ui` and `client/src/presentation`, skipping the
/// excluded modules.
fn scanned_files() -> Vec<(String, String)> {
    let client_root = client_crate_root();
    let mut paths: Vec<PathBuf> = Vec::new();
    for rel in SCAN_ROOTS_RELATIVE_TO_CLIENT {
        let root = client_root.join(rel);
        if !root.exists() {
            panic!("lint: scan root missing: {}", root.display());
        }
        collect_rs_files(&root, &mut paths);
    }
    let mut out: Vec<(String, String)> = Vec::with_capacity(paths.len());
    for path in paths {
        let rel_path = path
            .strip_prefix(&client_root)
            .expect("lint: path inside client/")
            .to_path_buf();
        let rel = normalise_rel(&rel_path);
        if is_path_excluded(&rel) {
            continue;
        }
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("lint: failed to read {}: {err}", path.display()));
        out.push((rel, text));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

// ─── Spawn-tuple extraction ─────────────────────────────────────────────

/// Given a file text and a 0-based byte offset of the start of a
/// `Button,` token, walk backward to find the most recent `.spawn((`
/// (or `commands.spawn((`) and forward to the matching closing `));`
/// at paren depth zero relative to that opening. Returns the full
/// tuple text (between the opening `((` and the closing `))`) or
/// `None` if no enclosing spawn is found within a generous window
/// (4096 characters).
///
/// The walk does not care about preceding identifier characters; it
/// matches the literal substring `.spawn((` anywhere on a line.
fn enclosing_spawn_tuple(text: &str, button_offset: usize) -> Option<&str> {
    // Backward search bound: 4096 chars or start-of-file.
    let back_floor = button_offset.saturating_sub(4096);
    let backward_window = &text[back_floor..button_offset];
    let spawn_marker = ".spawn((";
    let last_spawn = backward_window.rfind(spawn_marker)?;
    let open_paren_abs = back_floor + last_spawn + spawn_marker.len() - 2;
    // open_paren_abs points at the first `(` of the `((`. The tuple
    // body starts immediately after the second `(` — paren depth 1.
    let body_start = open_paren_abs + 2;
    if body_start > text.len() {
        return None;
    }
    // Forward paren scan from body_start at depth 1. We stop when
    // depth returns to 0, indicating we have consumed the matching
    // closing `)` of the outer spawn paren pair.
    //
    // The scan respects line comments (`// ...\n`) and string literals
    // (`"..."`) to avoid mis-counting parens inside text. Block
    // comments and raw strings are not currently used in the scanned
    // files; the lint asserts via the existing strips_test grep-guard
    // style, where simple line scanning has proven sufficient.
    let bytes = text.as_bytes();
    let mut depth: i32 = 1;
    let mut i = body_start;
    let mut in_string = false;
    let mut in_char = false;
    let mut in_line_comment = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_line_comment {
            if b == b'\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_string {
            if b == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if in_char {
            if b == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            if b == b'\'' {
                in_char = false;
            }
            i += 1;
            continue;
        }
        // Detect start of a line comment.
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            in_line_comment = true;
            i += 2;
            continue;
        }
        if b == b'"' {
            in_string = true;
            i += 1;
            continue;
        }
        if b == b'\'' {
            in_char = true;
            i += 1;
            continue;
        }
        if b == b'(' {
            depth += 1;
        } else if b == b')' {
            depth -= 1;
            if depth == 0 {
                // i points at the matching closing paren. Tuple body
                // is text[body_start..i].
                return Some(&text[body_start..i]);
            }
        }
        i += 1;
    }
    None
}

/// Return the 1-based line number of the given byte offset.
fn line_number_at(text: &str, offset: usize) -> usize {
    let safe = offset.min(text.len());
    text[..safe].bytes().filter(|b| *b == b'\n').count() + 1
}

// ─── L1 / L2 ─────────────────────────────────────────────────────────────

/// Find every `Button,` marker occurrence whose enclosing spawn tuple
/// satisfies the predicate (`needle in tuple`). Returns the violations
/// — `(file_rel, button_line_no)` pairs where the predicate is FALSE.
fn collect_button_tuple_violations(
    files: &[(String, String)],
    needle_present: impl Fn(&str) -> bool,
) -> Vec<(String, usize)> {
    let mut violations: Vec<(String, usize)> = Vec::new();
    for (rel, text) in files {
        // Anchor the marker scan to a line-start-prefixed `Button,`
        // possibly after leading whitespace. This matches the
        // canonical spawn-tuple component layout (one component per
        // line) used across the codebase and seen in the audit's
        // sampled spawn sites.
        let mut search_from = 0usize;
        while let Some(rel_pos) = text[search_from..].find("Button,") {
            let abs_pos = search_from + rel_pos;
            search_from = abs_pos + "Button,".len();
            // Require `Button,` to sit on its own line (preceded by
            // whitespace, followed by end-of-line). This avoids
            // false-positives on `ButtonInput`, `ButtonStyleState`,
            // etc.
            let line_start = text[..abs_pos]
                .rfind('\n')
                .map(|nl| nl + 1)
                .unwrap_or(0);
            let line_end = text[abs_pos..]
                .find('\n')
                .map(|nl| abs_pos + nl)
                .unwrap_or(text.len());
            let line = &text[line_start..line_end];
            let trimmed = line.trim();
            if trimmed != "Button," {
                continue;
            }
            let Some(tuple) = enclosing_spawn_tuple(text, abs_pos) else {
                continue;
            };
            if needle_present(tuple) {
                continue;
            }
            // Also accept if any builder allowlist function is called
            // inside the tuple (for L1 only — the caller passes an
            // appropriate predicate that already encodes the builder
            // accept rule).
            violations.push((rel.clone(), line_number_at(text, abs_pos)));
        }
    }
    violations.sort();
    violations
}

fn tuple_has_background_color(tuple: &str) -> bool {
    if tuple.contains("BackgroundColor(") {
        return true;
    }
    BUTTON_BG_BUILDER_ALLOWLIST
        .iter()
        .any(|builder| tuple.contains(builder))
}

fn tuple_has_interaction(tuple: &str) -> bool {
    tuple.contains("Interaction::")
}

#[test]
fn l1_every_button_spawn_tuple_carries_background_color() {
    test_helpers::init_test_tracing();
    let files = scanned_files();
    let observed = collect_button_tuple_violations(&files, tuple_has_background_color);
    let baseline: Vec<(String, usize)> = BUTTON_NO_BG_BASELINE
        .iter()
        .map(|(p, l)| ((*p).to_string(), *l))
        .collect();
    let new_violations: Vec<_> = observed
        .iter()
        .filter(|v| !baseline.contains(v))
        .collect();
    let stale_baseline: Vec<_> = baseline
        .iter()
        .filter(|b| !observed.contains(b))
        .collect();
    assert!(
        new_violations.is_empty(),
        "L1: new Button spawn tuples without BackgroundColor (or an \
         allowlisted builder helper). Either add BackgroundColor to \
         the spawn tuple, route through a helper added to \
         BUTTON_BG_BUILDER_ALLOWLIST, or — if intentional — add the \
         site to BUTTON_NO_BG_BASELINE with an explanatory comment.\n\
         Violations:\n{}",
        format_locations(&new_violations)
    );
    assert!(
        stale_baseline.is_empty(),
        "L1 baseline STALE: the following entries in \
         BUTTON_NO_BG_BASELINE no longer correspond to live \
         violations. Remove them.\nStale entries:\n{}",
        format_baseline(&stale_baseline)
    );
}

#[test]
fn l2_every_button_spawn_tuple_carries_interaction() {
    test_helpers::init_test_tracing();
    let files = scanned_files();
    let observed = collect_button_tuple_violations(&files, tuple_has_interaction);
    let baseline: Vec<(String, usize)> = BUTTON_NO_INTERACTION_BASELINE
        .iter()
        .map(|(p, l)| ((*p).to_string(), *l))
        .collect();
    let new_violations: Vec<_> = observed
        .iter()
        .filter(|v| !baseline.contains(v))
        .collect();
    let stale_baseline: Vec<_> = baseline
        .iter()
        .filter(|b| !observed.contains(b))
        .collect();
    assert!(
        new_violations.is_empty(),
        "L2: new Button spawn tuples without Interaction:: initialiser. \
         Click feedback (PROMPT 1150 ui_picking) requires every \
         clickable surface to declare its Interaction component on \
         spawn. Either add `Interaction::None,` (or `::Hovered`/`::Pressed` \
         where appropriate) to the spawn tuple, or — if intentional — \
         add the site to BUTTON_NO_INTERACTION_BASELINE with an \
         explanatory comment.\n\
         Violations:\n{}",
        format_locations(&new_violations)
    );
    assert!(
        stale_baseline.is_empty(),
        "L2 baseline STALE: the following entries in \
         BUTTON_NO_INTERACTION_BASELINE no longer correspond to live \
         violations. Remove them.\nStale entries:\n{}",
        format_baseline(&stale_baseline)
    );
}

// ─── L3 ──────────────────────────────────────────────────────────────────

/// Find every `Overflow::visible()` occurrence under the scanned
/// roots (strips.rs excluded by `is_path_excluded`) and decide whether
/// the site carries a `// AC: ` justification comment on the same line
/// or one of the two immediately preceding lines.
fn collect_overflow_visible_sites(files: &[(String, String)]) -> Vec<(String, usize, bool)> {
    let mut out: Vec<(String, usize, bool)> = Vec::new();
    for (rel, text) in files {
        let mut search_from = 0usize;
        while let Some(rel_pos) = text[search_from..].find("Overflow::visible()") {
            let abs_pos = search_from + rel_pos;
            search_from = abs_pos + "Overflow::visible()".len();
            let line_no = line_number_at(text, abs_pos);
            let justified = has_ac_comment_near(text, line_no);
            out.push((rel.clone(), line_no, justified));
        }
    }
    out.sort();
    out
}

/// Returns true when the line at `line_no` (1-based) or one of the
/// two preceding lines contains the literal `// AC: ` comment token.
fn has_ac_comment_near(text: &str, line_no: usize) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    let idx = line_no.saturating_sub(1);
    let scan_window = idx.saturating_sub(2)..=idx;
    for i in scan_window {
        if let Some(line) = lines.get(i) {
            if line.contains("// AC: ") {
                return true;
            }
        }
    }
    false
}

#[test]
fn l3_overflow_visible_sites_carry_ac_justification_or_baseline_entry() {
    test_helpers::init_test_tracing();
    let files = scanned_files();
    let sites = collect_overflow_visible_sites(&files);
    let baseline: Vec<(String, usize)> = OVERFLOW_VISIBLE_BASELINE
        .iter()
        .map(|(p, l)| ((*p).to_string(), *l))
        .collect();

    let mut new_violations: Vec<(String, usize)> = Vec::new();
    for (path, line, justified) in &sites {
        if *justified {
            continue;
        }
        let key = (path.clone(), *line);
        if baseline.contains(&key) {
            continue;
        }
        new_violations.push(key);
    }
    new_violations.sort();

    let site_keys: Vec<(String, usize)> = sites
        .iter()
        .map(|(p, l, _)| (p.clone(), *l))
        .collect();
    let stale_baseline: Vec<(String, usize)> = baseline
        .iter()
        .filter(|b| !site_keys.contains(b))
        .cloned()
        .collect();

    assert!(
        new_violations.is_empty(),
        "L3: new `Overflow::visible()` site without a `// AC: \
         <ticket>` justification comment on the same line or the two \
         lines above. The strip primitive module is the ONLY \
         documented consumer (see contract §5). Either (a) drop the \
         override, (b) add a `// AC: <ticket>` justification comment \
         describing why visible overflow is intentional, or (c) add \
         the site to OVERFLOW_VISIBLE_BASELINE with an explanatory \
         comment if it is a deliberate per-prompt accept-risk.\n\
         Violations:\n{}",
        format_locations(&new_violations.iter().collect::<Vec<_>>())
    );

    assert!(
        stale_baseline.is_empty(),
        "L3 baseline STALE: the following entries in \
         OVERFLOW_VISIBLE_BASELINE no longer correspond to live \
         `Overflow::visible()` sites. Remove them so future \
         regressions at the same file:line are caught.\nStale \
         entries:\n{}",
        format_baseline(&stale_baseline.iter().collect::<Vec<_>>())
    );
}

// ─── Baseline self-tests ────────────────────────────────────────────────

#[test]
fn baselines_are_sorted_and_unique() {
    for (name, baseline) in [
        ("BUTTON_NO_BG_BASELINE", &BUTTON_NO_BG_BASELINE[..]),
        (
            "BUTTON_NO_INTERACTION_BASELINE",
            &BUTTON_NO_INTERACTION_BASELINE[..],
        ),
        ("OVERFLOW_VISIBLE_BASELINE", &OVERFLOW_VISIBLE_BASELINE[..]),
    ] {
        let mut sorted = baseline.to_vec();
        sorted.sort();
        assert_eq!(
            sorted,
            baseline.to_vec(),
            "{name} must be sorted by (path, line) so reviewers can \
             diff additions cleanly"
        );
        let mut dedup = sorted.clone();
        dedup.dedup();
        assert_eq!(
            dedup.len(),
            sorted.len(),
            "{name} contains duplicate entries"
        );
    }
}

#[test]
fn scan_roots_resolve_to_real_directories() {
    // Cheap sanity-check that the lint is pointed at the right place
    // when the workspace layout shifts. Failure here means the lint
    // is silently scanning zero files.
    let root = client_crate_root();
    for rel in SCAN_ROOTS_RELATIVE_TO_CLIENT {
        let abs = root.join(rel);
        assert!(
            abs.is_dir(),
            "lint scan root must resolve to a real directory: {} \
             (resolved to {})",
            rel,
            abs.display()
        );
    }
    let files = scanned_files();
    assert!(
        !files.is_empty(),
        "lint scanned zero .rs files — SCAN_ROOTS_RELATIVE_TO_CLIENT \
         is mis-pointed or is_path_excluded is over-aggressive"
    );
}

// ─── Formatting helpers ────────────────────────────────────────────────

fn format_locations(items: &[&(String, usize)]) -> String {
    items
        .iter()
        .map(|(p, l)| format!("  - {p}:{l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_baseline(items: &[&(String, usize)]) -> String {
    items
        .iter()
        .map(|(p, l)| format!("  - (\"{p}\", {l}),"))
        .collect::<Vec<_>>()
        .join("\n")
}
