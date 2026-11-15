# Device-log capture and cluster state — 2026-11-10

Local Windows session with the Pixel 6a attached. Session branch `lane/burndown`,
base `main` `ba7fa77851`. Every statement below comes from a command run in this
session; method is named with each read.

**Standing fence: main is UNVERIFIABLE until the CircleCI port lands.**
`git ls-tree -r origin/main` has no `.circleci` entry. `lane/burndown` and port
head `53a0e5d8` both carry `.circleci/config.yml`.

Method note that changed an answer: `git cat-file -e <rev>:<path>` reports ABSENT
for paths whose blobs are absent from this partial clone even when the path exists.
`git ls-tree -r` is the authoritative read here. A first pass wrongly read
`lane/burndown` as having no CircleCI config on the `cat-file` basis.

---

## 1. Capture — complete

Destination `Treystu/slow-lane-ledger`,
`evidence/scmessenger/device-logs/2026-11-08-pixel6a-pre-delete/`.

- 63 files, 312,213,991 bytes
- ledger commits `91c32ed` (state, dumpsys, app data) and `a989ba7` (the runaway log)
- remote confirmed: `git ls-remote origin main` -> `a989ba738a761b0b0d59b248d06c1d694d8494e8`
- zero capture failures; receipt on PR #653 and issue #602

Device: Pixel 6a `bluejay`, `26261JEGR01896`, Android 17, wireless adb
`192.168.0.55:39239`. `run-as com.scmessenger.android` succeeded, so the installed
build is debuggable and the app-private tree is readable.

### The artifact

`files/logs/scmessenger-mesh.log`: 4,496,574,698 bytes, 15,226,408 lines,
`2026-10-10T21:42:14Z` to `2026-11-10T04:12:25Z`, never rotated, 99.9% of the app's
private data. Captured as a device-side `gzip -1` stream, 265,101,315 bytes
(16.96x), split into three parts under GitHub's per-file limit.

Integrity: `gzip -t` exit 0; part reassembly reproduces the whole-stream sha256
`f4569f7c6c90703e62add43a099f9568f108caba99adcc142708bd25d57f028c`; the
decompressed window is 15,227,153 lines. The app kept appending during capture
(+282,521 bytes between two reads), so the raw sha256 is point-in-time and the
artifact hash is the identity. Both device reads are recorded in `INVENTORY.md`.

### Method corrections that protected the evidence

1. A whole-tree `adb exec-out run-as ... tar -c files` was abandoned: it streams
   the 4.19 GB log and had produced 211 MB of partial output. It is not a hang, it
   is the wrong capture shape for a growing file.
2. `core.autocrlf` disabled before staging. With it on, git rewrites line endings
   inside the sled/sqlite blobs and every hash in the inventory would be wrong.
   Index blob hashes were verified against the working tree after staging.

### #619 is NOT proven by this capture

The device read `notifications_enabled: true`. The active foreground-service
notification (`id=1001`, channel `mesh_status`,
`ONGOING_EVENT|NO_CLEAR|FOREGROUND_SERVICE`) is correct behaviour with the toggle
ON. That satisfies criterion 1 of #619 and says nothing about criteria 2-6, which
need the toggle switched OFF and observed. This capture does not reproduce the
defect and must not be read as doing so.

## 2. New defect filed: #679

The 4.19 GB is one message stuck in an unbounded retry cycle:

```
attempt=630 pass=629 reason=RETRY_CYCLE_RESTART policy_reason=DIRECT_FIRST
destination=12D3KooWH3pTsLmksoiawE9T4JN2rES9ixg9L3wMbbyfH6fwa5w7
  Dial error: Unexpected peer ID 12D3KooWHEm4VMg9DVVxQ24PAHj21zeWvDdGGqeQVRMRmo3AbzRm
  at /ip4/127.0.0.1/tcp/9090
```

500+ restarts in a 2 MB window, 7 distinct message ids, highest counter 630. The
dashboard is loopback and answers as a different peer, so it can never converge;
the message neither delivers, fails, nor parks. Three harms: unbounded device
storage, permanent radio churn, and no terminal state for the sender.

## 3. The two blockers that stop every car

**B1 — clause 10 is unsatisfiable.** `reviewDecision` is empty on #178, #601, #611,
#609, #643, #506, #507, #508, #599, #521, #208. The repo has one collaborator
(`Treystu`) who is the author of record on every car, so no non-author can give the
two independent PASSes a code batch needs. This is not lane-fixable.

**B2 — hosted Actions jobs are billing-refused.** Confirmed by annotation: `The job
was not started because your account is locked due to a billing issue.` A1's split
holds - self-hosted OCI jobs execute, hosted jobs do not.

## 4. Cluster census read 2026-11-10T04:25Z

| Car | State | Head | Required 5 | Blocker |
|---|---|---|---|---|
| #506 | OPEN, UNSTABLE | `16b4edf754` | 5/5 success | B1 |
| #508 | OPEN, UNSTABLE | `4758160122` | 5/5 success | B1 |
| #507 | OPEN, BLOCKED | `f4d5155d4a` | 4/5 | B2 + routing |
| #521 | OPEN, UNSTABLE | `5c6b5e52ed` | unread | A17 rebase + crypto verdict |
| #599 | OPEN, CLEAN | `a7ef2f2997` | unread | A17 + gated transport files |
| #611 | OPEN, UNSTABLE | `e03c1d7df4` | unread | B1, landing is CI-owner call |
| #609 | OPEN, DIRTY | `4ed8168bbf` | unread | A12 |
| #643 | OPEN, DIRTY | `0340c09446` | unread | A13 |
| #178 | OPEN, DIRTY | `c1b457bbb1` | unread | A6 owner ruling, iOS gate Mac-only |
| #601 | CLOSED | `7f93eed6f2` | n/a | closed |
| #208 | OPEN, DIRTY | `11f3e5e93f` | unread | not assessed |

Movement since the plan: #601 closed, #521 head moved from `bff7f259e1`.

### #507: A25's premise holds, its cure does not

`Test (ubuntu-latest)` at `f4d5155d4a`, check-run `127192677132`, carries the
billing annotation with no steps executed. Not a code defect. But a refresh does
not cure it and A5 already recorded that a refresh was applied: #507 changes
**exactly one file**, a Kotlin file. A Kotlin-only diff does not touch the
Rust/CI paths, so inert-diff routing sends the required context to the hosted pool
where the lock refuses it. #506, which changes `cli/src/api.rs` and so routes to
OCI, reads 5/5 required success under the same lock. Cure is B2 or a routing change
(#642 class), never another re-run.

## 5. Assumption register

30 rows: 5 closed/withdrawn (A2, A3, A7, A15, A20), 1 not yet due (A6, 2026-11-30),
**24 overdue** (A1, A4, A5, A8, A9, A10, A11, A12, A13, A14, A16, A17, A18, A19,
A21, A22, A23, A24, A25, A26, A27, A28, A29, A30). Per clause 12 the register is
itself a blocker and nothing whose path depends on an overdue row is reported ready.
