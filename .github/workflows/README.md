# GitHub Actions Workflows

This directory contains GitHub Actions workflows for the Rainy SDK project.

## Workflows Overview

### 🔄 CI (`ci.yml`)

**Triggers:** Push/PR to main/develop branches

- **Multi-platform testing:** Ubuntu, Windows, macOS
- **Multi-Rust version testing:** Stable, beta, nightly, MSRV (1.70.0)
- **Quality checks:** Format, clippy, documentation
- **Coverage:** Code coverage reporting with Codecov

### 📦 Release (`release.yml`)

**Triggers:** Git tag push (e.g., `v1.2.3`)

- **Automated publishing:** Publishes to crates.io
- **GitHub release:** Creates release with changelog
- **Build artifacts:** Includes release binaries

### 🔒 Security (`security.yml`)

**Triggers:** Daily at 2 AM UTC, manual trigger, Cargo.toml/lock changes

- **Vulnerability scanning:** Uses `cargo audit`
- **Automated issues:** Creates GitHub issues for vulnerabilities
- **Report generation:** Detailed security reports

### 📏 MSRV (`msrv.yml`)

**Triggers:** Push/PR to main, manual trigger

- **Version compatibility:** Tests against minimum supported Rust version
- **Automated updates:** Can update MSRV badges
- **Compatibility reports:** Detailed MSRV compatibility reports

### 📦 Dependencies (`dependencies.yml`)

**Triggers:** Weekly on Mondays, manual trigger

- **Automated updates:** Updates dependencies to latest compatible versions
- **PR creation:** Creates pull requests for dependency updates
- **Testing:** Runs full test suite on updated dependencies

### ✍️ Sign-off Verification (`verify-signoffs.yml`)

**Triggers:** Pull requests to main/develop

- **DCO compliance:** Verifies Developer Certificate of Origin sign-offs
- **PR comments:** Provides helpful guidance for missing sign-offs
- **Contributing alignment:** Enforces CONTRIBUTING.md requirements

## Required Secrets

For full functionality, configure these repository secrets:

- `CRATES_IO_TOKEN`: Token for publishing to crates.io
- `GITHUB_TOKEN`: Automatically provided by GitHub

## Manual Triggers

Several workflows support manual triggering via GitHub Actions UI:

- **Security Audit:** Run on-demand vulnerability scanning
- **MSRV Check:** Test against specific Rust versions
- **Dependency Updates:** Force immediate dependency updates

## Workflow Status Badges

Add these badges to your README.md:

```markdown
[![CI](https://github.com/enosislabs/rainy-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/enosislabs/rainy-sdk/actions/workflows/ci.yml)
[![Security Audit](https://github.com/enosislabs/rainy-sdk/actions/workflows/security.yml/badge.svg)](https://github.com/enosislabs/rainy-sdk/actions/workflows/security.yml)
[![MSRV](https://github.com/enosislabs/rainy-sdk/actions/workflows/msrv.yml/badge.svg)](https://github.com/enosislabs/rainy-sdk/actions/workflows/msrv.yml)
```

## Troubleshooting

### Common Issues

1. **Crates.io Publishing Fails**
   - Check `CRATES_IO_TOKEN` secret is set
   - Verify token has publish permissions

2. **Security Audit Issues**
   - Some vulnerabilities may be false positives
   - Check cargo-audit documentation for overrides

3. **MSRV Compatibility**
   - Update MSRV in workflow file if needed
   - Check dependency MSRV requirements

4. **Sign-off Verification**
   - Contributors must use `git commit -s`
   - Or check "Sign off" in GitHub web interface

## Contributing

When modifying workflows:

- Test changes on a branch first
- Update this documentation
- Ensure backward compatibility
- Follow GitHub Actions best practices
