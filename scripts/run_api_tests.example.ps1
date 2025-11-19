<#
Example API test script (safe to commit)

This example shows how to run the real `run_api_tests.ps1` script in CI or locally
without embedding secrets. Copy this to `run_api_tests.ps1` locally if you want
to customize for a specific machine, or set environment variables in CI.

Usage (locally):
  # set env vars in PowerShell and run
  $env:USER_EMAIL='testuser@example.com'; $env:USER_PASSWORD='Password123!'; pwsh -ExecutionPolicy Bypass -File .\scripts\run_api_tests.ps1

  # or pass only BaseUrl and let env provide credentials
  pwsh -ExecutionPolicy Bypass -File .\scripts\run_api_tests.ps1 -BaseUrl "http://127.0.0.1:3001"

Notes:
- This example is safe to commit. Do NOT store real credentials here.
- In CI, set the following GitHub Secrets and map them to env vars in the workflow:
  `ADMIN_EMAIL`, `ADMIN_PASSWORD`, `USER_EMAIL`, `USER_PASSWORD`, `MIDTRANS_SERVER_KEY`, `MIDTRANS_CLIENT_KEY`.
#>

param(
    [string]$BaseUrl = "http://127.0.0.1:3001"
)

Write-Host "[example] Base URL: $BaseUrl"

# Resolve credentials from environment variables - no defaults with real secrets
$UserEmail = $env:USER_EMAIL
$UserPassword = $env:USER_PASSWORD
$UserName = if ($env:USER_NAME) { $env:USER_NAME } else { 'Test User' }
$AdminEmail = $env:ADMIN_EMAIL
$AdminPassword = $env:ADMIN_PASSWORD

# Optional Midtrans keys (CI should set these as secrets)
$MidtransServerKey = $env:MIDTRANS_SERVER_KEY
$MidtransClientKey = $env:MIDTRANS_CLIENT_KEY

function Show-EnvNote {
    if (-not $UserEmail -or -not $UserPassword) {
        Write-Host "[example] WARNING: USER_EMAIL / USER_PASSWORD not set in environment. Local test user registration may fail or use placeholders."
    } else {
        Write-Host "[example] Using USER_EMAIL from environment: $UserEmail"
    }
    if (-not $AdminEmail -or -not $AdminPassword) {
        Write-Host "[example] Admin credentials not set (admin flows will be skipped unless provided)."
    } else {
        Write-Host "[example] ADMIN_EMAIL provided via environment. Admin flows will run."
    }
}

Show-EnvNote

Write-Host "[example] This is a committed example. To run actual tests, set environment variables or copy to 'run_api_tests.ps1' and edit locally."

# End of example script. The real script `run_api_tests.ps1` contains the full test implementation.
