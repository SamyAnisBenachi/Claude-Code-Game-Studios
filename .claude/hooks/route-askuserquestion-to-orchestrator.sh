#!/usr/bin/env bash
# Route Claude Code AskUserQuestion calls to the gcs-app orchestrator.
# Auto-installed in this project (CCGS) per user request 2026-05-18.
# Original template: D:/_DEV/Work/gcs-app/docs/examples/route-askuserquestion-to-orchestrator.sh
#
# Active when CLAUDE_ROUTE_ASKUSERQUESTION_TO_ORCHESTRATOR=true is in the
# session env. The env var is set via .claude/settings.json `env` block, so
# every Claude Code session in this project picks it up automatically.

set -uo pipefail

[ "${CLAUDE_ROUTE_ASKUSERQUESTION_TO_ORCHESTRATOR:-false}" = "true" ] || exit 0

PAYLOAD="$(cat)"
WORKER_ID="${CLAUDE_WORKER_TERMINAL_ID:-unknown}"
PROJECT_ID="${CLAUDE_PROJECT_ID:-default}"
SIDECAR="${GCS_APP_SIDECAR_URL:-http://127.0.0.1:9788}"

QUESTIONS_JSON="$(python -c '
import json, sys
p = json.loads(sys.stdin.read())
print(json.dumps((p.get("tool_input") or {}).get("questions") or []))
' <<<"$PAYLOAD" 2>/dev/null)"

[ -z "$QUESTIONS_JSON" ] || [ "$QUESTIONS_JSON" = "[]" ] && exit 0

REQ="$(python -c '
import json, sys
print(json.dumps({
    "worker_id": sys.argv[1],
    "project_id": sys.argv[2],
    "questions": json.loads(sys.argv[3]),
    "timeout_s": 60,
}))
' "$WORKER_ID" "$PROJECT_ID" "$QUESTIONS_JSON")"

RESP="$(timeout 65s curl -s --max-time 65 -X POST "$SIDECAR/api/orchestrator/decide-question" \
    -H 'Content-Type: application/json' \
    -d "$REQ" 2>/dev/null || echo "")"

[ -z "$RESP" ] && exit 0

ANSWER="$(python -c '
import json, sys
try:
    d = json.loads(sys.argv[1])
    if d.get("fallthrough"):
        sys.exit(0)
    answers = json.dumps(d.get("answers", {}), indent=2)
    print(json.dumps({
        "decision": "block",
        "reason": (
            "[orchestrator-auto-decided]\n"
            + answers
            + "\nProceed with these answers; do not re-ask the human."
        ),
    }))
except Exception:
    sys.exit(0)
' "$RESP" 2>/dev/null)"

[ -n "$ANSWER" ] && echo "$ANSWER"
