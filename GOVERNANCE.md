# Governance

## Overview

This document defines how the Unykorn L1 project is governed, how decisions are made, and how contributors can participate in the project's direction.

---

## Principles

1. **Correctness over speed** — We ship when it's right, not when it's fast
2. **Simplicity over features** — Every addition must justify its complexity
3. **Determinism is sacred** — Same inputs → same outputs, always
4. **Trust boundaries are law** — POPEYE → TEV → MARS → TAR, no shortcuts
5. **Open by default** — Discussions, decisions, and code are public

---

## Roles

### Maintainers

Maintainers have write access to the repository and are responsible for:

- Reviewing and merging pull requests
- Triaging issues
- Making release decisions
- Enforcing code quality standards
- Responding to security reports

**Current Maintainers:**

| Name | GitHub | Focus |
|------|--------|-------|
| Kevan | @kevan | Core architecture, all components |

*Maintainer list will grow as the project matures.*

### Contributors

Anyone who submits a pull request that gets merged is a contributor. Contributors:

- Follow the contribution guidelines
- Sign off commits (DCO)
- Participate in code review
- Help with documentation and testing

### Community Members

Anyone participating in discussions, filing issues, or using the project. Community members:

- Report bugs and request features
- Provide feedback on proposals
- Help other users
- Spread the word

---

## Decision Making

### Routine Decisions

For routine changes (bug fixes, minor improvements, documentation):

- A single maintainer can approve and merge
- Must pass CI checks
- Must follow coding standards

### Significant Decisions

For significant changes (new features, API changes, architectural decisions):

1. **Proposal**: Open an issue or RFC describing the change
2. **Discussion**: Community feedback period (minimum 7 days)
3. **Consensus**: Maintainers discuss and reach consensus
4. **Decision**: Recorded in the issue/RFC
5. **Implementation**: PR submitted and reviewed

### Breaking Changes

Breaking changes require:

- Clear justification
- Migration guide
- Extended discussion period (minimum 14 days)
- Unanimous maintainer approval

---

## Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Stages

| Stage | Stability | Use Case |
|-------|-----------|----------|
| `alpha` | Unstable | Early testing |
| `beta` | Feature complete | Integration testing |
| `rc` | Release candidate | Final testing |
| `stable` | Production ready | General use |

### Release Checklist

1. All tests passing
2. Documentation updated
3. CHANGELOG updated
4. Version bumped
5. Release notes drafted
6. Tag created and signed
7. Binaries built and checksummed
8. Announcement published

### Official Releases

Official releases are:

- Tagged in the repository
- Signed by a maintainer
- Published with checksums
- Announced on official channels

---

## Code Review

### Requirements

All code changes require:

- At least one maintainer approval
- Passing CI (tests, linting, formatting)
- DCO sign-off

### Review Guidelines

Reviewers should check:

- [ ] Code correctness
- [ ] Test coverage
- [ ] Documentation
- [ ] Performance implications
- [ ] Security implications
- [ ] Trust boundary compliance
- [ ] Determinism preservation

### Response Time

We aim to provide initial review feedback within:

- **Critical fixes**: 24 hours
- **Regular PRs**: 72 hours
- **Large changes**: 1 week

---

## Conflict Resolution

When maintainers disagree:

1. **Discussion**: Attempt to reach consensus through discussion
2. **Data**: Gather evidence, benchmarks, or user feedback
3. **Time**: Wait and revisit with fresh perspective
4. **Vote**: If necessary, maintainers vote (majority wins)
5. **Escalation**: For fundamental disagreements, defer to project founder

---

## Communication

### Official Channels

| Channel | Purpose |
|---------|---------|
| GitHub Issues | Bug reports, feature requests |
| GitHub Discussions | General questions, ideas |
| Pull Requests | Code contributions |
| README/Docs | Official documentation |

### Communication Standards

- Be respectful and professional
- Stay on topic
- Provide context and details
- Follow the Code of Conduct

---

## Adding Maintainers

New maintainers are added when:

1. They have made significant, sustained contributions
2. They demonstrate understanding of project principles
3. They are nominated by an existing maintainer
4. All existing maintainers approve

---

## Removing Maintainers

Maintainers may be removed if they:

- Become inactive for extended periods (6+ months)
- Violate the Code of Conduct
- Act against project interests
- Request to step down

Removal is discussed privately first and handled with respect.

---

## Changes to Governance

This governance document can be changed through the significant decision process:

1. Propose changes via PR
2. 14-day discussion period
3. Unanimous maintainer approval
4. Changes take effect immediately upon merge

---

## Licensing

- Code: MIT License
- Documentation: MIT License
- Contributions: Licensed under MIT (via DCO)

---

## Acknowledgments

This governance model is inspired by:

- The Rust project
- The Node.js project
- The Kubernetes project

---

*This document is version 1.0.0 and may be updated.*
