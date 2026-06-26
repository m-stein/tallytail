# AGENTS.md

Guidance for coding agents working in this repository.

## Project Direction

Tallytail is a personal asset and investment management app. It helps track
asset allocations, transactions, and portfolio positions.

The codebase has two application variants:

- `desktop_app`: native desktop app.
- `web_back_end` plus `web_front_end`: web app with an HTTP backend and WASM
  frontend.

Both variants share most behavior through `core_lib`, `infra_lib`, and
`ui_lib`.

## Repository Map

- `core_lib`: shared domain types, DTOs, enums, and the request contract.
- `core_lib/src/request_list.rs`: the canonical frontend/backend interface.
- `infra_lib`: persistence and business logic. It currently uses SQLite files
  and allocation record files under `data/`.
- `ui_lib`: shared egui/eframe UI and the `AppBackend` trait used by both app
  variants.
- `desktop_app`: desktop entry point and backend adapter that calls `infra_lib`
  on background threads.
- `web_back_end`: Axum backend that exposes every request as a `POST` route and
  calls `infra_lib`.
- `web_front_end`: WASM frontend that implements `AppBackend` by posting to
  `web_back_end`.
- `data`: local application data. Treat this as user/project state, not scratch
  output.
- `img`: static UI assets.

## Architecture Rules

The request list in `core_lib/src/request_list.rs` is the central contract
between UI and backend. Many layers are generated from it via macros:

- `ui_lib/src/app_backend.rs` defines the `AppBackend` trait.
- `ui_lib/src/eframe_app.rs` stores request receivers and polls results.
- `desktop_app/src/desktop_backend.rs` maps requests to `infra_lib` calls.
- `web_back_end/src/main.rs` exposes request routes.
- `web_front_end/src/web_backend.rs` posts requests to the web backend.

When adding or changing an app operation:

1. Add or update shared input/output types in `core_lib`.
2. Add the request signature in `core_lib/src/request_list.rs`.
3. Implement the matching function in `infra_lib` with the exact request name.
4. Update `ui_lib` to start, poll, and render the operation as needed.
5. Verify both desktop and web targets still compile.

Prefer shared UI and domain code. Only put code in `desktop_app`,
`web_back_end`, or `web_front_end` when it is truly variant-specific.

## Data And Persistence

`infra_lib` currently stores:

- allocation records as `.ron` files in `data/allocation_records`;
- asset/category data in `data/assets.sdb`;
- transactions and portfolio state in `data/transactions.sdb`.

Be careful with `data/`. Do not delete, reset, or overwrite these files unless
the user explicitly asks for that. Tests or experiments that mutate persistent
data should use temporary files or isolated fixtures where possible.

Use precise decimal types for transaction and portfolio quantities/values.
Avoid floating point for money-like transaction calculations.

## Development Commands

Use PowerShell commands from the repository root unless noted otherwise.

Run the desktop app:

```powershell
cd desktop_app
cargo run
```

Run the web backend:

```powershell
cd web_back_end
cargo run
```

Run the web frontend:

```powershell
cd web_front_end
trunk serve
```

Prepare changes for commit:

```powershell
./precommit.ps1
```

This is the required project check script before each commit. It should be run
from the repository root and must succeed before committing changes.

Useful targeted checks:

```powershell
cargo fmt
cargo test -p infra_lib
cargo clippy -p desktop_app --target x86_64-pc-windows-msvc
cargo clippy -p web_back_end --target x86_64-pc-windows-msvc
cargo clippy -p web_front_end --target wasm32-unknown-unknown
```

Note: If `precommit.ps1` fails, report the failing command and the relevant
diagnostic.

## Coding Conventions

- Follow the existing Rust style and keep changes narrowly scoped.
- Use `eyre::Result` consistently where the surrounding code does.
- Keep shared DTOs serializable with `serde` when they cross the request
  boundary.
- Keep request argument and return types owned and serialization-friendly.
- Preserve the macro-based request plumbing instead of hand-writing duplicate
  methods or routes.
- Add focused tests in `infra_lib` for business rules and persistence behavior.
- Do not introduce a new frontend framework; the shared UI is egui/eframe.

## Product Language

Use **Tallytail** as the product name. For titles use the longer form
**Tallytail Asset Manager**. When touching visible text, keep labels
concise and workflow-focused.
