# Security Policy

## Reporting a Vulnerability

The Unykorn team takes security seriously. If you discover a security vulnerability, please report it responsibly.

### 🔐 How to Report

**DO NOT** open a public GitHub issue for security vulnerabilities.

Instead, please email: **security@unykorn.io** *(or open a private security advisory on GitHub)*

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Any suggested fixes (optional)

### ⏱️ Response Timeline

| Stage | Target Time |
|-------|-------------|
| Initial acknowledgment | 24 hours |
| Initial assessment | 72 hours |
| Status update | 7 days |
| Fix or mitigation | Depends on severity |

We will keep you informed throughout the process.

---

## Severity Classification

We classify vulnerabilities using the following severity levels:

### 🔴 Critical

- Remote code execution
- Consensus bypass or finality violation
- State corruption across nodes
- Private key exposure
- Complete loss of funds (when applicable)

**Response**: Immediate. All resources mobilized.

### 🟠 High

- Denial of service affecting network liveness
- Signature verification bypass
- Crash-safe persistence failure
- Validator impersonation
- Trust boundary violation

**Response**: Within 48 hours. Patch prioritized.

### 🟡 Medium

- Non-critical denial of service
- Information disclosure (non-sensitive)
- Memory leaks affecting stability
- Configuration vulnerabilities

**Response**: Within 7 days. Scheduled fix.

### 🟢 Low

- Minor issues with limited impact
- Documentation errors with security implications
- Non-exploitable edge cases

**Response**: Within 30 days. Normal release cycle.

---

## Scope

This security policy covers:

| Component | In Scope |
|-----------|----------|
| MARS (Runtime) | ✅ |
| TEV (Verification) | ✅ |
| POPEYE (Networking) | ✅ |
| TAR (Persistence) | ✅ |
| Consensus | ✅ |
| Node binary | ✅ |
| Official releases | ✅ |
| Documentation site | ⚠️ Limited |
| Community forks | ❌ Out of scope |

---

## Trust Model

Unykorn L1 maintains strict trust boundaries:

```
POPEYE (untrusted) → TEV (verification gate) → MARS (state) → TAR (persistence)
```

Security reports should identify which boundary is affected.

### Key Invariants

1. **Nothing from POPEYE reaches MARS without passing TEV**
2. **MARS execution is deterministic** (same inputs → same state)
3. **TAR writes are atomic and crash-safe**
4. **Consensus never mutates state directly**

Violations of these invariants are automatically **Critical** severity.

---

## Supply Chain Security

### Dependencies

- All dependencies are specified in `Cargo.lock`
- We audit dependencies for known vulnerabilities
- We prefer minimal, well-maintained crates
- Security-critical crates (ed25519-dalek, sha2) are pinned

### Build Process

- Official releases are built from tagged commits
- Binaries include SHA-256 checksums
- Release signatures verify provenance
- CI/CD pipeline enforces security checks

### Verification

To verify a release:

```bash
# Check checksum
sha256sum -c unykorn-v0.1.0.sha256

# Verify signature (when available)
gpg --verify unykorn-v0.1.0.sig unykorn-v0.1.0
```

---

## Secure Development Practices

### Code Review

- All changes require PR review
- Security-sensitive changes require maintainer approval
- No force-push to `main` branch

### Testing

- Unit tests for all components
- Integration tests for trust boundaries
- Soak tests for stability
- Fuzzing for critical paths (planned)

### Static Analysis

```bash
# Linting
cargo clippy --workspace

# Security audit
cargo audit
```

---

## Disclosure Policy

### Our Commitment

- We will acknowledge your report promptly
- We will keep you informed of progress
- We will credit you in the security advisory (unless you prefer anonymity)
- We will not take legal action against good-faith security researchers

### Coordinated Disclosure

We follow a coordinated disclosure process:

1. Report received and acknowledged
2. Vulnerability confirmed and assessed
3. Fix developed and tested
4. Patch released with advisory
5. Public disclosure after users have time to update

Standard disclosure timeline: **90 days** from initial report, or sooner if a fix is released.

---

## Security Advisories

Security advisories will be published:

- In the GitHub Security Advisories section
- In release notes for patched versions
- On official communication channels

Subscribe to repository notifications to receive security updates.

---

## Bug Bounty

*A formal bug bounty program is planned for the future.*

Currently, we offer:
- Public acknowledgment
- Inclusion in the security hall of fame
- Our sincere gratitude

---

## Known Security Considerations

### Current Limitations

| Area | Status | Notes |
|------|--------|-------|
| Formal verification | Planned | Not yet implemented |
| External audit | Planned | Not yet completed |
| Slashing | Deferred | Double-signing logged, not penalized |
| Encryption at rest | Not implemented | TAR stores data unencrypted |

### Assumptions

- Validator keys are stored securely by operators
- Network adversaries cannot permanently partition honest validators
- f < n/3 validators are Byzantine

---

## Contact

- **Security Reports**: security@unykorn.io
- **General Questions**: Open a GitHub issue
- **Urgent Issues**: Include "URGENT" in email subject

---

## Acknowledgments

We thank all security researchers who help keep Unykorn L1 secure.

### Security Hall of Fame

*No entries yet. Be the first!*

---

*This policy may be updated. Check the repository for the current version.*

*⚖️ Attorney Review Recommended: Disclosure policies and legal safe harbor language should be reviewed by qualified legal counsel.*
