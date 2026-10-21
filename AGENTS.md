# AGENTS.md

## Working rules

- **Code style** - Read `docs/CODE_STYLE.md` for code guidelines
- **Ask user if uncertain.** If something is unclear, difficult, or needs more context, ask the user.
- **Verify changes.** After each edit, check first if compliant with `docs/CODE_STYLE.md` then run `mise run build`. It runs `cargo fmt`, `cargo build`, `cargo clippy --all-targets` (without and with `--all-features`) `-- -D warnings`, and `cargo test --features simulation,schema` with the Rust toolchain that `mise.toml` pins. Do not run plain `cargo`, because it can use a different toolchain.
- **Comments only if necessary.** Make the code clear through names and signatures first. Write a comment only when the code cannot show the intent, the reason, an invariant, or a unit. See `docs/CODE_STYLE.md` "Comments".
