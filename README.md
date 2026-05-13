# hearth

> A unified inventory of every CLI tool installed on your Mac — across Homebrew, npm, bun, cargo, rustup, go, and `curl | sh` installs. See sizes. Find orphans. Spot duplicates.

```
$ hearth orphans
 name       | version | source |       size | path
------------+---------+--------+------------+-------------------------------------
 agent      |         | manual | 159.67 MiB | /Users/you/.local/bin/agent
 brew       |         | manual |   8.47 KiB | /opt/homebrew/bin/brew
 mole-admin |         | manual |   2.28 KiB | /usr/local/bin/mole-admin
 tailscale  |         | manual |       68 B | /usr/local/bin/tailscale
```

The wave of AI CLIs — claude, cursor-agent, codex, ollama, aider, llm, gemini, openai — has made `curl ... | sh` installers normal again. They drop binaries in `~/.local/bin` and `/usr/local/bin` that no package manager tracks. Combined with `brew`, `npm -g`, `bun -g`, `cargo install`, `go install`, and rustup toolchains, the picture of what's actually installed on your Mac has become impossible to see.

`hearth` gives you that picture in one command.

## Quick start

```bash
# Homebrew tap (recommended)
brew install cryt3rion/hearth/hearth

# Cargo
cargo install hearth-cli

# Curl (yes, on purpose — your hearth install will show up in `hearth orphans`)
curl -fsSL https://hearth.sh/install | sh
```

```bash
hearth                       # default: list everything
hearth orphans               # binaries no package manager claims (manual / curl|sh)
hearth size --top 20         # what's eating your disk
hearth duplicates            # same tool from multiple sources
hearth ai                    # curated AI tool view
hearth doctor                # broken symlinks, shadowed binaries, missing PATH entries
```

All commands support `--json` for piping into `jq`.

## What hearth knows about

v0.1.0 scanners:

| Source                | What's read |
|-----------------------|-------------|
| Homebrew (formula)    | `brew info --json=v2 --installed` |
| Homebrew (cask)       | same call, `casks` field |
| npm global            | `npm ls -g --depth=0 --json` |
| bun global            | `~/.bun/install/global/package.json` |
| cargo install         | `~/.cargo/.crates2.json` |
| rustup toolchains     | `~/.rustup/toolchains/*` (one entry per toolchain) |
| go install            | `${GOBIN:-$GOPATH/bin:-~/go/bin}` |
| gh extensions         | `~/.local/share/gh/extensions` |
| **PATH orphans**      | every executable in `$PATH` not claimed above |
| **`.app` bundles**    | manual binaries that resolve into `/Applications/*.app/` |

The orphan / `.app` detection is the differentiator. It walks `$PATH`, resolves symlinks, attributes binaries to their owning manager when possible, and surfaces everything else as `manual` or `app` so you can see exactly what `curl | sh` left behind.

## Sample output

```
$ hearth size --top 10
 name                               | version | source |       size | path
------------------------------------+---------+--------+------------+-----------------------------------
 rustup:stable-aarch64-apple-darwin | stable  | rustup |   1.14 GiB | ~/.cargo/bin/rustup
 windsurf                           |         | app    | 856.01 MiB | /opt/homebrew/bin/windsurf
 cursor                             |         | app    | 707.80 MiB | /opt/homebrew/bin/cursor
 ollama                             |         | app    | 520.10 MiB | /usr/local/bin/ollama
 happy                              | 1.1.7   | npm    | 499.58 MiB | /opt/homebrew/bin/happy
 Claude Code                        | 2.1.126 | cask   | 206.24 MiB | /opt/homebrew/Caskroom/claude-code
 Codex                              | 0.130.0 | cask   | 183.71 MiB | /opt/homebrew/Caskroom/codex
 ...
total: 7.40 GiB
```

```
$ hearth ai
 name        | version | source |       size | path
-------------+---------+--------+------------+----------------------------------------
 Claude      | 1.1.86… | cask   |            | /opt/homebrew/Caskroom/claude/…
 Claude Code | 2.1.126 | cask   | 206.24 MiB | /opt/homebrew/Caskroom/claude-code/…
 Codex       | 0.130.0 | cask   | 183.71 MiB | /opt/homebrew/Caskroom/codex/…
 ollama      |         | app    | 520.10 MiB | /usr/local/bin/ollama
```

```
$ hearth duplicates
 name       | version | source |      size | path
------------+---------+--------+-----------+------------------------------
 create-dmg | 1.2.3   | brew   | 70.60 KiB | /opt/homebrew/bin/create-dmg
 create-dmg | 8.1.0   | npm    |  7.20 MiB | /opt/homebrew/bin/create-dmg
```

## How is this different from `mpm` / `topgrade` / `mise` / `pkgx`?

| Tool | Cross-manager listing | Per-tool disk usage | Detects manual / `curl\|sh` installs | Detects `.app`-bundle binaries | Modern single-binary |
|---|---|---|---|---|---|
| **hearth**             | ✓ | ✓ | ✓ | ✓ | ✓ (Rust) |
| [meta-package-manager](https://github.com/kdeldycke/meta-package-manager) | ✓ (47+ managers, more than hearth today) | ✗ | ✗ | ✗ | ✗ (Python) |
| [topgrade](https://github.com/topgrade-rs/topgrade)               | upgrade only, no inventory | ✗ | ✗ | ✗ | ✓ |
| [mise](https://mise.jdx.dev/) / asdf / volta / pkgx | version managers, not inventory | ✗ | ✗ | ✗ | varies |
| [dua-cli](https://github.com/Byron/dua-cli) / gdu / ncdu | not package-aware | ✓ | ✗ | ✗ | ✓ |

`hearth` is deliberately scoped: it does inventory and disk visibility very well. It does not install, update, or uninstall — `brew`, `cargo`, `npm`, and `topgrade` already do that. If you want to upgrade everything across managers, run `topgrade` after `hearth list` shows you what's there.

## v0.1.0 limitations (documented, not surprises)

- **macOS only.** Linux support is on the roadmap.
- **No shell function / alias detection.** `hearth` only sees executables on disk in `$PATH`. Things you `alias` in your shellrc don't appear.
- **No fingerprinting yet.** A `~/.local/bin/agent` binary appears as `agent`; `hearth` currently won't tell you "this is cursor-agent". v0.2 ships a curated registry that resolves this.
- **No `pip` / `pipx` scanners.** Python globals are a swamp; coming in v0.2.
- **No `.app` size counting for casks by default.** A cask like `claude` (the desktop app) shows a near-zero Caskroom size because the actual `.app` lives in `/Applications`. Pass `--include-app-bundles` once that lands in v0.2.
- **Shell script "launchers" are not followed.** `/usr/local/bin/tailscale` is a tiny shell script that execs the real binary in `/Applications/Tailscale.app/`; `hearth` correctly reports the script's size but does not yet attribute it to the `.app`.

## Roadmap

- **v0.2** — curated AI/dev tool registry with fingerprinting; TUI via [ratatui](https://github.com/ratatui/ratatui); `pip`/`pipx`/`mas`/`mise`/`pnpm`/`yarn` scanners; `--include-app-bundles` flag; `hearth update <tool>` / `hearth remove <tool>` that delegate to the right manager.
- **v0.3** — Linux support; remote registry sync; `self-update`.

## Contributing

Each scanner lives in `src/scan/<name>.rs` and is ~100-150 LOC. Adding a new one (pnpm, yarn, mas, pipx, …) is a great first PR — copy `src/scan/go.rs` as the simplest template.

The orphan/PATH scanner lives in `src/scan/path_scan.rs`; that's the heart of the project.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
