<div align="center">

  # occonfig

  ### Named model profiles for your opencode configuration

  **Stop editing twenty JSON keys by hand every time you change which model your agents use.**

  <p>
    <a href="https://github.com/defilantech/occonfig/actions/workflows/test.yml">
      <img src="https://github.com/defilantech/occonfig/actions/workflows/test.yml/badge.svg" alt="Tests">
    </a>
    <a href="https://github.com/defilantech/occonfig/actions/workflows/lint.yml">
      <img src="https://github.com/defilantech/occonfig/actions/workflows/lint.yml/badge.svg" alt="Lint">
    </a>
    <a href="https://github.com/defilantech/occonfig/actions/workflows/security.yml">
      <img src="https://github.com/defilantech/occonfig/actions/workflows/security.yml/badge.svg" alt="Security">
    </a>
    <a href="https://github.com/defilantech/occonfig/releases">
      <img src="https://img.shields.io/github/v/release/defilantech/occonfig?label=version" alt="Version">
    </a>
    <a href="https://github.com/defilantech/occonfig/stargazers">
      <img src="https://img.shields.io/github/stars/defilantech/occonfig?style=social" alt="GitHub Stars">
    </a>
    <a href="LICENSE">
      <img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License">
    </a>
    <img src="https://img.shields.io/badge/rust-1.80%2B-orange.svg" alt="Rust Version">
  </p>

  <p>
    <a href="#the-problem">The Problem</a> &bull;
    <a href="#install">Install</a> &bull;
    <a href="#quick-start">Quick Start</a> &bull;
    <a href="#why-not-opencode-config-studio">Why not Config Studio?</a> &bull;
    <a href="#commands">Commands</a> &bull;
    <a href="ROADMAP.md">Roadmap</a> &bull;
    <a href="CONTRIBUTING.md">Contributing</a>
  </p>

</div>

---

## The Problem

You use opencode with more than one model setup. Maybe you have a fast local
model for routine edits and a large remote one for architecture work. Maybe
your homelab gateway serves several models and you rotate which one is the
default depending on what you are doing.

So you open `~/.config/opencode/opencode.json` and change the top-level
`model`. Except that only covers the primary agent. Every subagent you have
defined carries its own `model` key, and `title`, `summary`, and `compaction`
carry one too. Switch your default model and you have left two dozen agents
pointing at the thing you just stopped using.

**`occonfig` turns that into one command.** Capture the set of model
assignments you are using, name it, and switch between named profiles without
touching the JSON.

```bash
occonfig save fast-local      # snapshot what you are using now
occonfig save big-remote      # snapshot another arrangement
occonfig use fast-local       # switch, with a backup and a validation pass
```

---

## Install

### Homebrew (macOS and Linux)

```bash
brew install defilantech/tap/occonfig
```

### From source

```bash
git clone https://github.com/defilantech/occonfig.git
cd occonfig
cargo build --release
./target/release/occonfig --version
```

### Manual download

Grab the archive for your platform from the
[releases page](https://github.com/defilantech/occonfig/releases), extract it,
and put `occonfig` on your `PATH`.

---

## Quick Start

`occonfig` reads and writes your real opencode configuration at
`~/.config/opencode/`. It honors `OPENCODE_CONFIG_DIR` first and
`OPENCODE_CONFIG` second, matching opencode's own precedence.

```bash
# What am I using right now?
occonfig current

# Save the current model set under a name.
occonfig save lab

# See what you have saved.
occonfig list

# Change one model everywhere, in one command.
occonfig set-model agw-dsv41/DeepSeek-V4.1-Flash-EXL3

# Or switch to a profile you saved earlier.
occonfig use lab

# Check the config is sane and every profile still resolves.
occonfig doctor
```

Every mutating command writes a backup first, using
`.pre-occonfig-<name>-<YYYYMMDD-HHMMSS>.bak` next to your config. If a profile
references a provider or model that is not in your live config, `occonfig use`
fails **before** it writes anything.

---

## Why not opencode Config Studio?

Because it solves a different problem, and it solves it well.

[`@mirrowel/opencode-config-studio`](https://www.opendock.net/plugins/mirrowel--opencode-config-studio/)
is a visual config editor that runs inside the opencode TUI. It shows you which
file each setting came from, stages your edits, shows you a diff before saving,
and lets you browse providers and models. If you want to change what a setting
*is*, and you want to see the provenance while you do it, use Config Studio.
It is better at that than this tool will ever be.

`occonfig` does not edit settings. It answers a narrower question: **which
named set of model assignments am I currently running, and how do I switch to
another one from a shell?** That is a profile problem, not an editing problem.
Config Studio has no concept of a named profile you can switch to, and it is a
TUI plugin, so it is not scriptable from a shell or a `Makefile` or a dotfiles
bootstrap.

They compose. Use Config Studio to figure out the settings you want. Use
`occonfig` to name the result and switch to it later.

---

## Commands

| Command                              | What it does                                                     |
|--------------------------------------|------------------------------------------------------------------|
| `occonfig save <name>`               | Snapshot the top-level model and every agent model into a profile |
| `occonfig use <name>`                | Validate, back up, and apply a profile                           |
| `occonfig list`                      | List saved profiles with their saved-at timestamp                |
| `occonfig current`                   | Show the active model and how many agents have drifted from it  |
| `occonfig set-model <provider/model>`| Rewrite the top-level model (and agents with `--agents-only`)    |
| `occonfig doctor`                    | Validate the config and flag profiles with stale references     |

Run `occonfig --help` or `occonfig <command> --help` for flags.

### What a profile carries

A profile records **model assignments only**: the top-level `model` key and a
map of agent name to model. Applying a profile rewrites exactly those keys.

It never touches `provider`, `mcp`, `plugin`, `tools`, or `permission`. Those
are global configuration, and a tool that silently rewrote them because you
switched models would be a tool you stop trusting.

---

## Profiles on disk

Profiles live at `~/.config/opencode/profiles/<name>.json` and are plain JSON
you can read, edit, and commit to your dotfiles repository:

```json
{
  "name": "lab",
  "model": "agw-dsv41/DeepSeek-V4.1-Flash-EXL3",
  "agents": {
    "build": "agw-dsv41/DeepSeek-V4.1-Flash-EXL3",
    "writer": "agw-writer/gemma4-writer"
  },
  "saved_at": "2026-09-13T16:07:26Z"
}
```

A profile is not a secret. It stores model references, never credentials. If
you extend the format, keep it that way.

---

## How it works

`occonfig` is a small, boring, local tool:

- **No network calls.** It rewrites strings in a JSON file and validates them
  against your own config. Nothing else.
- **Backs up before every mutation.** Your existing `.bak` convention, extended
  with an `occonfig` marker so the tools do not collide.
- **Preserves key order.** `serde_json` runs with `preserve_order` so your
  `agent` block keeps the order you wrote it in. Without that feature, saving
  would reorder your agents alphabetically, which is the kind of quiet
  vandalism that makes people distrust a tool.
- **Fails closed on a bad reference.** A profile pointing at a provider you
  removed is caught before the write, not after.

---

## Status

Early. The core loop (save, use, list, current, set-model, doctor) works and is
tested. See [ROADMAP.md](ROADMAP.md) for what is planned and what is
deliberately deferred.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup,
conventions, and the process. We accept AI-assisted contributions under a
human-accountability standard, described in that file.

- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Security Policy](SECURITY.md)
- [Maintainers](MAINTAINERS.md)

## License

[Apache License 2.0](LICENSE).
