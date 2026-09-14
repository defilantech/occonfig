# GitHub Actions Workflows

Four workflows, all triggered on push to `main` and on pull requests unless
noted. Every action is SHA-pinned with a version comment so a supply-chain
change in a tag cannot silently alter what runs.

## Tests

**File**: `test.yml`

Runs `cargo test` with coverage instrumentation via `cargo-llvm-cov`, then
uploads `lcov.info` to Codecov.

- Installs `llvm-tools-preview` before `cargo llvm-cov`; the tool fails without it
- `fail_ci_if_error: false` on the Codecov step: an unreachable Codecov should
  not turn a green test run red

**Duration**: ~2 minutes

## Lint

**File**: `lint.yml`

Two gates, both required:

1. `cargo fmt --check`: formatting drift fails the build
2. `cargo clippy -- -D warnings`: a clippy warning fails the build

**Duration**: ~1 minute

## Security

**File**: `security.yml`

Runs `cargo audit` against the dependency tree.

- Also runs weekly on Monday at 07:00 UTC, so a newly-disclosed advisory in a
  pinned crate is caught even when the repo has no other changes
- `permissions: contents: read`

**Duration**: ~30 seconds

## DCO Check

**File**: `dco.yml`

Verifies every commit in a pull request carries a `Signed-off-by:` trailer, per
the [Developer Certificate of Origin](https://developercertificate.org/).

Pull requests only. This is the check that makes "a human is accountable for
every commit" a fact rather than an aspiration.

**Duration**: ~10 seconds

## Release Please

**File**: `release-please.yml`

Two jobs, push to `main` only:

1. **release-please**: reads conventional commit messages, opens or updates a
   release PR, and cuts a tag when it merges
2. **goreleaser**: on a created release, builds darwin and linux archives for
   amd64 and arm64, generates checksums, then runs
   `hack/publish-homebrew-formula.sh` to publish the formula to
   `defilantech/homebrew-tap`

Requires two repository secrets: `HOMEBREW_TAP_TOKEN` and `CODECOV_TOKEN` (the
latter is consumed by `test.yml`). Set them before the first release, or the
publish step fails at release time.

**Duration**: ~5 minutes

## Local parity

`make ci` runs what CI runs (`fmt-check`, `lint`, `test`) in order. Run it
before pushing to avoid the round trip.
