# raytha CLI

A command-line interface for [Raytha](https://github.com/RaythaHQ/raytha), built so LLM agents can
manage a whole site: themes, templates, widgets, site pages, content types, content, media and
menus. Everything prints JSON, exit codes are stable, and errors say how to fix the problem.

```
export RAYTHA_URL=https://your-site.example.com
export RAYTHA_API_KEY=...        # Settings > Administrators > (admin) > API Keys
raytha doctor                    # connectivity, key, permissions
raytha guide build-a-site        # the agent playbook (markdown)
```

## Install

Linux and macOS:

```sh
curl -fsSL https://github.com/RaythaHQ/raytha-cli/releases/latest/download/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://github.com/RaythaHQ/raytha-cli/releases/latest/download/install.ps1 | iex
```

The installer downloads the binary for your OS and CPU, verifies its SHA-256 against the release's
`SHA256SUMS`, and installs to `~/.local/bin` (`RAYTHA_INSTALL_DIR` to change, `--version vX.Y.Z` to
pin). The hosting location is one variable, `RAYTHA_INSTALL_BASE`, at the top of `install.sh`.

Build from source (Rust 1.88+):

```sh
cargo install --path .
```

Binaries are static on Linux (musl), so they run on any distribution and in minimal containers
that have CA certificates.

## Configuration

| variable | flag | meaning |
|----------|------|---------|
| `RAYTHA_URL` | `--url` | site root, for example `https://example.com` |
| `RAYTHA_API_KEY` | `--api-key` | an administrator's API key; it has that admin's permissions |

Nothing is stored on disk.

## Five-minute walkthrough for an agent

```sh
raytha doctor
raytha theme pull raytha_default_theme ./site                 # the default theme as plain files
# ... edit site/theme.json, site/web-templates/*.liquid, site/widget-templates/* ...
raytha theme push ./site --dry-run
raytha theme push ./site --activate

raytha content-type create posts --label-plural Posts --label-singular Post
raytha content create posts --data '{"title":"Hello","content":"<p>First post</p>"}'

raytha site-page create --title About --template raytha_html_page_fullwidth --route-path about \
  --sections '{"main":[{"type":"hero","settings":{"headline":"About us"}}]}'

raytha menu items create main --label About --link /about
```

Every command group has `--help`. `raytha guide` lists the topics: `overview`, `build-a-site`,
`themes`, `liquid`, `widgets`, `content-types`, `site-pages`, `media`, `errors`. The guides ship
inside the binary, so they always match the installed version.

## Output contract

- stdout is one JSON document: `{"ok":true,"data":...}` or
  `{"ok":false,"error":{"code","message","hint","fields","status"}}`.
  (`guide` prints markdown; `--help` and `--version` print text.)
- Exit codes: `0` ok, `2` usage or config, `3` auth or permission, `4` not found, `5` validation,
  `6` server or network.
- Destructive commands require `--yes`. `theme push` has `--dry-run` and an opt-in `--prune`.
- Edits are partial: flags you leave out keep their current value.

## Commands

| group | what it manages |
|-------|-----------------|
| `doctor`, `guide`, `spec` | diagnostics, docs, live OpenAPI operations |
| `theme` | themes, pull/push as files, theme media, activate |
| `web-template`, `widget-template` | Liquid templates and widget settings forms |
| `site-page` | pages built from widgets, sections, publishing |
| `content-type`, `content` | schema (fields, views) and items |
| `media`, `menu` | media library, navigation |
| `user`, `user-group` | public users and groups |

## Development

```sh
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

Integration tests run the real binary against a mock Raytha (wiremock) and check every request
against a snapshot of Raytha's OpenAPI v1 operations (`tests/fixtures/openapi-v1.json`). To refresh
the snapshot, run `scripts/refresh-openapi.py` against a running Raytha (it reads `/raytha/api/v1/swagger.json`: paths, methods,
operation ids and parameters only).

`scripts/smoke.sh` runs a live check against a disposable Raytha instance. Read its header first.

Releases: tag `vX.Y.Z` (matching `Cargo.toml`) and push. The Release workflow builds the five
binaries, writes `SHA256SUMS`, and publishes the release with the install scripts.
