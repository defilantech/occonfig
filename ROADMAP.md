# Roadmap

`occonfig` manages named model profiles for opencode configuration. This file
records what is planned and, just as importantly, what is deliberately not.

## v0.1: Model profiles (current)

The core loop: capture the model assignments you are using, name them, and
switch between them without editing JSON by hand.

- [x] `occonfig save <name>`: snapshot top-level model plus every agent model
- [x] `occonfig use <name>`: validate, back up, apply
- [x] `occonfig list`: enumerate saved profiles
- [x] `occonfig current`: show what is active and what has drifted
- [x] `occonfig set-model <provider/model>`: the one-liner for the common case
- [x] `occonfig doctor`: validate the config and flag stale profile references
- [x] Backup on every mutation, matching the existing `.bak` convention
- [x] Validation on the write path: a profile referencing a removed provider
      fails without mutating the file

## v0.2: Reach and ergonomics

- [ ] Shell completion generation for `bash`, `zsh`, and `fish`
- [ ] `occonfig diff <name>`: show what `use` would change before it changes it
- [ ] Broader provider/model reference validation, including nested provider
      aliases and model variants
- [ ] `--json` output on `list`, `current`, and `doctor` for scripting

## v0.3: Sharing and portability

- [ ] Export and import profiles, so a curated model set can move between
      machines
- [ ] Explicit opt-in for project-level opencode configs. Global-only today by
      design: auto-activating a profile based on the current directory is
      convenient and surprising in equal measure, and the surprise is worse.

## Explicitly deferred: a TUI

A full-screen terminal interface is the most requested shape for a tool like
this, and it is not planned. The reasons:

1. **The CLI covers every operation.** Every task named above is one command.
2. **A TUI is a maintenance surface.** Layout, keybinds, terminal capability
   negotiation, and rendering bugs are a permanent tax for a tool whose actual
   job is rewriting a handful of strings in a JSON file.
3. **Something already does it.** [`@mirrowel/opencode-config-studio`](https://www.opendock.net/plugins/mirrowel--opencode-config-studio/)
   is an in-TUI config editor with provenance and staged saves. If you want to
   browse settings visually, use it. `occonfig` exists because switching a
   named model set from the shell is a different problem.

If that reasoning stops holding, this section gets an entry, not a debate.

## Non-goals

- **Replacing `opencode debug config`.** That shows the merged effective config.
  `occonfig` writes the files that feed into it.
- **Managing MCP servers, plugins, or permissions.** Profiles carry model
  assignments. Everything else in `opencode.json` is global and untouched.
- **Contacting model providers.** `occonfig` never makes a network call. It
  rewrites strings and validates them against your own config.

## How to influence this

Open an issue. If you want to work on an item, comment on it first so a
maintainer can confirm it is a good fit and not already in progress. See
[CONTRIBUTING.md](CONTRIBUTING.md).
