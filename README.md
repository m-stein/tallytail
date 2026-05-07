# Asset Allocation Tracker

A simple app to track your asset allocation.

## Requirements

- Rust and cargo (tested with rustc 1.94.1)

## Running the app from source

### Desktop target

```powershell
cargo run --release
```

### Web target (WASM)

```powershell
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve
```