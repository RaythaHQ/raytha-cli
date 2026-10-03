# AGENTS.md

## Using the raytha CLI (you are building or managing a Raytha site)

1. `raytha doctor` first. It checks the URL, the key, and which command groups the key may use.
2. `raytha guide` lists the docs; read `raytha guide build-a-site` before touching anything, then
   `themes`, `liquid`, `widgets`, `site-pages`, `content-types` as needed. `errors` explains every
   failure.
3. Work from files. `raytha theme pull raytha_default_theme ./site`, edit, `raytha theme push ./site
   --dry-run`, then push for real. Keep the directory in git.
4. Read `ok`, `error.code` and `error.hint` in the JSON; exit codes: 2 usage, 3 auth, 4 not found,
   5 validation, 6 server. Never guess around a `forbidden`; tell the user which permission is
   missing.
5. After a change, fetch the public page (`curl -s "$RAYTHA_URL/path"`) and look at the HTML.
   Liquid errors only show up at render time.

Configuration is two environment variables: `RAYTHA_URL` and `RAYTHA_API_KEY`.

## Working on this repository

- Rust 2024, single binary crate. `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test` must pass.
- stdout is reserved for the JSON envelope (`src/output.rs`). Never `println!` elsewhere; write
  human text to stderr only if it is essential.
- Every new command needs: a wiremock integration test in `tests/`, verified by
  `Harness::assert_contract` against `tests/fixtures/openapi-v1.json`; and, if it is user-facing,
  an example in the relevant `guide/*.md` (the `guide_commands_parse` test parses every
  ```` ```bash ```` block's `raytha ...` lines).
- Keep commands curated: friendly flags over raw API bodies, partial edits (GET, merge, PUT),
  actionable errors with hints (`src/client.rs::http_error`).
- Do not add a local `--url` flag to a subcommand; it would shadow the global one. Use another name
  (`menu items create --link`).
- Raytha 2.0.0's OpenAPI document is trustworthy, including request bodies: each command has its
  own schema (`raytha spec`). Paths, methods, parameters and body property names can be taken from
  it. The `content` object on content items and widget `settings` stay free-form; those follow the
  content type or the widget's fields. Cover a new command with a test.
- The guides are embedded with `include_str!`; editing `guide/*.md` changes the binary.
