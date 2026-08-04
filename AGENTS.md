# Repository Guidelines

`atosl` is a Rust CLI and library for local symbolication across Mach-O and other object formats. Correct address translation and deterministic diagnostics are more important than best-effort guesses.

## Structure

- `src/`: CLI, object loading, DWARF lookup, demangling, and output formatting.
- `tests/`: integration, fixtures, golden output, and Apple `atos` differential coverage.
- `benches/`: batch-symbolication benchmarks.
- `scripts/`: golden refresh and release helpers.
- `docs/`: published documentation source.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo bench --bench batch_symbolize --no-run
```

On macOS, run `./scripts/refresh_apple_goldens.sh` only when the expected output intentionally changes.

## Conventions

- Keep CLI output stable and script-friendly; treat JSON fields as a compatibility surface.
- Validate architecture, UUID, load address, and slice selection explicitly.
- Avoid loading entire large binaries when memory mapping or streaming is sufficient.
- Add malformed-input and ambiguous-slice tests with every parser hardening change.
- Do not weaken comparisons with Apple `atos` to make a regression pass.

Update `README.md`, `README.zh-CN.md`, and `RELEASING.md` with behavior or workflow changes. Canonical repository: `https://github.com/everettjf/atosl-rs`.
