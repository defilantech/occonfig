---
name: Bug Report
about: Report a bug or unexpected behavior
title: '[BUG] '
labels: bug
assignees: ''
---

## Bug Description

A clear and concise description of what the bug is.

## Steps to Reproduce

1. Run command '...'
2. Observe error '...'

## Expected Behavior

What you expected to happen.

## Actual Behavior

What actually happened.

## Environment

**occonfig Version:**
```bash
occonfig --version
# Output:
```

**opencode Version:**
```bash
opencode --version
# Output:
```

**Operating System:**
- [ ] macOS (Apple Silicon)
- [ ] macOS (Intel)
- [ ] Linux (x86_64)
- [ ] Linux (arm64)
- [ ] Other:

## Config Shape

The relevant structure of your `opencode.json`, **with all credentials and
private endpoints redacted**. Provider names and model strings are enough.

```json
{
  "model": "provider/model",
  "agent": {
    "build": { "model": "provider/model" }
  }
}
```

**Never paste API keys, tokens, or private hostnames.** If your issue depends
on a private endpoint existing, say "a private endpoint" rather than naming it.

## Command Output

```bash
# Paste the exact command and its full output here
```

## Additional Context

Add any other context about the problem here (screenshots, error messages,
etc.). If a backup file was written, mention whether it looks correct.
