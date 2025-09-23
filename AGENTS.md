# Repository Guidelines

## Project Structure & Module Organization
Each `day_xx` directory (e.g. `day_2`, `day_3`, `day_4`) is an independent Rust crate representing one day of the Bevy rogue-lite challenge. Place source files in `<day_xx>/src`; organize gameplay code into `components/`, `systems/`, and `plugins/` as day_4 does, and keep shared constants in `constants.rs`. Store art and audio in `<day_xx>/assets`. The `target/` directory is Cargo build output and should stay untracked. Update the README if you introduce a new day.

## Build, Test, and Development Commands
Run a crate locally with `cd day_04 && cargo run` (substitute the day you are working on). Use `cargo check` during development to validate compilation without running the game. Format code via `cargo fmt`, and lint with `cargo clippy -- -D warnings`. Regenerate assets or shaders before commits so runtime paths remain valid.

## Coding Style & Naming Conventions
Follow idiomatic Rust style: four-space indentation, `snake_case` for files, modules, and functions, `CamelCase` for structs/enums, and `SCREAMING_SNAKE_CASE` for constants. Group system functions by gameplay feature inside `systems/` and expose them through minimal `mod.rs` files. Run `cargo fmt` before committing; avoid hand-editing generated files.

## Testing Guidelines
Use Rust’s built-in test harness. Place unit tests in the same module under a `#[cfg(test)]` block, and integration tests in `<day_xx>/tests/` if they grow larger. Prefer stateful logic tests over rendering snapshots. Execute `cargo test` from the relevant day directory; target at least one regression test when fixing bugs.

## Commit & Pull Request Guidelines
The history favors Conventional Commits (`feat:`, `docs:`). Keep subject lines under 72 characters and write in the imperative mood. Pull requests should describe the gameplay change, list manual or automated test commands (e.g. `cargo test`), and attach screenshots or clips when visuals change. Reference related issues or journal entries so daily progress stays traceable.

## Bevy & Asset Tips
Bevy hot-reloads assets when `CARGO_WATCH` is running; consider `cargo install cargo-watch` locally and run `cargo watch -x run` for rapid iteration (do not commit the binary). When adding textures, keep paths relative to the crate root and document any licensing notes alongside the asset.
