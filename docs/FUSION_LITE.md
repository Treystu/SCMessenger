# fusion_lite.py -- Cost-Bounded Multi-Model Planning/Verification

Status: Active
Added: 2026-07-11

## What this is

`scripts/fusion_lite.py` is a hand-rolled replacement for OpenRouter's
"Fusion" feature (`model: "openrouter/fusion"`, the `fusion` plugin, or the
`openrouter:fusion` server tool). It dispatches a tightly-scoped prompt to
2-4 independent cheap/small models in parallel (the "panel"), then feeds
all their responses to one more model that synthesizes agreement,
disagreement, and a final verdict (the "judge"). This is the same pattern
Fusion itself uses -- the difference is that this script gives you real
control over cost, while OpenRouter's own Fusion does not.

Use it as an available planning/verification path for narrow, well-scoped
questions -- a second (or third) opinion on an implementation approach, or
a sanity check on a small piece of code -- when the question can be fully
answered from pasted context, without web access or file access.

## Why this exists instead of using OpenRouter's Fusion directly

Fusion was evaluated for this purpose on 2026-07-11 and rejected. Findings,
in order:

1. **All-`:free` panels don't work.** Four different configurations
   (plugin form, server-tool form, forced `tool_choice`, mixed free/paid
   judge) were tested with a panel of `:free`-suffixed models. All four
   failed -- HTTP 404, HTTP 500, HTTP 400 with internal retries, or a
   silent fallback where the judge model answered solo after quietly
   failing to invoke the panel at all (visible only in the response's
   internal `reasoning` field, which said "despite tool failure"). Even
   with a fully **paid** judge model and a `:free` panel, the request
   still failed (400, retried twice, still failed) -- this isolates the
   break to the panel dispatch itself, not judge cost or the invocation
   mechanism.

2. **No way to disable Fusion's forced web tools.** Every Fusion panel
   call automatically gets `openrouter:web_search` and
   `openrouter:web_fetch` enabled, with no documented flag to turn this
   off per panel member. A real planning request -- fully self-contained,
   no web lookup needed -- was projected to cost under a tenth of a cent
   based on token counts alone. The actual cost came back at **$0.057**,
   roughly 730x the estimate, because panel models invoked web search
   without any visible way to prevent it. The `fusion` plugin config
   surface only exposes `analysis_models` and `model` (the judge slug) --
   nothing for tool suppression.

3. **Conclusion:** Fusion is not controllable enough for a cost-bounded
   loop. The fix was to stop using Fusion's plugin/router mechanism
   entirely and make plain chat completions by hand, with no `tools` key
   in any payload -- see "How it works" below.

## How it works

For each panel model, `fusion_lite.py` sends a standard
`/chat/completions` request:

```json
{
  "model": "<panel model slug>",
  "messages": [{"role": "user", "content": "<your prompt>"}],
  "max_tokens": 300
}
```

Note there is no `"tools"` key. Nothing is available for the model to
invoke, so nothing can be invoked -- this is the entire fix. All N panel
responses are then folded into one more call to the judge model, asking it
to summarize agreement/disagreement and give a final verdict.

Because no payload ever contains a `tools` key, worst-case cost is purely
a function of `prompt_tokens * prompt_price + max_tokens * completion_price`
for each of the N+1 calls -- a number the script can compute exactly before
sending anything, and it is a *true* ceiling (not an estimate with a
hidden variable), because there is no tool-invocation cost that could push
actual spend above it.

## Hard guarantees (refusals, not warnings)

The script enforces six things before or during every run. All are
`sys.exit(1)` refusals -- there is no "proceed with a warning" path for
any of them:

1. **No `tools` key ever.** Asserted on every payload right before it's
   sent.
2. **Pre-flight cost ceiling.** Total worst-case cost across all N+1 calls
   (every call maxing its `--max-tokens`) is computed before any network
   call and compared to `--max-cost` (default `$0.01`). If the estimate
   exceeds the ceiling, the script exits before spending anything.
   IMPORTANT implementation detail if you ever touch this code:
   OpenRouter's `/api/v1/models` pricing fields (`pricing.prompt`,
   `pricing.completion`) are already **per-token** dollar prices (e.g.
   `"0.00000001"` means $0.01 per million tokens) -- do not divide by
   `1e6` again. An earlier draft of this script did exactly that and
   silently undercounted cost by roughly six orders of magnitude
   (estimate ~$0.0000000001 against an actual cost of ~$0.0001). The
   pre-flight refusal was completely inert during that period; only
   guarantee #5 below (which checks real returned cost, not the estimate)
   was doing any protective work. Fixed and reverified 2026-07-11 --
   corrected estimates now land slightly *above* actual cost (~1.1-1.2x),
   which is the intended relationship for a worst-case ceiling.
3. **BYOK hard-block.** Some OpenRouter accounts have Bring-Your-Own-Key
   provider credentials configured (visible at
   `openrouter.ai/settings/plugins`). Calls routed through a BYOK
   credential show `usage.cost: 0` in the API response while the real
   spend appears separately as `usage.cost_details.upstream_inference_cost`
   -- outside the tracked API key's balance entirely (confirmed against
   `GET /api/v1/key`, which never reflected this spend). A script that
   only watches the key's balance would have a real, invisible cost leak.
   `fusion_lite.py` hard-blocks any model whose org-prefix is listed in
   `BYOK_DENYLIST_PREFIXES` at the top of the script (currently just
   `mistralai/`, the one confirmed case as of 2026-07-11). If you add a
   BYOK credential for a new provider, or discover another provider routes
   this way on this account, add its prefix to that list. If you remove a
   BYOK credential, you can remove the corresponding prefix.
4. **Key must have a finite limit.** On startup the script calls
   `GET /api/v1/key` and refuses to run if the key's `limit` field is
   `null` (uncapped). This makes the whole cost-ceiling guarantee
   self-enforcing rather than trust-based -- there's no way to
   accidentally point this tool at a key with no spend cap at all.
5. **Mid-batch fail-closed.** After every call, actual cumulative cost is
   checked against `--max-cost`. If it's somehow already exceeded (the
   pre-flight estimate proved wrong, or a model's pricing changed between
   the pricing lookup and the call), the script aborts immediately rather
   than continuing to the judge step. This is independent of guarantee #2
   and exists specifically so an error in the pre-flight math (like the
   1e6 bug above) doesn't silently defeat the whole guarantee.
6. **Key identity check.** The script only ever reads whichever key
   `OPENROUTER_API_KEY` resolves to (env var first, then
   `~/.config/scmorc/openrouter.env`) -- by itself, that's a name lookup
   with no verification that it's the *intended* credential. Every run
   prints the live key's masked label (e.g. `sk-or-v1-4d4...5c3` --
   OpenRouter's `/api/v1/key` endpoint never returns full key values, only
   this masked form) to stderr, so whoever is watching a run can eyeball
   it. If you pass `--expect-key-label <substring>` (or set the
   `FUSION_LITE_EXPECT_KEY_LABEL` env var), the script refuses to run
   unless the live key's label contains that substring -- turning a
   silent wrong-key run into a loud, immediate failure. Added 2026-07-11
   after a review caught that nothing previously verified key identity,
   only that *some* key with *some* finite limit was present.

## Setting up the key this tool expects

`fusion_lite.py` and `scripts/delegate_task.py` share the same key
convention: `~/.config/scmorc/openrouter.env`, containing a single line
`OPENROUTER_API_KEY=sk-or-v1-...`. This file lives outside the git repo
tree entirely (not `.gitignore`d -- it's just never inside `SCMessenger/`
in the first place), so it is never at risk of being committed.

Before using this tool for real work, set a spend limit on the key at
`https://openrouter.ai/settings/keys` (guarantee #4 refuses to run against
an unlimited key regardless, but set one deliberately rather than relying
on that refusal as the only check). To pin down key identity so a future
session can't silently run against a different key, either:

- Export `FUSION_LITE_EXPECT_KEY_LABEL` in your shell/orchestrator
  environment to a substring of the intended key's masked label (check it
  via `curl -s https://openrouter.ai/api/v1/key -H "Authorization: Bearer
  $OPENROUTER_API_KEY"` and read the `label` field), or
- Pass `--expect-key-label <substring>` on each invocation.

Neither of these ever needs (or should have) the actual key value in it --
only the masked label substring, which is safe to put in an orchestrator
prompt, a doc, or a shell profile.

## Usage

```bash
python3 scripts/fusion_lite.py \
  --prompt-file /path/to/tightly_scoped_prompt.txt \
  --panel "inclusionai/ling-2.6-flash,meta-llama/llama-3.1-8b-instruct,ibm-granite/granite-4.1-8b" \
  --judge "inclusionai/ling-2.6-flash" \
  --max-tokens 300 \
  --max-cost 0.01 \
  --expect-key-label "<substring of your key's masked label>" \
  --out /tmp/result.json
```

| Flag | Required | Default | Notes |
|---|---|---|---|
| `--prompt-file` | yes | -- | Must be self-contained. See "Writing a good prompt" below. |
| `--panel` | yes | -- | Comma-separated, 2-4 model slugs. |
| `--judge` | yes | -- | One model slug for synthesis. Can repeat a panel model. |
| `--max-tokens` | no | 300 | Hard cap per call, panel AND judge. Lower = cheaper but more truncation risk. |
| `--max-cost` | no | 0.01 | Refuses to run if worst-case estimate exceeds this. |
| `--expect-key-label` | no | none (no identity check) | Substring of the expected key's masked label. Falls back to `FUSION_LITE_EXPECT_KEY_LABEL` env var. |
| `--out` | no | stdout | Where to write the full JSON result. |

Key source: `~/.config/scmorc/openrouter.env` (`OPENROUTER_API_KEY=...`),
same convention as `scripts/delegate_task.py`. Falls back to the
`OPENROUTER_API_KEY` environment variable if set. Never pass a key as a
CLI argument (shell history exposure).

### Writing a good prompt

The prompt file should paste in exactly the context the panel needs --
relevant code snippets, the specific rule or constraint at play, and a
narrow question -- rather than pointing at a file and hoping the model can
"go read it" (it can't; there's no file or web tool available by design).
A good prompt is a few hundred words: existing pattern + what's missing +
the specific question. See the git history of this file's introduction
for a real example (the `mobile_bridge.rs` async-FFI blocking-counterpart
question) if one is still present in `tmp/` or `HANDOFF/`.

## When to reach for this vs. other lanes

| Situation | Use |
|---|---|
| Narrow, self-contained planning/verification question; want 2-3 independent takes + synthesis; cost must stay near-zero | `fusion_lite.py` |
| Actual implementation work (writing/editing files) | `scripts/delegate_task.py` (Qwen/OpenRouter/Ollama lanes, per `docs/ORCHESTRATION.md` Section 3) |
| Crypto/transport/routing/privacy code -- mandatory adversarial review | `crypto-security-auditor` subagent (native) or the read-only Qwen thinking dispatch (per `docs/ORCHESTRATION.md` Section 4, `[AUDIT-GATE]`) -- `fusion_lite.py` is NOT a substitute for this gate, regardless of how the panel answers |
| Need real tool use (web search, file access) as part of the answer | Not this tool -- by design, panel/judge calls here never get tools. Use a Claude subagent or a Qwen dispatch with the appropriate context instead |
| Large-context job, more than a handful of KB of code | Not this tool -- keep prompts small; use `delegate_task.py` |

`fusion_lite.py` output is a second opinion / planning aid, not a
verdict. Treat disagreement between panel models as a signal to think
harder about the question, not as noise to average away -- the judge step
is instructed to preserve and report disagreement, not paper over it.

## Known limitations

- **Small/cheap models only.** The panel and judge are typically
  small-to-medium models chosen for near-zero per-token cost. Quality is
  bounded accordingly -- this is a cheap sanity-check tool, not a
  substitute for `crypto-security-auditor` or a real spec review on
  anything with real stakes.
- **Truncation at low `--max-tokens`.** The default (300) is enough for a
  short plan or verdict but can cut off a longer code snippet mid-line.
  The script flags this (`finish_reason=length` warning to stderr, and
  `"truncated": true` in the JSON output) rather than hiding it, and
  passes the flag through to the judge so it knows a given panel response
  may be incomplete -- but raising `--max-tokens` raises cost
  proportionally, so balance against `--max-cost`.
- **BYOK denylist is a manually maintained list, not a live check.**
  There is no API call that tells this script in advance which providers
  route via BYOK on a given account -- `BYOK_DENYLIST_PREFIXES` at the top
  of the script must be updated by a human when BYOK credentials change.
  The script also re-checks every actual response for `is_byok: true` as
  a backstop (and aborts immediately if it ever fires despite passing the
  pre-call denylist check), but that backstop fires *after* a call has
  already happened -- the denylist is the real prevention mechanism.
- **Pricing is fetched live on every run**, which means a brief added
  latency (one extra `GET /api/v1/models` call) but also means the cost
  ceiling always reflects current prices rather than a value that could
  go stale in this document.
- **`--expect-key-label` is a label-substring check, not a full key
  comparison.** It reads OpenRouter's masked label (e.g.
  `sk-or-v1-4d4...5c3`) via `GET /api/v1/key`, never the full key value.
  This is deliberate -- the script should never need to see or compare a
  complete key value beyond the one it's already using to authenticate --
  but it means a sufficiently short or generic `--expect-key-label` value
  could theoretically match more than one key's masked label. Use a
  distinctive-enough substring (the full trailing segment, e.g. `4d4...5c3`,
  rather than something like `sk-or-v1` which matches every key).

## Incident log

- **2026-07-11:** Fusion (OpenRouter's own feature) tested for an
  all-free-model triple-verification use case. All-`:free` panels fail
  outright (see "Why this exists" above). A subsequent attempt at a real
  paid-model planning call through Fusion's plugin mechanism cost $0.057
  against a sub-cent estimate, due to forced, uncontrollable web-tool
  invocation on panel members. `fusion_lite.py` was built same-day as a
  replacement, hand-rolling the panel+judge pattern without Fusion's
  plugin/router layer. An early draft of the cost-estimate formula had a
  units bug (treated already-per-token OpenRouter pricing as per-million,
  dividing by `1e6` a second time), undercounting worst-case cost by
  ~1,000,000x -- caught during the test pass before the script was copied
  into `scripts/`, fixed, and reverified with a live run landing at
  $0.000085 actual against a $0.000103 estimate (a genuine, working
  ceiling) on a 3-panel-plus-judge real planning question about
  `core/src/mobile_bridge.rs`'s async FFI blocking-helper pattern.
- **2026-07-11 (later same day):** A follow-up review found the script
  had no key-identity check -- it would accept and run against *any*
  OpenRouter key that had a finite spend limit, with no way to confirm it
  was the *intended* credential. Also surfaced: a cost-reporting error in
  an earlier summary conflated the daily-reset `usage_daily` figure with
  total session spend, understating the real total (lifetime `usage` on
  the tracked key was $0.0578, plus $0.00081 routed via BYOK outside the
  tracked balance entirely -- both real numbers, not the smaller figure
  originally reported). Added the `--expect-key-label` /
  `FUSION_LITE_EXPECT_KEY_LABEL` guarantee (#6 above) in response, and
  corrected the reporting going forward to always read `usage` (lifetime)
  rather than `usage_daily` when stating total spend.
