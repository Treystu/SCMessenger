# SCMessenger Security Audit (2026-04-19)

## Scope & Method

Audited key attack surfaces in:
- `core/` (message handling, crypto, logging)
- `cli/` (HTTP/WS control plane, installer generation, web UI)
- `wasm/` (browser storage model)
- `log-visualizer/` dependencies

Primary techniques:
- Manual code review of security-sensitive paths.
- Pattern search for risky sinks (shell script templating, HTML injection, secret logging, key persistence).
- Dependency check (`npm audit`) for JS components.
- Dynamic runtime exercises (property-test/fuzz-style execution attempt + exploit PoC generation).

---

## Scoring Mechanism

### Risk score (0–10)
`Risk = round( (Impact * 0.45) + (Exploitability * 0.35) + (Confidence * 0.20), 1 )`

Each sub-score is 0–10.

### Severity bands
- **Critical:** 9.0–10.0
- **High:** 7.0–8.9
- **Medium:** 4.0–6.9
- **Low:** 1.0–3.9

### Fix effort estimate
LOC estimates are rough implementation deltas (code + tests + docs):
- **XS:** 1–10 LOC
- **S:** 11–30 LOC
- **M:** 31–80 LOC
- **L:** 81–200 LOC
- **XL:** 200+ LOC

---

## Findings

## 1) Installer script injection via unvalidated `host` query parameter
- **Severity:** High
- **Risk score:** 8.4 (Impact 9 / Exploitability 8 / Confidence 8)
- **Effort:** S (~20–35 LOC)

### Evidence
`/api/install` directly interpolates `host` from query params into a generated shell script (`URL="http://{host}/..."`) that users are instructed to execute via `curl ... | bash` flows.

### Why this is risky
An attacker controlling or influencing the URL (clipboard poison, malicious page, social engineering, link-shortener tampering) can inject shell metacharacters or malformed host strings and get arbitrary command execution on the installer client.

### Recommended fix
- Strictly validate `host` using an allowlist for the hostname portion (for example, `^[a-zA-Z0-9.-]+(?::\d{1,5})?$` for hostname + optional port structure only), then parse and range-check the port separately to ensure it is within `1–65535`.
- If IPv6 literals are meant to be allowed, validate them with dedicated logic for bracketed IPv6 host syntax; otherwise explicitly reject them instead of relying on the hostname regex above.
- Prefer server-side canonical host generation; remove user-supplied `host` where possible.
- Avoid `curl | bash` UX patterns; provide downloaded, signed script + checksum verification.

---

## 2) Reflected/stored DOM-XSS risk in landing page ledger renderer
- **Severity:** Medium
- **Risk score:** 6.8 (Impact 7 / Exploitability 6 / Confidence 7)
- **Effort:** XS (~5–15 LOC)

### Evidence
The ledger table is rendered with `innerHTML`, and `known_topics` is injected without escaping (`(e.known_topics || []).join(", ")`), while other fields are escaped.

### Why this is risky
If topic strings become attacker-controlled (via remote metadata ingestion, compromised peer record pipeline, or future feature expansion), this becomes script injection in the local admin UI. Combined with privileged WS commands, this can escalate to local destructive actions.

### Recommended fix
- Escape `known_topics` values (map each topic through `esc`).
- Prefer DOM node creation (`textContent`) instead of string-concatenated HTML.

---

## 3) Sensitive cryptographic metadata leaked to world-readable temp logs
- **Severity:** Medium
- **Risk score:** 6.6 (Impact 7 / Exploitability 6 / Confidence 7)
- **Effort:** S (~15–40 LOC)

### Evidence
On message-processing errors, code appends diagnostics to `/tmp/scm_debug.log`, including local and sender key material identifiers (`local_key`, `sender_key`) and message IDs.

### Why this is risky
`/tmp` logs are often readable by other local users/processes depending on umask/runtime setup. This increases metadata exposure and forensic leakage beyond intended telemetry channels.

### Recommended fix
- Remove direct writes to `/tmp` for security events.
- Route to structured logger with redaction and explicit secure file permissions.
- Never log full key identifiers in error paths.

---

## 4) Private relay key file permissions hardened only on Unix
- **Severity:** Medium
- **Risk score:** 5.8 (Impact 6 / Exploitability 5 / Confidence 7)
- **Effort:** S (~20–40 LOC)

### Evidence
Relay network private key (`relay_network_key.pb`) is written and then chmod’d to `0600` only under `#[cfg(unix)]`; no equivalent Windows ACL hardening is applied.

### Why this is risky
On Windows, default ACL inheritance may be broader than intended, allowing unintended local read access to long-lived private key material.

### Recommended fix
- Add platform-specific ACL hardening for Windows (`std::os::windows` + security descriptor APIs / crate).
- Add post-write permission verification checks and explicit warnings if hardening fails.

---

## 5) WASM persistence model explicitly delegates plaintext-at-rest decisions to caller
- **Severity:** Medium
- **Risk score:** 5.2 (Impact 6 / Exploitability 4 / Confidence 6)
- **Effort:** M (~40–90 LOC)

### Evidence
Storage module states exported serialized state is intended for caller persistence via `localStorage` or OPFS; there is no built-in encryption-at-rest wrapper in this layer.

### Why this is risky
In browser contexts, XSS or compromised extensions can read plaintext persisted message/state material if integrators store raw exports without additional protection.

### Recommended fix
- Provide first-class encrypted export/import API for WASM state.
- Document secure persistence defaults (WebCrypto key wrapping, non-extractable keys, origin pinning).

---

## 6) JavaScript dependency vulnerability in `path-to-regexp` (DoS)
- **Severity:** Medium
- **Risk score:** 5.0 (Impact 5 / Exploitability 5 / Confidence 5)
- **Effort:** XS (~1–5 LOC + lock refresh)

### Evidence
`log-visualizer/package-lock.json` pins `path-to-regexp` 8.3.0, which `npm audit` reports as vulnerable to ReDoS/DoS advisories (`GHSA-j3q9-mxjg-w52f`, `GHSA-27v5-c462-wpq7`).

### Why this is risky
Crafted paths can trigger expensive regex behavior and degrade service responsiveness.

### Recommended fix
- Update to patched `path-to-regexp` (`>=8.4.0`) and regenerate lockfile.
- Re-run `npm audit` in CI.

---

## Dynamic Runtime Fuzzing & Live Exploit PoCs (This Pass)

## A) Fuzz-style runtime execution attempt (Rust property tests)

Command run:

`cargo test -p scmessenger-core proptest -- --nocapture`

Outcome:
- Build failed before fuzz/property tests could execute due pre-existing compile issues:
  - unresolved `libc` in `core/src/store/relay_custody.rs` (non-android target path),
  - test callsite arity mismatch for `is_peer_blocked(...)` in `core/src/lib.rs` tests.

Security implication:
- This currently blocks continuous dynamic security fuzzing in CI for core, reducing assurance.

## B) Live PoC — installer command injection string break-out

PoC command:

```bash
python3 - <<'PY'
host = 'localhost:9000\";echo PWNED >/tmp/scm_poc;#'
script_line = f'URL=\"http://{host}/api/download/scm-linux-amd64\"'
print(script_line)
PY
```

Observed output:

`URL="http://localhost:9000";echo PWNED >/tmp/scm_poc;#/api/download/scm-linux-amd64"`

Interpretation:
- The injected `"` closes the quoted URL assignment and allows command append.
- If this pattern is emitted by installer endpoint and piped to shell, arbitrary command execution is feasible.

## C) Live PoC — DOM injection/XSS sink manifestation

PoC command:

```bash
node - <<'NODE'
function esc(s){ const d={textContent:String(s)}; return d.textContent.replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;').replaceAll('"','&quot;').replaceAll("'",'&#39;'); }
function renderRow(e){
  return "<tr>" +
  '<td class="mono">' + esc(e.address || e.multiaddr) + '</td>' +
  '<td style="font-size:0.72rem">' + (e.known_topics || []).join(', ') + '</td>' +
  '</tr>';
}
const payload = '<img src=x onerror=\"console.log(\\'XSS\\')\">';
console.log(renderRow({address:'/ip4/127.0.0.1/tcp/9000', known_topics:[payload]}));
NODE
```

Observed output includes executable HTML:

`...<td style="font-size:0.72rem"><img src=x onerror="console.log('XSS')"></td>...`

Interpretation:
- Unescaped `known_topics` content reaches HTML output sink.

---

## Prioritized Remediation Plan

1. **Patch installer host injection immediately** (Finding #1).
2. **Fix landing-page unsafe topic rendering** (Finding #2).
3. **Remove `/tmp` sensitive debug logging** (Finding #3).
4. **Implement Windows ACL hardening for key files** (Finding #4).
5. **Patch npm dependency and add audit gate** (Finding #6).
6. **Add encrypted WASM state persistence API** (Finding #5).

---

## Audit Limitations

- I executed dynamic runtime attempts and live PoCs, but **cannot honestly claim “perfectly safe” or “all issues”** for a large, evolving codebase in one pass.
- Full-codebase exploit-complete verification would require staged threat-model-driven campaigns, dedicated fuzz harnesses, and CI gating across all targets/platforms.
- Rust dependency CVE scanning via `cargo audit` was unavailable in this environment.
