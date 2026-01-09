# Agent Guide for atosl-rs

This guide is intended for AI agents (Codex, bots, contributors) working on the `atosl-rs` repository. It provides an overview of the project structure, development workflows, and coding standards.

## 1. Project Overview
`atosl-rs` is a Rust-based tool designed to act as a partial replacement for Apple's `atos` tool on Linux. It converts memory addresses within a binary file to symbols (function names, file paths, line numbers), supporting both DWARF and Mach-O formats.

## 2. Repository Map

The project follows a standard Rust binary structure:

- **`src/`**: Source code directory.
  - **`main.rs`**: The entry point of the application. Handles CLI argument parsing using `clap` and orchestrates the execution.
  - **`atosl.rs`**: Contains the core logic for address-to-symbol translation using the `gimli` and `object` crates.
  - **`demangle.rs`**: Handles symbol demangling logic to make function names readable.
- **`Cargo.toml`**: Project configuration, metadata, and dependencies (including `gimli`, `object`, `clap`, `symbolic-common`).
- **`README.md`**: User-facing documentation.

## 3. How to Run Locally

You can run the tool directly using `cargo`.

```bash
# General syntax
cargo run -- -o <OBJECT_PATH> -l <LOAD_ADDRESS> [ADDRESSES]...

# Example
cargo run -- -l 0x1000 -o ./path/to/binary 0x1050 0x2000
```

## 4. How to Test

Run the standard Rust test suite:

```bash
cargo test
```

*Note: If specific test files or integration tests are added in the future, check the `tests/` directory.*

## 5. Linting and Formatting

Ensure code quality and consistency using standard Rust tools:

- **Format code**:
  ```bash
  cargo fmt
  ```

- **Lint code**:
  ```bash
  cargo clippy -- -D warnings
  ```

## 6. Build and Release

To build the project for release (optimized binary):

```bash
cargo build --release
```

The resulting binary will be located at `target/release/atosl`.

## 7. Coding Style & Conventions

- **Language**: Rust (2021 edition).
- **Formatting**: Follow standard `rustfmt` rules.
- **Error Handling**: Use `anyhow::Result` for error propagation in the application layer.
- **CLI**: Use `clap` (v3+ features) for argument parsing.

## 8. Debugging

- Use `println!` or `eprintln!` for simple logging.
- Set the environment variable `RUST_BACKTRACE=1` to see full stack traces on panic.
- If verbose output is implemented, use the `-v` flag when running the tool.

## 9. Rules for Making Changes

1.  **Small PRs**: Keep changes focused on a single issue or feature.
2.  **Update Documentation**: If you change CLI arguments or behavior, update `README.md`.
3.  **Verify**: Always run `cargo check` and `cargo test` before submitting changes.
4.  **No Unrelated Refactors**: Avoid changing code style or logic outside the scope of your task.

## 10. PR Checklist

- [ ] Code compiles without errors (`cargo check`).
- [ ] Code is formatted (`cargo fmt`).
- [ ] Lints pass (`cargo clippy`).
- [ ] Tests pass (`cargo test`).
- [ ] Documentation updated (if applicable).
