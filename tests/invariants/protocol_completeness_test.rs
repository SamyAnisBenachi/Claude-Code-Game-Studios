//! Protocol completeness invariant test (S13-PROTO-INVARIANT-001).
//!
//! Every C2S/S2C message type registered through `register_c2s::<T>` or
//! `register_s2c::<T>` in `shared/src/protocol.rs` must have:
//!
//! - For C2S types: at least one **send-site** in `client/src/` (the client
//!   originates the message) AND at least one **drain-site** in
//!   `server/src/` (the server consumes it).
//! - For S2C types: at least one **send-site** in `server/src/` (the server
//!   originates the message) AND at least one **drain-site** in
//!   `client/src/` (the client consumes it).
//!
//! When the invariant fails, the test panics with a list of every
//! violation, naming the message type, the declaration `file:line`, the
//! missing side, and a one-line remediation hint.
//!
//! Defect-class anchor: PROMPT 803 §3 DC-1 + DC-15 / §4 Lane A.
//! Pre-`S13-PROTO-ORPHAN-DRAIN-001` the test FAILS with the documented
//! orphan list captured in
//! `production/qa/evidence/sprint-13-proto-invariant-evidence.md`; the
//! `#[ignore]` attribute below is removed by `S13-PROTO-ORPHAN-DRAIN-001`
//! once the drains land.
//!
//! Detection patterns (Lightyear 0.26):
//! - **C2S send-site (client)**: `MessageSender<C2SX>` SystemParam.
//! - **C2S drain-site (server)**: `MessageReceiver<C2SX>` SystemParam.
//! - **S2C send-site (server)**: `MessageSender<S2CX>` SystemParam OR
//!   `send::<S2CX, _>` invocation on `ServerMultiMessageSender` (the
//!   canonical server-side broadcast pattern in this codebase).
//! - **S2C drain-site (client)**: `MessageReceiver<S2CX>` SystemParam.
//!
//! The test is source-text driven (`include_str!` for the manifest,
//! `std::fs` walk for the consumer trees) and does not construct a Bevy
//! `App` or call any Lightyear runtime API. ADR-002 binding: no
//! optimistic client-side authority is introduced or implied.

use std::fs;
use std::path::{Path, PathBuf};

const PROTOCOL_SOURCE: &str = include_str!("../../shared/src/protocol.rs");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    C2S,
    S2C,
}

#[derive(Debug)]
struct MessageDecl {
    name: String,
    direction: Direction,
    decl_line: usize,
}

fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("CARGO_MANIFEST_DIR must have a parent (workspace root)")
        .to_path_buf()
}

fn discover_registered_messages() -> Vec<MessageDecl> {
    let mut out = Vec::new();
    for line in PROTOCOL_SOURCE.lines() {
        let trimmed = line.trim_start();
        let (direction, name) = if let Some(rest) = trimmed.strip_prefix("register_c2s::<") {
            (Direction::C2S, rest)
        } else if let Some(rest) = trimmed.strip_prefix("register_s2c::<") {
            (Direction::S2C, rest)
        } else {
            continue;
        };
        if let Some(end) = name.find('>') {
            let type_name = name[..end].trim().to_string();
            if !type_name.is_empty() {
                let decl_line = find_decl_line(&type_name);
                out.push(MessageDecl {
                    name: type_name,
                    direction,
                    decl_line,
                });
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn find_decl_line(type_name: &str) -> usize {
    let prefix = format!("pub struct {}", type_name);
    for (i, line) in PROTOCOL_SOURCE.lines().enumerate() {
        if let Some(rest) = line.strip_prefix(prefix.as_str()) {
            if rest.is_empty()
                || rest.starts_with(' ')
                || rest.starts_with('{')
                || rest.starts_with(';')
                || rest.starts_with('<')
            {
                return i + 1;
            }
        }
    }
    0
}

fn collect_rs_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !root.exists() {
        return out;
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// Erase `#[cfg(test)] mod ... { ... }` blocks (matching brace-balanced) so
/// that references inside per-file unit-test modules do not satisfy the
/// production-side reference requirement.
fn strip_cfg_test_blocks(source: &str) -> String {
    let bytes = source.as_bytes();
    let marker = b"#[cfg(test)]";
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    while i < bytes.len() {
        if i + marker.len() <= bytes.len() && &bytes[i..i + marker.len()] == marker {
            if let Some(after) = consume_cfg_test_block(bytes, i + marker.len()) {
                i = after;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn consume_cfg_test_block(bytes: &[u8], start: usize) -> Option<usize> {
    let mut i = start;
    while i < bytes.len() && bytes[i] != b'{' {
        i += 1;
    }
    if i >= bytes.len() {
        return None;
    }
    let mut depth = 0i32;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn read_stripped(path: &Path) -> String {
    match fs::read_to_string(path) {
        Ok(text) => strip_cfg_test_blocks(&text),
        Err(_) => String::new(),
    }
}

fn count_substr(haystack: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    let mut count = 0;
    let mut rest = haystack;
    while let Some(pos) = rest.find(needle) {
        count += 1;
        rest = &rest[pos + needle.len()..];
    }
    count
}

/// Find `TYPE as ALIAS` aliases used inside `use ...` statements (or
/// anywhere else the canonical name is renamed). Required because
/// `server/src/network/economy_dispatch.rs` and
/// `server/src/feature/auction/system.rs` import protocol types with
/// `as ProtocolXxx` to disambiguate them from server-internal shadow
/// types of the same name. Without alias-tracking the test would flag
/// real send-sites as orphans.
fn extract_aliases_in(text: &str, canonical: &str) -> Vec<String> {
    let needle = format!("{} as ", canonical);
    let mut out: Vec<String> = Vec::new();
    let mut cursor = 0usize;
    while let Some(rel) = text[cursor..].find(&needle) {
        let pos = cursor + rel;
        if pos > 0 {
            let prev = text.as_bytes()[pos - 1];
            if prev.is_ascii_alphanumeric() || prev == b'_' {
                cursor = pos + needle.len();
                continue;
            }
        }
        let after = &text[pos + needle.len()..];
        let end = after
            .char_indices()
            .take_while(|(_, c)| c.is_alphanumeric() || *c == '_')
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        if end > 0 {
            let alias = after[..end].to_string();
            if !out.contains(&alias) {
                out.push(alias);
            }
        }
        cursor = pos + needle.len() + end.max(1);
    }
    out
}

fn names_for(canonical: &str, text: &str) -> Vec<String> {
    let mut names = vec![canonical.to_string()];
    names.extend(extract_aliases_in(text, canonical));
    names
}

fn has_send_site(files: &[(PathBuf, String)], type_name: &str) -> bool {
    files.iter().any(|(_, text)| {
        for name in names_for(type_name, text) {
            let param = format!("MessageSender<{}>", name);
            let call = format!("send::<{},", name);
            if count_substr(text, &param) > 0 || count_substr(text, &call) > 0 {
                return true;
            }
        }
        false
    })
}

fn has_drain_site(files: &[(PathBuf, String)], type_name: &str) -> bool {
    files.iter().any(|(_, text)| {
        for name in names_for(type_name, text) {
            let param = format!("MessageReceiver<{}>", name);
            if count_substr(text, &param) > 0 {
                return true;
            }
        }
        false
    })
}

#[test]
#[ignore = "S13-PROTO-ORPHAN-DRAIN-001 pending — pre-drain orphan list captured in production/qa/evidence/sprint-13-proto-invariant-evidence.md; remove this attribute in the drain-story commit"]
fn protocol_completeness_assert_send_and_drain_sites() {
    let root = workspace_root();
    let client_src = root.join("client").join("src");
    let server_src = root.join("server").join("src");

    let client_files: Vec<(PathBuf, String)> = collect_rs_files(&client_src)
        .into_iter()
        .map(|p| {
            let text = read_stripped(&p);
            (p, text)
        })
        .collect();
    let server_files: Vec<(PathBuf, String)> = collect_rs_files(&server_src)
        .into_iter()
        .map(|p| {
            let text = read_stripped(&p);
            (p, text)
        })
        .collect();

    assert!(
        !client_files.is_empty(),
        "client/src walk returned zero .rs files (workspace root = {})",
        root.display()
    );
    assert!(
        !server_files.is_empty(),
        "server/src walk returned zero .rs files (workspace root = {})",
        root.display()
    );

    let messages = discover_registered_messages();
    assert!(
        !messages.is_empty(),
        "protocol manifest discovery returned zero messages; \
         shared/src/protocol.rs parsing is broken"
    );

    let mut violations: Vec<String> = Vec::new();

    for msg in &messages {
        let decl = if msg.decl_line == 0 {
            String::from("shared/src/protocol.rs:?")
        } else {
            format!("shared/src/protocol.rs:{}", msg.decl_line)
        };

        match msg.direction {
            Direction::C2S => {
                if !has_send_site(&client_files, &msg.name) {
                    violations.push(format!(
                        "{name}  ({decl})\n    missing client-side send-site: \
                         add a `MessageSender<{name}>` SystemParam (or call \
                         `sender.send::<{name}, _>(...)`) under client/src/, \
                         or delete the type from the protocol with a rationale.",
                        name = msg.name,
                        decl = decl,
                    ));
                }
                if !has_drain_site(&server_files, &msg.name) {
                    violations.push(format!(
                        "{name}  ({decl})\n    missing server-side drain: add a \
                         `MessageReceiver<{name}>` SystemParam under server/src/, \
                         or delete the type from the protocol with a rationale.",
                        name = msg.name,
                        decl = decl,
                    ));
                }
            }
            Direction::S2C => {
                if !has_send_site(&server_files, &msg.name) {
                    violations.push(format!(
                        "{name}  ({decl})\n    missing server-side send-site: \
                         add a `MessageSender<{name}>` SystemParam or call \
                         `sender.send::<{name}, _>(...)` on a \
                         `ServerMultiMessageSender` under server/src/, or delete \
                         the type from the protocol with a rationale.",
                        name = msg.name,
                        decl = decl,
                    ));
                }
                if !has_drain_site(&client_files, &msg.name) {
                    violations.push(format!(
                        "{name}  ({decl})\n    missing client-side drain: add a \
                         `MessageReceiver<{name}>` SystemParam under client/src/, \
                         or delete the type from the protocol with a rationale.",
                        name = msg.name,
                        decl = decl,
                    ));
                }
            }
        }
    }

    if !violations.is_empty() {
        let mut report = String::new();
        report.push_str(&format!(
            "Protocol completeness invariant violated: {n} violation(s) \
             across {m} registered message type(s).\n\n",
            n = violations.len(),
            m = messages.len(),
        ));
        report.push_str(
            "Each entry: <MessageType>  (declaration:line)\n    \
             missing-side: actionable remediation hint.\n",
        );
        report.push_str(
            "Cross-link: \
             reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md \
             §3 DC-1, §3 DC-15, §4 Lane A.\n\n",
        );
        for v in &violations {
            report.push_str(v);
            report.push('\n');
            report.push('\n');
        }
        panic!("{}", report);
    }
}

/// Smoke check: the manifest parser must see every `register_c2s::<...>` /
/// `register_s2c::<...>` line in `shared/src/protocol.rs`. The exact count
/// is asserted as a lower bound (>= 16 C2S + >= 34 S2C as of `origin/main`
/// at story authoring) to catch a parser regression that would silently
/// underreport the inventory.
#[test]
fn protocol_manifest_parser_discovers_registered_messages() {
    let messages = discover_registered_messages();
    let c2s = messages
        .iter()
        .filter(|m| matches!(m.direction, Direction::C2S))
        .count();
    let s2c = messages
        .iter()
        .filter(|m| matches!(m.direction, Direction::S2C))
        .count();
    assert!(
        c2s >= 16,
        "expected >=16 C2S messages from register_c2s::<...> scan, found {c2s}"
    );
    assert!(
        s2c >= 34,
        "expected >=34 S2C messages from register_s2c::<...> scan, found {s2c}"
    );

    for msg in &messages {
        assert!(
            msg.decl_line > 0,
            "registered message `{}` has no `pub struct` declaration in \
             shared/src/protocol.rs (parser regression?)",
            msg.name
        );
    }
}
