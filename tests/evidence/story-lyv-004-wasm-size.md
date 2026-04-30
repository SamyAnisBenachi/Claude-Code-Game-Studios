# Story LYV-004 WASM Size Evidence

Date: 2026-04-30

## Command

```powershell
C:\Users\Sam\.cargo\bin\cargo.exe build -p client --target wasm32-unknown-unknown --release
```

## Result

PASS. The release WASM client built successfully.

## Artifact

```text
D:\_DEV\claude-code-game-studios\target\msvc-local\wasm32-unknown-unknown\release\client.wasm
```

Measured size: `20,843,568` bytes (`19.88` MiB).

Budget: `50,000,000` bytes.

Verdict: PASS. The raw cargo WASM artifact is under the story budget.
