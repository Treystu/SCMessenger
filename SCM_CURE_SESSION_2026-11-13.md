# SCM-CURE session record, 2026-11-13 (v2)

Lane: scm-cure. Base: `main` @ `ced2189e9d036d389d4aa4e7e40d42357a61f5c6`.
This version supersedes the copy committed as `b4302e39` on `lane/cure-684`; that
branch carries the original record plus the #684 Part A cure. Everything here is
read from this checkout and this sandbox during the session, whose UTC clock read
2026-11-13T01:58Z early on.

## Stand-up

Canonical check: **NOT CHECKED**. This session runs in a cloud sandbox whose home
is not the operator's machine; none of the three doctrine key locations resolves:

- `~/.typesafe_key`: No such file or directory
- `~/SCM*` (SCM ORC folder keys): No such file or directory
- `~/Documents/GitHub` (Harness-configured root): No such file or directory

No key was printed, created, or claimed absent from the operator's machines. This
is a NOT CHECKED line, not a finding about his environment.

Toolchain: `which cargo rustc python3` -> `/usr/local/bin/python3` only. The
Python/CI-only rail holds by fact.

## Write channel: recorded once, as an in-repo commitment

This session searched for a working credential rather than assuming none exists:

| Location | Result |
|---|---|
| `gh auth status` | "You are not logged into any GitHub hosts" |
| `~/.config/gh` | No such file or directory |
| `~/.netrc` | No such file or directory |
| `~/.git-credentials` | No such file or directory |
| `git config --get credential.helper` (repo and global) | empty |
| `env` filtered for `token`, `github`, `gh_`, `circle` | no matches |
| `.env*` in the repo | none present |

The anonymous GitHub API was also exhausted: `GET /rate_limit` reported
`{'limit': 60, 'remaining': 0, 'used': 60}` for this shared egress IP.

Consequence, stated once: there is no GitHub write channel from here. Nothing has
been posted to #602, no PR exists, no check-run or CircleCI status has been read.
The only authenticated write is a branch push through the environment's remote
helper. Every filing obligation is discharged below as a committed artifact, and
the tracker-side actions are listed as pending rather than claimed.

Reads that were still possible, and the method that made them possible: issue and
PR pages were read as rendered HTML (`read_url` / direct GET of
`https://github.com/Treystu/SCMessenger/issues/<n>`), which is not the API and is
therefore not subject to the API quota. This is a fallback and it announces
itself: bodies came back readable for #545, #579, #580, #586, #604, #605, #612 and
#642, but a page fetched this way gives no comment-count guarantee, so anything
comment-sensitive stays unread.

## #545 draft: `docs/DEPENDENCY_RAND_010_UPGRADE_545.md`

The Batch D drafting item asked for by this session. What the draft contains:

- The issue's own tasks and acceptance, then a premise check against this tree.
- Manifest facts: `Cargo.toml:53` pins `x25519-dalek` 2.0 with `static_secrets`,
  `Cargo.toml:59` pins `rand = "0.8"`. `Cargo.lock` already carries rand 0.8.7,
  0.9.5 and 0.10.2, `rand_core` 0.6.4/0.9.5/0.10.1, and `x25519-dalek` 2.0.1 and
  3.0.0; the 0.10 copies arrive through libp2p 0.44, hickory 0.26, quinn-proto and
  proptest, and `libp2p-noise 0.47` is what already pulls `x25519-dalek 3.0.0`.
  So the upgrade changes this workspace's own edge only, and no "one rand
  version" claim would be true afterwards.
- Crate API facts read from docs.rs: `rand 0.10.2` has `rng()`, `random()`,
  `random_range()`, `fill()`, `make_rng()`; traits `TryRng`/`Rng`/`RngExt`/`Fill`;
  `rngs::SysRng` behind feature `sys_rng` as a fallible `TryRng` with `SysError`;
  `rngs::ThreadRng` behind `thread_rng`; no `OsRng` in the module.
  `x25519-dalek 3.0.0` keeps `random_from_rng<R: rand_core 0.10 CryptoRng + ?Sized>(&mut R)`
  (so the issue's mismatch premise is real, and the call shape changes from by-value
  to `&mut`), and adds `random()` behind `getrandom` plus `From<[u8; 32]>`.
- Scope correction: the issue names three files and "8+ call sites". The measured
  surface is 63 production hits in 16 files plus 55 test hits, of which only 14 are
  `random_from_rng`. The rest are `thread_rng`, `gen_range`, `.gen(`, `use rand::Rng`
  and `OsRng` fills in `privacy/`, `transport/` and `iron_core.rs`.
- Two paths, written as text rather than applied: Path A is the issue's own full
  migration, ordered so the non-crypto renames prove the manifest first; Path B is
  the small `getrandom`/`From<[u8;32]>` change that unblocks #538 without touching
  the workspace rand pin, offered as an owner choice rather than a silent
  substitution for #545's acceptance.
- One owner decision is open by construction: `KeyPair::generate() -> Self`
  (`core/src/identity/keys.rs:242`) is infallible today and becomes either
  `Result<Self>` or a panic once `SysRng` is fallible. That is an API-contract
  break, which AGENTS.md rule 9 escalates.

New artifacts that make parts of the draft checkable without a Rust toolchain:

- `scripts/x25519_rfc7748_kat_check.py` - independent pure-Python X25519 checked
  against RFC 7748 section 6.1 (Alice/Bob public keys and the shared secret),
  section 5.2 single-call, and the 1- and 1,000-iteration vectors. The
  1,000,000-iteration vector is quoted as NOT EVALUATED rather than dropped.
  Exit 0, every evaluated vector matched.
- `scripts/rand_010_migration_check.py` - inventories rand-0.8 API shapes.
  Report mode exits 0 by design so it can land before the migration; `--enforce`
  exits 1 while a production shape remains. Current reading: 118 hits in 28 files,
  63 production in 16 files, 55 test.

Not verified, and the draft says so in those words: nothing in it compiles. There
is no `rustc`/`cargo` here, and one load-bearing API question, whether `ThreadRng`
satisfies `x25519-dalek`'s `rand_core 0.10 CryptoRng` bound through the blanket
impl, could not be settled from the docs. The draft's recommendation is designed
to avoid depending on it.

## Batch D order: per-item read and carryability

Every row below was read this session from the issue or PR page. "Carryable" means
this sandbox could do the work; it does not mean it was done.

| Item | What it requires | Carryable here? |
|---|---|---|
| #545 | rand 0.8 -> 0.10, RFC 7748 KATs, unblock #538 | Drafted. Rust uncompiled, rule-8 gated, parked behind #633 |
| #579 | Point `scripts/orchestrate_strict.py` at the V051 queue; acceptance is `--dry-run` lists the cars and `orchestration_contract.py --print-version` | Yes in principle: Python only. Its own stated blocker is #580 first, so it was not started |
| #580 | Per-car status for MT-04, MT-08..MT-12 from primary sources (head SHA, CI, disposition) | Partly: repo greps yes, GitHub state reads no (API exhausted) |
| #586 | Crypto coder dispatch wave, scope undefined in the issue itself | No: rule-8 crypto, needs `cargo test --workspace --no-run` and an independent reviewer |
| #604 | PR: rustls 0.23.45 -> 0.23.46 for RUSTSEC-2026-0346/0347/0348/0349, lock only | Partly: the locked version is checkable in `Cargo.lock`; `cargo deny` and CI are not |
| #605 | Rule-8 policy text in `docs/rules/SECURITY_PROTOCOL.md`, owner-directed 2026-10-19 | Yes as a doc draft: text-only, no build |
| #612 | PR adding sccache to the test job | Nothing to do: **merged**, landed on main at `946400c62b92a684e458b48578d18042d33d76fb` per the PR page. The queue entry is stale |
| #642 | Move a required-but-vacuous context off the OCI runner | No: parked decision-ready on the Actions billing lock and on what a required context should mean for a diff it does not test |

Not read this session, and so not dispositioned: #693, #650, #651, the #619 code
half, and the #658 sweep. The API quota ran out and the HTML fallback was not
extended to them in the time available.

## Carried state (not re-litigated here)

- #622: verified cured cold on `main` in the earlier pass - 26/26 orchestration
  tests, and the real merged pack yields `(0.95, 0.95, True)` on all-pass and
  `(0.02, ...)` on a weak `instruction_matches`, with the bare three-key pack still
  accepted. The 2026-10-28 `--canon-only` ruling is not implemented; main carries
  the 2026-11-10 owner-endorsed Option 2, which supersedes it.
- #684 Part A: in-session bound cure (`CappedFileWriter`) sits uncompiled on
  `lane/cure-684` at `b4302e39`. Parts B and C are still absent from the tree.
  Not touched by this branch.

## Pending for a session that holds the write channel

1. Claim on #602 and post this stand-up with the NOT CHECKED canonical line.
2. Post the #622 cold re-verification on #622.
3. Open the PRs for `lane/cure-684` (compiled and green first) and
   `lane/cure-545`, and mark them READY-TO-MERGE only on green exact-head rail
   reads. Neither is READY-TO-MERGE now.
4. Ask the owner the one #545 question in 3.4 of the draft, and disposition #633
   so the dependabot park does not get duplicated.
5. Read the ledger, then apply every DISPATCH entry past the last ack.

## Verification actually performed in this session

- `python3 scripts/x25519_rfc7748_kat_check.py` -> all evaluated RFC 7748 vectors
  matched, rc 0.
- `python3 scripts/rand_010_migration_check.py` -> rc 0 (report mode);
  `--enforce` -> rc 1 with 63 production hits, which is the correct reading before
  the migration and is why `--enforce` is not wired in yet.
- `python3 scripts/rules_check.py --staged` -> rc 0 for the committed set.
- No emoji in the changed files; bracket/paren balance of the one changed Rust file
  in the other branch checked after stripping strings and comments.
- No Rust compiled anywhere in this session, and no CI status read.
