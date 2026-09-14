# Contributing to occonfig

Thank you for your interest in contributing to `occonfig`! This project aims to
make switching the model set your opencode agents use a one-command operation
instead of a hand-edit of twenty-plus JSON keys.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [AI-Assisted and Agent Contributions](#ai-assisted-and-agent-contributions)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Community](#community)

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inspiring community for all.
Please be respectful and constructive in all interactions.

**Expected Behavior:**
- Use welcoming and inclusive language
- Be respectful of differing viewpoints and experiences
- Gracefully accept constructive criticism
- Focus on what is best for the community
- Show empathy towards other community members

**Unacceptable Behavior:**
- Harassment, discriminatory language, or personal attacks
- Trolling, insulting/derogatory comments
- Public or private harassment
- Publishing others' private information without permission

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates.

**Good bug reports include:**
- Clear, descriptive title
- Exact steps to reproduce the problem
- Expected vs actual behavior
- `occonfig` version (`occonfig --version`)
- opencode version (`opencode --version`)
- OS and architecture
- The relevant shape of your `opencode.json` **with credentials redacted**.
  Provider names and model strings are enough; never paste API keys.
- The failing command and its full output

**Use the bug report template** when creating issues.

### Suggesting Features

We track feature requests via GitHub Issues with the `enhancement` label.

**Good feature requests include:**
- Clear use case and problem statement
- Proposed solution (if you have one)
- Alternatives you've considered
- Examples of similar features in other projects

### First-Time Contributors

Look for issues labeled:
- `good-first-issue` - Small, well-defined tasks
- `help-wanted` - Larger tasks where we need help
- `documentation` - Documentation improvements

### Areas We Need Help

**High Priority:**
- Profile validation coverage: more provider/model reference shapes
- Support for opencode config layering (`OPENCODE_CONFIG_DIR` precedence tests)
- Shell completion generation for `bash`, `zsh`, and `fish`
- A `doctor` subcommand that reports drift across every profile

**Medium Priority:**
- Interactive profile picker for `occonfig use` with no argument
- Export and import profiles for sharing between machines
- Support for project-level opencode configs as an explicit opt-in

**Explicitly deferred:**
- A full-screen TUI. The CLI covers every operation today, and a TUI is a
  larger maintenance surface than the problem justifies right now. See
  [ROADMAP.md](ROADMAP.md).

## Development Setup

### Prerequisites

**Required:**
- **Rust 1.80+**: Install from [rustup.rs](https://rustup.rs/)
- **git**: For DCO sign-off and conventional commits

**Optional:**
- **GoReleaser**: Only needed to test the release pipeline locally
  (`goreleaser check`, `goreleaser release --snapshot --clean`)
- **actionlint**: Only needed to lint the GitHub Actions workflows locally

### Clone and Build

```bash
# Fork the repository on GitHub first, then:
git clone git@github.com:<your-username>/occonfig.git
cd occonfig

# Build the binary
cargo build --release

# Run the test suite
cargo test

# Lint and format
cargo clippy -- -D warnings
cargo fmt --check
```

### Running Locally

`occonfig` reads and writes your real opencode configuration. During
development, point it at a throwaway directory so you do not mutate the config
you actually use:

```bash
mkdir -p /tmp/occonfig-test
cp ~/.config/opencode/opencode.json /tmp/occonfig-test/
OPENCODE_CONFIG_DIR=/tmp/occonfig-test ./target/release/occonfig current
```

`occonfig` honors `OPENCODE_CONFIG_DIR` first and `OPENCODE_CONFIG` second, in
that order, matching opencode's own precedence.

## Making Changes

### Branching Strategy

- `main` is the release branch and always green
- Create feature branches from `main`, named descriptively
  (`fix/backup-naming`, `feat/interactive-picker`)
- Rebase on `main` before requesting review

### Commit Messages

We use [conventional commits](https://www.conventionalcommits.org/) because
[Release Please](https://github.com/googleapis/release-please) reads them to
generate the changelog and pick version bumps.

```
<type>: <description>

<optional body>

Signed-off-by: Your Name <you@example.com>
```

**Types:** `feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `chore`, `ci`

**Examples:**
```
feat: add interactive profile picker
fix: reject profiles referencing a removed provider
docs: explain the profile format in the README
test: cover backup naming across a DST boundary
```

**Sign off every commit** (`git commit -s`) per the
[DCO](https://developercertificate.org/). CI enforces this.

### Code Changes

- Keep the diff scoped to the issue. Unrelated edits make review harder and
  will be asked to split.
- Add or update tests for behavior changes. See [Testing Guidelines](#testing-guidelines).
- Update the README when user-visible behavior changes.
- Never commit a real opencode configuration. Fixtures only, and redact any
  provider names that identify a private endpoint if they are yours.

### Testing Guidelines

**Unit tests:** live beside the code in `src/`, exercising `validate_ref`,
profile round-trips, and version fallback.

```bash
cargo test --lib
```

**Integration tests:** live in `tests/cli.rs` and drive the built binary
against a temporary config directory via `assert_cmd`.

```bash
cargo test --test cli
```

**The whole suite:**

```bash
cargo test
```

**A test that still passes when the code under test is broken is not
coverage.** When you add a guard, falsify it: apply the edit it is supposed to
catch, watch the test fail, then revert. The falsification block in any plan
handed to an agent exists for exactly this reason.

### Lint and Format

```bash
cargo fmt --check
cargo clippy -- -D warnings
```

Both run in CI. `cargo clippy -- -D warnings` means a warning fails the build;
fix it rather than silencing it with an `#[allow]` unless the `allow` carries a
comment explaining why the lint is wrong here.

## AI-Assisted and Agent Contributions

`occonfig` is built by people who use AI coding agents heavily. This repository
was scaffolded and implemented with agent assistance, and we welcome
AI-assisted contributions. We hold them to one standard: human accountability,
not where the code came from.

If any part of your contribution (code, tests, commit messages, PR text) was
generated or substantially assisted by an AI tool, the following apply.

**A human is accountable.**

- A human signs off every commit with the DCO (`git commit -s`). The sign-off
  means a person takes responsibility for understanding, testing, and
  maintaining the change, however it was produced. Bot-only sign-offs are not
  accepted.
- A human, not an agent, owns the review conversation. Replies to review
  feedback come from you, not from an autonomous agent posting on your behalf.

**Disclose the assistance.**

- State in the PR description which tool was used, what it generated, and what
  you verified before submitting. One line is enough, for example:
  `Assisted-by: <tool> (generated the tests; I reviewed and ran cargo test)`.
- Put the disclosure in the PR body, not the commit message. We keep commit
  messages free of attribution trailers (see `AGENTS.md`).

**Same bar as any other PR.**

- Tests must exercise real behavior. A test that still passes when the code
  under test is broken is not coverage.
- The change must match the scope of the issue it claims to fix: no unrelated
  edits, no placeholder or fabricated content.
- Run the gate locally first: `cargo test`, `cargo clippy -- -D warnings`.

**Unsolicited and drive-by PRs.**

- We may close unsolicited AI-generated PRs that were not preceded by an issue
  or discussion, especially low-effort ones targeting `good-first-issue`
  labels. To work an issue, comment on it first so a human can confirm it is a
  good fit and not already in progress.

This is not an anti-AI policy. It is a "stand behind your work" policy. We
dogfood agentic contribution daily; we just require that a person stay in the
loop and on the hook.

## Pull Request Process

### Before Submitting

- [ ] Code passes all tests (`cargo test`)
- [ ] Linter passes (`cargo clippy -- -D warnings`)
- [ ] Formatter passes (`cargo fmt --check`)
- [ ] Manually tested against a **copy** of a real opencode config
- [ ] Documentation updated (if adding features)
- [ ] Commit messages follow conventions
- [ ] All commits signed off (`git commit -s`)
- [ ] Branch is up-to-date with `main`
- [ ] No real credentials, endpoints, or personal config in fixtures

### Submitting a PR

1. **Push your branch** to your fork
2. **Open a Pull Request** against `main`
3. **Fill out the PR template** completely
4. **Link related issues** (e.g., "Closes #42")
5. **Request review** from maintainers

### PR Title Format

**Important:** PR titles are used for automated changelog generation and
version bumps via Release Please. Since we use **squash merging**, your PR
title becomes the commit message on `main`.

```
<type>: <description>
```

### Review Process

1. **Automated checks** run first: tests, lint, and the DCO check
2. **Maintainer review** focuses on correctness, test quality, and scope
3. **Revisions** may be requested; push additional commits to the same branch
4. **Merge** happens via squash once approved and green

## Coding Standards

### Rust Code Style

- `rustfmt` owns formatting. Do not hand-format around it.
- `cargo clippy -- -D warnings` is the bar. A justified `#[allow]` carries a
  comment, or it does not get added.
- Prefer `thiserror` or `anyhow` for errors, but do not swallow them. A
  function that can fail returns a `Result`.
- No `unwrap()` or `expect()` in a path a user can trigger. `unwrap()` in a
  test is fine.
- Public functions get doc comments only where the contract is not obvious.
  Do not narrate the body.
- Match the surrounding style, including how comments are written.

### Config and File Handling

- **Always back up before mutating a user's file.** Every mutating subcommand
  goes through `src/backup.rs`.
- **Preserve JSON key order.** `serde_json` is used with the `preserve_order`
  feature for this reason. Removing it reorders the user's `agent` block
  alphabetically on every save.
- **Validate before writing.** A profile that references a provider or model
  absent from the live config must fail without touching the file.

### Documentation

- Update `README.md` for user-visible changes
- Keep `AGENTS.md` current when the commands, layout, or conventions change, so
  future agent sessions do not drift

## Community

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and broader conversation, when enabled

### Recognition

Contributors are recognized in the [CHANGELOG.md](CHANGELOG.md) through
Release Please, and significant contributors may be invited to
[MAINTAINERS.md](MAINTAINERS.md).

## Development Workflow Example

```bash
# 1. Create feature branch
git checkout main && git pull
git checkout -b feat/interactive-picker

# 2. Make changes
# ... edit src/commands/use_profile.rs ...

# 3. Write tests
# ... add a test in tests/cli.rs ...

# 4. Run tests
cargo test

# 5. Test manually against a copy, never the live config
OPENCODE_CONFIG_DIR=/tmp/occonfig-test ./target/release/occonfig use lab

# 6. Lint and format
cargo fmt
cargo clippy -- -D warnings

# 7. Update documentation
# ... edit README.md ...

# 8. Commit with sign-off
git commit -s -m "feat: add interactive profile picker"

# 9. Push and create PR
git push -u origin feat/interactive-picker
# Open PR on GitHub
```

## Questions?

Open an issue or start a discussion. We would rather answer a question than
have you guess at the intended behavior.

## License

By contributing, you agree that your contributions will be licensed under the
[Apache License 2.0](LICENSE).
