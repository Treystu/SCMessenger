# Epic burndown session record - 2026-11-10

Session branch `freebuff/43b05b2aadda7ca389644aa3`, based on `origin/main`
`ba7fa77851f0c47e2bf3037f8d9c56f8dd966aad`.

Every statement below comes from a command run in this session. Where a claim could
not be re-read here it is labelled CARRIED and its source named, rather than restated
as a fact of this session (AGENTS.md rule 13).

**Fence, stated once and standing: main is UNVERIFIABLE.** `origin/main` carries no
`.circleci/config.yml`. Read this session:

```
$ git cat-file -e origin/main:.circleci/config.yml   -> ABSENT
$ git cat-file -e origin/lane/burndown:.circleci/config.yml -> PRESENT
```

No car based on main can produce an admissible battery until the port lands.

---

## 0. Stand-up block

Posted nowhere: this environment holds no GitHub write credential (section 6). The
block is recorded here verbatim so the next session can file it unchanged.

### (a) Canonical Jev check

```
$ python3 scripts/jev_canonical_check.py --wp STANDUP-2026-11-10 \
      --state-file tmp/standup_state.json
[FAIL] Harness checkout missing: /workspace/chats/43b05b2aadda7ca389644aa3/vendor/sovereign-harness
exit=1
```

**No `typesafe_key` line was produced. There is no valid key claim in this session.**
The gate refuses before it reaches a model because its pinned admission root
(`scripts/harness_admission.json` -> `vendor/sovereign-harness`) is absent from this
checkout and the checkout has no `.gitmodules`. `TYPESAFE_API_KEY`, `TYPESAFE_TOKEN`
and `OPENROUTER` credentials are all unset. Per the standing rule, this reads as NOT
CHECKED, not as a pass and not as a soft skip.

### (b) Disk budget

```
$ python3 scripts/disk_budget.py
  filesystem : 15.58 GB total, 9.46 GB free (34.1% used)
  thresholds : TIGHT below 20 GB, BLOCKED below 8 GB
  verdict    : TIGHT
exit=1
```

TIGHT, not BLOCKED: a build is not refused outright, but the guard's own advice is to
prefer a single targeted run over a workspace build. Section 5 explains why no build
ran anyway.

### (c) adb devices -l

```
$ command -v adb          -> not found (exit 1)
$ adb devices -l
bash: adb: command not found   (exit 127)
```

No adb binary and no device transport in this environment. This is the capture blocker
in section 1, not a "no devices attached" reading - the tool itself is absent.

### (d) Lane head SHA

```
session HEAD          = ba7fa77851f0c47e2bf3037f8d9c56f8dd966aad
origin/main           = ba7fa77851f0c47e2bf3037f8d9c56f8dd966aad
origin/lane/burndown  = 7007e3dc112855c47bdd51a078f4ff7f19caef45
port head (#677)      = 53a0e5d83beab844df26f42af118ed7d62b99479
```

The burndown lane head is unchanged at `7007e3dc` (committed 2026-11-08 23:36 -1000).
The session branch sits on `main`, so nothing in this session moved the lane head.

---

## 1. Priority 1 - Pixel 6a device-log capture: BLOCKED

Filed once, precisely. It is not re-nudged in this record.

**What was asked:** pull all app logs from the Pixel 6a into
`Treystu/slow-lane-ledger` at
`evidence/scmessenger/device-logs/2026-11-08-pixel6a-pre-delete/`, and post the receipt
on #653 and #602. Deletion of the app is ON HOLD until that receipt posts; #619 is the
reason for the capture.

**The blocker is three independent facts, any one of which alone is fatal:**

| # | Fact | Read in this session |
|---|---|---|
| 1 | No `adb` binary exists here | `command -v adb` exit 1; `adb devices -l` exit 127 |
| 2 | This is a cloud worktree with no USB or wireless-debugging path to a physical phone. `adb devices -l` could not even be attempted as a real query | exit 127 above |
| 3 | The destination is unreachable: `Treystu/slow-lane-ledger` answers HTTP 404 unauthenticated (private or not shared with this identity), and there is no GitHub write credential here | `curl -o /dev/null -w '%{http_code}'` -> 404; `gh auth status` -> not logged in |

**Consequence:** the capture cannot run and the receipt cannot post from this
environment. The deletion HOLD therefore stands. This is one blocker note.

**What clears it:** a session on the operator host with the Pixel attached
(wake/unlock, wireless debugging on, same LAN) and ledger write access. Per the
2026-11-09 steering, one device window serves three uses - the capture receipt, the
#619 on-device proof (acceptance criteria 1-5), and the later reinstall check. Do not
spend separate windows.

---

## 2. Priority 2 - the CircleCI port (#677): DECISION-READY

The lane's one open question on #677 has been: *zero `ci/circleci:*` contexts on the
port head while `lane/burndown` posts twelve - does a pipeline exist, and is the
CircleCI project even wired to this repository?*

This session answers the second half, and it retires one of the two candidate causes.

### The read

Combined status per SHA (`GET /repos/Treystu/SCMessenger/commits/<sha>/status`):

| SHA | What it is | `ci/circleci:*` contexts |
|---|---|---|
| `53a0e5d83` | #677 head | **0** (`total_count=0`) |
| `ba7fa7785` | `main` | **0** (`total_count=0`) |
| `7007e3dc` | `lane/burndown` head | **12, all `success`** |

The twelve, with their creation times:

```
2026-11-09T09:36:19Z  Handoff ownership scope
2026-11-09T09:36:19Z  JavaScript Linting
2026-11-09T09:36:19Z  Naming policy
2026-11-09T09:36:19Z  Orchestration control plane
2026-11-09T09:36:43Z  Repository Hygiene Checks
2026-11-09T09:36:46Z  Rust Linting
2026-11-09T09:38:35Z  Kotlin Linting
2026-11-09T09:41:20Z  Docs
2026-11-09T09:41:28Z  Lint
2026-11-09T09:45:31Z  FFI Surface Contract
2026-11-09T09:55:31Z  Forgery/Negative Security Regression Tests
2026-11-09T10:01:05Z  Test (ubuntu-latest)
```

### What it proves, and what it kills

1. **The CircleCI project IS wired to this repository.** Twelve `ci/circleci:*`
   contexts were produced *from a branch in `Treystu/SCMessenger`*. That kills the
   "wrong project slug (`Sovereign-Communication` vs `Treystu`)" candidate outright -
   it is not a slug mismatch, because the rail demonstrably reports on this repo.
2. **It also kills a branch-filter in the config.** The port workflow carries no
   `filters:`/`branches:` block at all - `grep -n "filters:" ` over the ported
   `.circleci/config.yml` returns nothing, and the `workflows:` block schedules all
   twelve jobs unconditionally. Nothing in the config excludes the port branch.
3. **The port head's last push predates the first context this repository ever
   showed.** The port branch commits, and the lane head, in the order they happened:

```
53a0e5d8 2026-11-08 07:02:57 -1000  ci: re-fire the rail on an unchanged tree
201d2c1c 2026-11-08 06:45:56 -1000  ci(circleci): fix the four review-thread defects
f3b0db3c 2026-11-08 00:49:35 -1000  ci(circleci): port the rail config to main
7007e3dc 2026-11-08 23:36:05 -1000  docs(audit): stop the digest restating its own head
```

`53a0e5d8` was pushed 2026-11-08T17:02:57Z. The earliest `ci/circleci:*` context on
any SHA in this repository was created 2026-11-09T09:36:19Z - **sixteen and a half
hours later.** The commit literally named "re-fire the rail on an unchanged tree" was
fired before the rail existed here. That is why it produced nothing, and it is a
complete explanation of the zero: not a missing pipeline gate, a **pre-wiring head**.

### Disposition

**DECISION-READY.** One owner act remains, and it is a re-fire, not a config change:

> Push one commit (empty is fine) to `ci/circleci-port-main-20261108`, or use the
> CircleCI "re-run" on that branch, now that the project is wired. The battery should
> appear within ~30 minutes. If it does, #677 is merge-ready on its five required
> Actions contexts plus a green twelve-job rail.

**Why the lane cannot do it:** the port branch is the head branch of an open pull
request (#677). Per AGENTS.md rule 5 no capability class may force-push a shared
branch, and this session has no push rights to a branch it does not own. Pushing to
`ci/circleci-port-main-20261108` is not a lane action.

**Not verified here:** the five required contexts on `53a0e5d8`. The unauthenticated
API budget (60 requests/hour) was exhausted before that read, so the 11-09 oversight
reading (5/5 success, latest per job) is CARRIED and labelled, not restated as this
session's own.

### The folded #528 repair (steering P1 item 4: "drop the `toolchain:` input, keep the pin in the action ref")

`.github/workflows/lint.yml` fed a `toolchain:` input to an action that has no such
input. Read from the action's own manifest at the pinned tag:

```
$ curl https://raw.githubusercontent.com/dtolnay/rust-toolchain/1.99.0/action.yml
inputs:
  targets:     ...
  target:      "Alias for `targets`"
  components:  ...
```

No `toolchain` input exists. The version at that tag is baked into the action body
(`env: toolchain: 1.99.0` inside the composite step), so **the ref is the pin** and the
call-site input was an undeclared input the action ignores. Fixed by deleting that one
line and leaving `components: rustfmt`, plus a comment saying where the pin now lives.

Checked, not assumed:

```
$ python3 -c "...yaml.safe_load('lint.yml')..."        -> yaml OK, 5 jobs
$ job rust (name: "Rust Linting")
    uses : dtolnay/rust-toolchain@1.99.0
    with : {'components': 'rustfmt'}
$ python3 scripts/check_toolchain_pin.py
  pinned toolchain: 1.99.0  (from rust-toolchain.toml)
  checked 26 declaration(s): 25 action ref(s), 1 toolchain input(s)
  [OK] every declared Rust toolchain is pinned to 1.99.0 and agrees with rust-toolchain.toml
  exit=0
```

It was `27 declaration(s): 25 action ref(s), 2 toolchain input(s)` before the edit. The
one remaining `toolchain:` input is `desktop.yml:40` on `actions-rs/toolchain@v1`, whose
manifest **does** declare `toolchain` - so it is correct there and was left alone.

---

## 3. #619 - what the code actually says (new narrowing)

#619's premise, per the issue: notifications "still come through anyways" with the
in-app toggle OFF, so the switch "clearly does nothing". The capture was meant to say
which card the owner sees. The code can already bound the answer.

### The complete post inventory

Added `scripts/notification_gate_audit.py` this session: a static guard that finds
every Android post primitive and requires each one to be either behind the
`notificationsEnabled` gate or on a documented mandatory-foreground-service
allowlist. It prints **all** sites, never a sample (rule 15).

```
$ python3 scripts/notification_gate_audit.py
post sites found : 8
[EXEMPT-MANDATORY-FGS] MeshAutoRestart.kt:187       getForegroundInfo
[EXEMPT-MANDATORY-FGS] MeshAutoRestart.kt:193       getForegroundInfo
[EXEMPT-MANDATORY-FGS] MeshForegroundService.kt:730 updateNotification
[EXEMPT-MANDATORY-FGS] MeshForegroundService.kt:737 tryStartForeground
[EXEMPT-MANDATORY-FGS] MeshForegroundService.kt:744 tryStartForeground
[GATED] NotificationHelper.kt:550                   showMessageNotification
[GATED] NotificationHelper.kt:569                   showMessageNotification
[GATED] NotificationHelper.kt:686                   showPeerDiscoveredNotification
gated: 3   exempt (mandatory FGS): 5
[OK] ... exit=0
```

### Findings

**F1 - Every user-content path is gated, and gated twice.** Message and
message-request cards post only from `NotificationHelper.showMessageNotification`,
after the global gate, the DND check, the per-kind gates, the foreground-suppression
check and the POST_NOTIFICATIONS permission check. Peer-discovery cards gate on the
same global field, failing closed while unhydrated. There is no second posting path:
`MeshRepository` never touches `NotificationManager`, and the only production callers
are `MeshForegroundService:828` (message) and `:401` (peer event).

**F2 - The gate is hydrated from the store the toggle writes.** `SettingsScreen`
collects `SettingsViewModel.notificationsEnabled`, which is
`PreferencesRepository.notificationsEnabled` (DataStore key `notifications_enabled`);
`setNotificationsEnabled` applies the value to the in-memory gate immediately *and*
persists it; `MeshForegroundService.hydrateNotificationGates()` collects that same
flow for the service lifetime. The "two settings stores, switch writes the wrong one"
hypothesis in the issue does **not** hold on this head: the toggle and the gate are on
the same DataStore key.

**F3 - Exactly two ungated classes survive, and both are mandatory foreground-service
notifications.**
- the persistent mesh-status card (`MeshForegroundService` `createNotification` ->
  `startForeground` / `updateNotification`, channel `mesh_status`, IMPORTANCE_LOW,
  silent, ongoing, **not dismissible** by the user), and
- the transient "mesh resuming" card from the expedited retry worker
  (`MeshEnsureWorker.getForegroundInfo`, same channel, silent, ongoing).

**F4 - So with the toggle OFF, the only thing that can still sit in the notification
bar is the mesh-status card.** That is the strongest match for "the switch clearly
does nothing": the card is permanent while the mesh runs and the user cannot dismiss
it, so the shade never looks different after flipping the switch. This is not proof -
only the device window proves which card the owner saw - but it is the only remaining
candidate on this head, because F1 and F2 close the message and store paths.

**F4a - and that card keeps rewriting itself.** `updateNotification()` is called from
three collectors, not just at startup:

```
MeshForegroundService.kt:399  PeerEvent.Connected     -> connectedPeers.add(...); updateNotification()
MeshForegroundService.kt:409  PeerEvent.Disconnected  -> connectedPeers.remove(...); updateNotification()
MeshForegroundService.kt:422  StatusEvent.StatsUpdated -> messagesRelayed.set(...); updateNotification()
```

and the text it re-posts is `notification_mesh_status_content_format` filled with
`connectedPeers.size` and `messagesRelayed.get()`. So with the toggle OFF the card does
not merely persist - its text visibly changes on every peer connect/disconnect and
every stats tick. Peer-discovery cards themselves are gated (F1), so peer events
produce no *new* card; they refresh the mandatory one. That is a better fit for
"notifications still come through anyways" than a permanent static card is.

**F5 - "OFF means nothing posts" is not implementable literally while the mesh runs as
a foreground service.** Android requires a visible notification for an FGS; suppressing
it would end the foreground state, which is the thing keeping the mesh alive. #619
already provides for this: *"If the persistent foreground-service status notification
is intentionally exempt, say so explicitly in the fix and in the UI copy."*

### Disposition

**DECISION-READY**, on one owner decision:

- (a) Confirm the mesh-status card as the documented exemption, and state it in the UI
  copy under the toggle. This is what the issue pre-authorises, it is a two-file
  change (one string, one conditional subtitle), and it makes the switch truthful.
- (b) Or, if OFF must also remove the card, accept that the mesh can no longer run as
  a foreground service - an architecture change, which rule 9 reserves to the operator.

**No Kotlin was edited this session, deliberately.** `java` is not installed and
`ANDROID_HOME`/`ANDROID_SDK_ROOT` are unset, so a Kotlin change here cannot be
compiled or unit-tested; and because main carries no rail (section 2), it could not be
verified downstream either. Writing an uncompilable edit into a shared service file
whose only checker is a CI run that cannot happen yet is the worst trade available.
The guard, the inventory and the decision are the honest deliverable.

### The exact cure, for a session that has a JDK

Four edits, all inside the F1/F4a boundary, none of which touches `startForeground`:

1. `res/values/strings.xml` - add
   `settings_description_notifications_off`:
   "Message notifications are off. One silent card stays visible while the mesh
   service runs, because Android requires it for a background service."
2. `ui/screens/SettingsScreen.kt` (`AppPreferencesSection`, the
   `SwitchPreference` at ~714) - select that string as the subtitle when
   `notificationsEnabled` is false, keeping `settings_description_notifications`
   otherwise. This is the UI copy #619 asks for.
3. `service/MeshForegroundService.kt` (`createNotification`) - when
   `NotificationHelper.notificationsEnabled == false`, use a static content string
   with no peer/relay counts or actions instead of the formatted one. The card still
   exists (F5), it just stops carrying live mesh state.
4. `service/MeshForegroundService.kt` (`updateNotification`) - early-return when the
gate is false, since the card content is now constant: no re-post on peer or stats
   events, which is the F4a churn that makes the toggle look inert.

Then: `python3 scripts/notification_gate_audit.py` (must stay exit 0),
`./gradlew :app:testDebugUnitTest --tests '*NotificationGateTest' --tests
'*NotificationHelperGateTest'`, and the on-device pass in #619 criteria 1-5 on the
device window that the capture already needs.

---

## 4. Queue state

**No PR head was re-read in this session.** The unauthenticated GitHub API budget
(60/hour) was spent on the status reads in section 2 and then returned
`API rate limit exceeded`. The dispositions below are therefore CARRIED from
`docs/audit/CLUSTER_DISPOSITIONS_2026-11-07.md` and the 2026-11-09 cluster record on
`lane/burndown` (`778c0495a`), which walked all 44 cars. They are not this session's
reads and are marked as such.

| Item | Last recorded disposition | Source | Changed this session |
|---|---|---|---|
| C1 - Batch D (#660, and #550/#554/#556/#509 as superseded) | DECISION-READY (two preconditions: rail battery, two non-author PASSes) | 11-07 cluster record | no |
| C2 - #601 / #611 / #643 / #609 | #601 CLOSED; rest DECISION-READY (CI/infra owner) | 11-07 | no |
| C4 - #506 / #508 / #507 / #510 | DECISION-READY x3, #510 PARKED-WITH-PACKET | 11-07 | no |
| C3 - #178 first | DECISION-READY (owner): head is a fork (`push:false`), iOS gate is xcodebuild-on-Mac | 11-07 | no |
| C5 - #521 / #599 | #521 DECISION-READY, #599 PARKED on #521's base | 11-07 | no |
| #208 | DECISION-READY (fork author/owner) | 11-07 | no |
| #677 port | DECISION-READY - **now narrowed to one re-fire** | this session, section 2 | yes |
| #619 | DECISION-READY - **now narrowed to the FGS card** | this session, section 3 | yes |
| Pixel capture | BLOCKED, filed once | this session, section 1 | yes (filed) |
| #489 | HELD, untouched | standing order | no read taken |

Eleven of these rows are decision-ready on an input the lane cannot supply (a
second reviewer account, a fork author's push, a CI owner's runner, or an operator
architecture call). That is the shape of the backlog, not a session failure - but it
does mean this session could not CLOSE any row, because the one act that changes what
is possible for all of them (`#677` landing) needs a re-fire this lane cannot perform.

---

## 5. Verification actually run here

| Check | Command | Result |
|---|---|---|
| Lane fence | `git cat-file -e origin/main:.circleci/config.yml` | ABSENT |
| Port carries the rail | `git cat-file -e origin/lane/burndown:.circleci/config.yml` | PRESENT |
| Port scope | `git diff --stat origin/main...origin/ci/circleci-port-main-20261108` | 1 file, +972 |
| Port ancestry | `git merge-base` / `git rev-list --count` | merge-base = main head; 3 commits |
| Rail evidence | `commits/<sha>/status` x3 | port 0, main 0, lane 12 success |
| Post-site coverage | `python3 scripts/notification_gate_audit.py` | 8 sites classified, exit 0 |
| #528 repair | `python3 scripts/check_toolchain_pin.py` | 26 decls, all 1.99.0, exit 0 (27 before) |
| #528 repair | `yaml.safe_load(.github/workflows/lint.yml)` | parses, 5 jobs, Rust Linting step ok |
| Disk guard | `python3 scripts/disk_budget.py` | TIGHT, 9.46 GB free, exit 1 |
| Jev gate | `python3 scripts/jev_canonical_check.py ...` | FAIL, Harness checkout missing, exit 1 |
| Device transport | `adb devices -l` | command not found, exit 127 |

**Not run, and why:** no `cargo` (not installed), no `java` (not installed), no Android
SDK (`ANDROID_HOME` unset), so no Rust build, no Gradle build, no Kotlin or JVM unit
test. No device proof. No Jev driving gate (no key, no Harness root). No run was
dispatched.

---

## 6. Why this session posted no receipt

Stated once, precisely, because every "receipt on #602 / #653" instruction in the
tasking depends on it:

```
$ gh auth status                -> You are not logged into any GitHub hosts.
$ $GH_TOKEN / $GITHUB_TOKEN / $GH_PAT  -> all unset
$ curl https://api.github.com/repos/Treystu/SCMessenger  -> 200 (public reads only)
$ curl https://api.github.com/repos/Treystu/slow-lane-ledger -> 404
$ curl -X POST (any comment endpoint) -> requires a token that does not exist here
```

Reads of a public repository work anonymously, up to 60 requests/hour, which this
session exhausted on the section 2 evidence. **Writes do not work at all.** The lane
protocol's channel (a status comment on #653 mirrored to #602, plus a digest commit to
the ledger) is therefore unavailable, and the ledger repo is not even readable from
this identity. Per the standing rule that GitHub beats local state, this record is
committed to the session branch instead of being asserted as posted, so the next
session with a credential can publish it verbatim rather than reconstruct it.

---

## 7. First three actions for the next session

1. Re-fire `ci/circleci-port-main-20261108` (or re-run the pipeline) and read its
   `ci/circleci:*` contexts. That is the last gate on #677.
2. Apply the #619 decision in section 3: one string plus one conditional subtitle, then
   run `scripts/notification_gate_audit.py` and the two existing gate test classes
   (`NotificationGateTest`, `NotificationHelperGateTest`) on a machine with a JDK.
3. File the stand-up in section 0 and this record's sections 1-3 on #602 and #653 from
   a session that can write.
