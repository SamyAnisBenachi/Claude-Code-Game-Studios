$active = @("PAW-001", "lightyear-replication-sender-fix")
Get-ChildItem "D:\_DEV\claude-code-game-studios-worktrees" -Directory |
  Where-Object { $active -notcontains $_.Name } |
  ForEach-Object {
    $t = Join-Path $_.FullName "target"
    if (Test-Path $t) {
      Write-Host "Cleaning $($_.Name)..."
      Remove-Item $t -Recurse -Force
    }
  }
Write-Host "Done."
