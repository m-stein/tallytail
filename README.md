# Asset Allocation Tracker

A simple app to track your asset allocation.

## Requirements

- Rust and cargo (https://rust-lang.org/, tested with rustc 1.94.1)
- Clang (https://github.com/llvm/llvm-project, tested with clang 22.1.5)
- Trunk (https://crates.io/crates/trunk, tested with trunk 0.21.14)

## Running the app from source

### Desktop target

```powershell
cd dekstop_app
cargo run --release
```

### Web target

```powershell
cd web_back_end
cargo run --release
```

```powershell
cd web_front_end
trunk serve
```

Open front end URL in browser