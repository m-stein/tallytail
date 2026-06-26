# tallytail

tallytail is a personal asset and investment management app. It helps track
asset allocations, transactions, and portfolio positions.

## Requirements

- Rust and cargo (https://rust-lang.org/, tested with rustc 1.94.1)
- Clang (https://github.com/llvm/llvm-project, tested with clang 22.1.5)
- Trunk (https://crates.io/crates/trunk, tested with trunk 0.21.14)

## Running the app from source

### Desktop target

```powershell
cd desktop_app
cargo run
```

### Web target

```powershell
cd web_back_end
cargo run
```

```powershell
cd web_front_end
trunk serve
```

Then, open the front end URL in a browser.

## Preparing changes for a commit

```powershell
./precommit.ps1
```

This script should be run from the repository root and must succeed before each
commit.