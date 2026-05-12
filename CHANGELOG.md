# Changelog

## [Unreleased]

### Added — first cut (v0.1.0)

- Six read-only commands: `list`, `orphans`, `duplicates`, `size`, `doctor`, `ai`.
- Parallel scanners for Homebrew (formula + cask), npm globals, bun globals, `cargo install`, rustup toolchains, `go install`, and gh extensions.
- PATH-orphan detector with symlink resolution, app-bundle classification, alias dedup, and shadow detection.
- Exclusive per-tool disk sizing via `jwalk` with on-disk size cache keyed by (path, mtime).
- `--json` machine-readable output on every command.
- Hardcoded AI-tool name list (claude, codex, cursor-agent, ollama, aider, llm, gemini, openai, anthropic, mods, fabric, cody, tgpt, butterfish, exa, perplexity).
- macOS-only for now. Dual MIT / Apache-2.0 licensed.
