$commands = @(
    "cargo fmt",
    "cargo clippy -p web_front_end --target wasm32-unknown-unknown",
    "cargo clippy -p desktop_app --target x86_64-pc-windows-msvc",
    "cargo clippy -p web_back_end --target x86_64-pc-windows-msvc"
)

foreach ($cmd in $commands) {
    Write-Host "$cmd"
    Invoke-Expression $cmd
    if ($LASTEXITCODE -ne 0) {
        Write-Host "error: command failed: $cmd"
        exit $LASTEXITCODE
    }
}