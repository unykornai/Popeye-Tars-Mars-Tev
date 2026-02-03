# Unykorn L1 — Release Checklists

This document maintains the strict checklists for cutting official releases. 

**All releases must follow this process to be considered official.**

---

## 📦 Release Candidate Checklist

Before tagging a release:

- [ ] **Test Coverage**: All `cargo test --workspace` pass
- [ ] **Soak Test**: Run `scripts/run-soak-test.ps1` for minimum 1 hour (4 hr+ for major)
- [ ] **Lint Clean**: `cargo clippy --workspace` is clean
- [ ] **Deterministic Build**: Binary build is reproducible (when applicable)
- [ ] **Version Bump**: `Cargo.toml` versions updated

## 🏷️ Tagging & Signing (Official Process)

1. **Tag the Commit**
   ```bash
   git tag -s v0.1.0-testnet -m "Official Release v0.1.0-testnet: Consensus-enabled runtime"
   # If no GPG key: git tag v0.1.0-testnet -m "..."
   ```

2. **Build Official Artifacts**
   ```bash
   cargo build --release --workspace
   ```

3. **Generate Provenance**
   ```bash
   # Windows
   Get-FileHash target/release/unykorn.exe -Algorithm SHA256 > unykorn-v0.1.0.sha256
   
   # Linux/Mac
   sha256sum target/release/unykorn > unykorn-v0.1.0.sha256
   ```

4. **Sign the Artifacts (Maintainer Only)**
   ```bash
   gpg --detach-sign --armor unykorn.exe
   ```

5. **Publish**
   - Push tag: `git push origin v0.1.0-testnet`
   - Create GitHub Release
   - Upload `unykorn.exe`
   - Upload `unykorn-v0.1.0.sha256`
   - Upload `unykorn.exe.asc` (Signature)

## 🔍 Verification Guide for Operators

To verify you are running an official Unykorn binary:

1. **Download files**:
   - `unykorn.exe`
   - `unykorn-v0.1.0.sha256`
   - `unykorn.exe.asc`

2. **Verify Checksum**:
   ```powershell
   # Expected output: True
   (Get-FileHash unykorn.exe).Hash -eq (Get-Content unykorn-v0.1.0.sha256).split(" ")[-1]
   ```

3. **Verify Signature**:
   ```bash
   gpg --verify unykorn.exe.asc unykorn.exe
   ```

---

*This process is mandated by GOVERNANCE.md.*
