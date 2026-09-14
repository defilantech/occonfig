# AGENTS.md

`occonfig` is a Rust CLI that manages named model profiles for opencode
configuration. It reads and rewrites `~/.config/opencode/opencode.json` on the
user's machine. This file tells coding agents how to work in this repo.
Humans: see `CONTRIBUTING.md`.

## Setup

- Rust 1.80+ (stable). Installed via [rustup](https://rustup.rs/).
- No system dependencies. Pure Rust, single static binary.
- GoReleaser and `actionlint` are opt-in for release and CI work.

## Commands

| Task                     | Command                          |
|--------------------------|----------------------------------|
| Build (debug)            | `cargo build`                    |
| Build (release)          | `make build`                     |
| Run all tests            | `make test`                      |
| Unit tests only          | `cargo test --lib`               |
| Integration tests only   | `cargo test --test cli`          |
| Tests with coverage      | `make test-cover`                |
| Format                   | `make fmt`                       |
| Check formatting         | `make fmt-check`                 |
| Lint                     | `make lint`                      |
| Compile check            | `make check`                     |
| Install locally          | `make install`                   |
| Validate release config  | `goreleaser check`               |

`make help` lists every target with its description.

## Before you commit

A change is not done until all of these pass:

1. `make fmt`
2. `make check`
3. `make lint`
4. `make test`

## Before pushing a PR

1. Run the full gate locally: `cargo fmt --check && cargo clippy -- -D warnings && cargo test`
2. If you touched `.github/workflows/`, run `actionlint` if available
3. If you touched `.goreleaser.yaml`, run `goreleaser check`
4. If you touched `hack/publish-homebrew-formula.sh`, exercise the dry run:
   `DRY_RUN=1 ./hack/publish-homebrew-formula.sh 0.0.0-snapshot dist/checksums.txt`
   (generate `dist/checksums.txt` first with
   `goreleaser release --snapshot --clean`)
5. `git status` must be clean afterward. Uncommitted generated files mean a
   step was skipped.

## Code style

- Match the surrounding code: naming, error handling, comment density, layout.
  Do not introduce a different style for new code.
- `rustfmt` owns formatting. Do not hand-format around it.
- `cargo clippy -- -D warnings` is the bar. A justified `#[allow]` carries a
  comment explaining why the lint is wrong here, or it does not get added.
- No `unwrap()` or `expect()` in a path a user can trigger. In tests, fine.
- Comments explain *why*, not *what*. Do not add doc comments or decorative
  comments to hit a quota.
- Errors propagate. A function that can fail returns a `Result`; do not swallow
  with a default and continue.

## Testing

- Every behavior change needs a test. A bug fix gets a regression test that
  fails before the fix and passes after.
- Unit tests live beside the code in `src/`, exercising pure functions such as
  `validate_ref`, profile round-trips, and version fallback.
- Integration tests live in `tests/cli.rs` and drive the built binary against a
  temp config directory via `assert_cmd`. They never touch the real
  `~/.config/opencode`.
- Assert observable behavior, not internal implementation detail.
- Do not weaken or delete a test to make a change pass. If a test is genuinely
  wrong, fix it and say why.
- **Falsify your guards.** When you add a check, apply the edit it is meant to
  catch, watch the test fail, then revert. A guard whose test passes when the
  guard is removed is not a guard.

## Commits

This repo uses conventional commit prefixes so `release-please` can generate
changelogs and version bumps. Every commit needs one:

| Prefix      | Use for                                   | Version bump |
|-------------|-------------------------------------------|--------------|
| `feat:`     | New feature, CLI command                  | minor        |
| `fix:`      | Bug fix, correctness improvement          | patch        |
| `perf:`     | Performance improvement                   | patch        |
| `docs:`     | Documentation only                        | patch        |
| `chore:`    | Deps, CI, tooling (hidden from changelog) | none         |
| `test:`     | Test-only change (hidden)                 | none         |
| `refactor:` | Refactor, no behavior change (hidden)     | none         |

Use `feat!:` / `fix!:` for breaking changes.

- Sign off every commit: `git commit -s`. A human is accountable for every
  commit, however it was produced (DCO is enforced by CI; bot-only sign-offs
  are not accepted).
- Subject says *what* changed; body says *why*. No implementation play-by-play.
- Keep commit messages free of attribution trailers (`Co-Authored-By`,
  `AI-Agent`, `Assisted-by`, etc.). Disclose AI assistance in the PR
  description instead, per [CONTRIBUTING.md](CONTRIBUTING.md) ("AI-Assisted and
  Agent Contributions").
- One logical change per commit.

## Branches and pull requests

- Contributions go through fork-based PRs. Branch from an up-to-date `main`.
- Branch names: `feat/<slug>`, `fix/<slug>`, `chore/<slug>`.
- PRs follow `.github/PULL_REQUEST_TEMPLATE.md` (What / Why / How / Checklist)
  and reference the issue with `Fixes #N`.
- Do not push to `main`. Do not force-push shared branches.

## Project layout

| Path                  | Contents                                              |
|-----------------------|-------------------------------------------------------|
| `src/main.rs`         | clap dispatch, error reporting, exit codes           |
| `src/lib.rs`          | module root so integration tests can use the crate   |
| `src/config.rs`       | Locate, load, and read model assignments             |
| `src/profile.rs`      | Profile type and on-disk storage                     |
| `src/validate.rs`     | Provider/model reference validation                  |
| `src/backup.rs`       | Backup on every mutation                             |
| `src/version.rs`      | Version string, populated by ldflags at release      |
| `src/commands/`       | One file per subcommand                              |
| `tests/cli.rs`        | Integration tests against the built binary           |
| `hack/`               | Release helper scripts (Homebrew formula publish)    |

## User data invariants

`occonfig` edits a file the user depends on. Three invariants are load-bearing
and are covered by tests:

1. **Back up before every mutation.** Every mutating subcommand writes
   `.pre-occonfig-<name>-<YYYYMMDD-HHMMSS>.bak` next to the target first. Do
   not add a write path that skips `src/backup.rs`.
2. **Preserve JSON key order.** `serde_json` is used with the `preserve_order`
   feature for exactly this reason. Removing it reorders the user's `agent`
   block alphabetically on every save. `saving_preserves_agent_key_order` fails
   if you do.
3. **Validate before writing.** A profile that references a provider or model
   absent from the live config fails without touching the file. See
   `src/validate.rs` and `use_rejects_profile_with_unknown_provider`.

A profile carries model assignments only. It never rewrites `provider`, `mcp`,
`plugin`, `tools`, or `permission`. If a change would widen that scope, it is a
design conversation first, not a PR.

## Do not

- Do not hand-edit `CHANGELOG.md`. `release-please` owns it and rewrites the
  released sections from conventional commit messages on every release PR.
  Fix the commit message instead.
- Do not commit a real opencode configuration. Fixtures only, redacted.
- Do not commit secrets, API keys, or personal endpoints.
- Do not add a code path that contacts a network. `occonfig` rewrites strings
  and validates them against the user's own config. Nothing else.
- Do not remove the `--force` guard on `save`. Overwriting a profile by
  accident loses the set a user curated.
- Do not add a TUI without reading [ROADMAP.md](ROADMAP.md) first. It is
  deferred with a stated reason, not forgotten.
