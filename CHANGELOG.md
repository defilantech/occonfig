# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

`release-please` owns this file from the first release. Do not hand-edit
anything below the `Unreleased` section; the release workflow regenerates it
from conventional commit messages.

## [Unreleased]

### Added

- `occonfig save <name>`: snapshot the top-level model and every agent model
  assignment into a named profile.
- `occonfig use <name>`: validate a profile against the live config, back up,
  then apply it.
- `occonfig list`: enumerate saved profiles with their saved-at timestamp.
- `occonfig current`: show the active top-level model and how many agents have
  drifted from it.
- `occonfig set-model <provider/model>`: rewrite the top-level model, and every
  agent with `--agents-only`, in one command.
- `occonfig doctor`: validate the live config and report profiles whose model
  references no longer resolve.
- Backup on every mutation, using `.pre-occonfig-<name>-<YYYYMMDD-HHMMSS>.bak`.
- Validation on the write path: a profile referencing a provider or model
  absent from the live config fails without mutating the file.

[Unreleased]: https://github.com/defilantech/occonfig/commits/main
