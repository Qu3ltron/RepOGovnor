# Security Policy

## Supported Versions

The latest v2 patch release is supported for security fixes.

## Reporting a Vulnerability

Use GitHub private vulnerability reporting if it is enabled for this repository,
or open a GitHub Security Advisory draft with the maintainers. Do not open a
public issue for a suspected vulnerability.

Include:

- Affected version or commit.
- Impact and reproduction steps.
- Any relevant logs or proof of concept.
- Whether the report affects generated consumer workspaces, local hooks, or the
  Rust task-registry CLI.

We will triage reports against the latest v2 release and publish fixes through a
new patch release when needed.

## Local Trust Boundary

This plugin installs repo-local hooks and scripts that run in the consumer
workspace. Treat a repository checkout as executable input. The installer and
posture checks are hardened to avoid writing outside the target repo, sourcing
repo-controlled env files, or executing target-controlled wrappers during
status checks, but users should still review untrusted repositories before
enabling their hooks.
