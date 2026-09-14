# Security Policy

## Supported Versions

`occonfig` is pre-1.0. Security fixes land on the latest release only; there is
no maintained back-port branch yet. This table will gain rows once a stable
line exists.

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability in
`occonfig`, please report it responsibly.

### How to Report

1. **Do NOT open a public GitHub issue** for security vulnerabilities.
2. **Preferred:** use GitHub's [private vulnerability reporting](https://github.com/defilantech/occonfig/security/advisories/new). Reports stay private, are tracked in one place, and let us publish an advisory once a fix ships.
3. **Alternative:** email **contact@defilan.com**.

### What to Include

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 7 days
- **Resolution Target**: Within 30 days for critical issues

### Scope

This security policy applies to the `occonfig` CLI and the profile and backup
files it writes under the user's opencode configuration directory.

The tool reads and writes local JSON files. The threat surface worth naming:

- Path handling when locating the config directory and profile directory
- Backup file creation and overwrite behavior
- Any future code path that shells out or makes network calls (none today)

### Out of Scope

- opencode itself, and its config schema (report to the opencode project)
- Third-party dependencies (report to upstream, though we will bump pinned
  advisories)
- Security of the model endpoints a user configures; `occonfig` never contacts
  them

## Security Best Practices

When using `occonfig`:

1. **Keep your config in version control separately.** `occonfig` writes backup
   files but does not replace your own history.
2. **Review the diff.** `occonfig use` prints what changed; read it before
   trusting a profile you did not write.
3. **Do not commit API keys into profiles.** A profile stores model references
   only, never credentials. Keep it that way if you extend the format.
4. **Run from a trusted checkout.** Until this project has releases you trust,
   build from source rather than piping an install script.
