# Dev Environment Setup

> Story: `S11-OPS-GH-CLI-001` (Sprint 13 Nice to Have; doc-only)
> Source: `production/epics/devops/story-004-gh-cli-setup.md`
> Authored: 2026-05-14 (PROMPT 873)
> Path option chosen: **(a)** -- new file at `docs/setup/dev-environment.md`.
> `docs/setup/` already exists on `origin/main` (created by Story 017 /
> PROMPT 858 for `two-client-runtime-harness.md`), so only the file is
> new under this story; the directory is not.

This document lists the required developer tooling for working on the
Claude Code Game Studios repo. It is **onboarding documentation only**.
Nothing in this file installs or configures tooling automatically.

## Required tooling

### GitHub CLI (`gh`)

The GitHub CLI (`gh`) is **required** for orchestrator and producer
workflows in this repo. Worker and integration prompts use `gh` for:

- Pull-request creation, review-comment fetching, and merge
  inspection without leaving the terminal.
- Issue triage and labelling when bug rows are surfaced from the
  CLI rather than the GitHub web UI.
- Token-scoped API access (`gh api ...`) for read-only checks
  against branch protection, workflow runs, and release artifacts.

During Sprint 11 Wave 12 the GitHub CLI was logged as absent from
the dev machine 3+ times during orchestrator workflows, forcing
worker fallback to the GitHub web UI. This note exists to prevent
that recurrence by naming `gh` as required onboarding tooling.

#### Install commands

Pick the command for your platform. Run it once per machine; the CLI
self-updates from there.

**Windows (primary supported dev platform)** -- via `winget`:

```pwsh
winget install --id GitHub.cli
```

**macOS** -- via Homebrew:

```bash
brew install gh
```

**Linux (Debian / Ubuntu)** -- via the official apt repo (see
<https://cli.github.com/> for the current key + repo line; package
managers on other distros also ship `gh`):

```bash
sudo apt update
sudo apt install gh
```

#### One-time authentication (optional but recommended)

After install, authenticate once per machine:

```bash
gh auth login
```

Choose **GitHub.com**, then **HTTPS**, then **Login with a web
browser**. The default token scope (`repo`, `read:org`, `gist`,
`workflow`) is sufficient for the orchestrator workflows above; no
extra scopes are required by this story.

Verify with:

```bash
gh auth status
gh --version
```

## Out of scope for this story

This story is **documentation only**. The following are explicitly **not**
performed by the story commit:

- Installing `gh` on any machine.
- Adding `gh` to any CI workflow or build script
  (`.github/workflows/**`, `*.sh`, `*.ps1`).
- Recommending or adding any other tooling beyond `gh`.
- Activating or closing any Sprint 13 row.

See also: `production/epics/devops/story-005-win-appcompat-note.md`
(Sprint 13 Nice to Have `S13-OPS-WIN-APPCOMPAT-NOTE-001`) for the
sibling Windows App Compatibility note. That note is authored
separately and is not modified by this story.

---

## Status / No-Claim Banner (verbatim restatement, per AC5)

This story is authored as a Sprint 13 candidate. Sprint 13 is **NOT**
activated by PROMPT 819. Sprint 12 is closed-with-conditions per PROMPT
817 and is not changed by this authoring run.

PROMPT 819 (this authoring run) does NOT:

- Activate Sprint 13.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-13.md` or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check
  artifact.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this
  story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Install or change `gh` or any other tool.
- Retry the PROMPT 761 Polish->Release gate-check.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

Sprint 10 / Sprint 11 / Sprint 12 dispositions unchanged. PROMPT 761
Polish->Release gate-check FAIL evidence preserved.

**This story is documentation only. NO TOOLING CHANGES LAND.**

---

## Implementation note for PROMPT 873

The implementing prompt (PROMPT 873, `/dev-story` doc-only) chose
option **(a)** from the story's Prevention target: create
`docs/setup/dev-environment.md` as the canonical onboarding doc rather
than amending an existing sibling. Rationale:

- `docs/setup/` already exists on `origin/main@3cf5e41` (created by
  Story 017 / PROMPT 858 for `two-client-runtime-harness.md`), so
  option (a) only adds a new file under the established convention.
- The sibling story 005 (`S13-OPS-WIN-APPCOMPAT-NOTE-001`) is also
  expected to land in `docs/setup/dev-environment.md` per the story's
  Dependency Notes. Establishing the file now gives that future story
  an unambiguous target and avoids same-file conflict risk between
  the two paragraph-scale edits.
- No canonical sibling onboarding doc (e.g. `CONTRIBUTING.md`,
  `docs/onboarding.md`) currently exists on `origin/main`, so option
  (b) would require choosing a less-canonical host such as
  `docs/WORKFLOW-GUIDE.md` or `docs/octogent-integration.md`; both
  are not onboarding docs by purpose and would muddy their scope.

The single doc-only commit under this story touches **only** this
file. No code, CI, build script, or sprint-tracker artifact is
modified.
