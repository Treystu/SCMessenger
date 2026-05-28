# Install Git hooks for SCMessenger (PowerShell version)

$ErrorActionPreference = "Stop"

Write-Host "Installing Git hooks..." -ForegroundColor Cyan

# Check if we're in a git repository
if (-not (Test-Path ".git")) {
    Write-Host "Error: Not in a git repository" -ForegroundColor Red
    exit 1
}

# Create hooks directory if it doesn't exist
New-Item -ItemType Directory -Force -Path ".git/hooks" | Out-Null

# Install pre-commit hook
if (Test-Path "scripts/pre-commit") {
    Copy-Item "scripts/pre-commit" ".git/hooks/pre-commit" -Force
    Write-Host "✓ Installed pre-commit hook" -ForegroundColor Green
} else {
    Write-Host "⚠ scripts/pre-commit not found" -ForegroundColor Yellow
}

# Install commit-msg hook
if (Test-Path "scripts/commit-msg") {
    Copy-Item "scripts/commit-msg" ".git/hooks/commit-msg" -Force
    Write-Host "✓ Installed commit-msg hook" -ForegroundColor Green
} else {
    Write-Host "⚠ scripts/commit-msg not found" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Git hooks installed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "The following checks will run before each commit:"
Write-Host "  • Rust formatting (cargo fmt)"
Write-Host "  • Clippy linting (cargo clippy)"
Write-Host "  • Unit tests (cargo test --lib --bins)"
Write-Host "  • No unwrap() in library code"
Write-Host "  • No println! in library code"
Write-Host "  • Conventional commit message format"
Write-Host ""
Write-Host "To skip hooks (not recommended), use: git commit --no-verify"
