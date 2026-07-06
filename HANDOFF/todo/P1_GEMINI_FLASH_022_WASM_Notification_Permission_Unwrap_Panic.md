# P2_WASM_Notification_Permission_JS_Interop_Unwrap_Panic

**Priority:** P2
**Platform:** WASM
**Status:** TODO
**Source:** native sweep 2026-07-04

## Problem

`wasm/src/notification_manager.rs` has two `.unwrap()` calls on the result of
`js_sys::Reflect::apply(...)` inside the notification-permission-request path
(not test code — this file has no `#[cfg(test)]` module at all, confirmed via
grep):

- `notification_manager.rs:150` (inside the `navigator.permission()` lookup
  branch):
  ```rust
  let promise = js_sys::Promise::from(
      js_sys::Reflect::apply(
          &req_fn,
          &JsValue::UNDEFINED,
          &js_sys::Array::new(),
      )
      .unwrap(),
  );
  ```
- `notification_manager.rs:190` (the `window.Notification.requestPermission()`
  fallback branch, same pattern):
  ```rust
  let promise = js_sys::Promise::from(
      js_sys::Reflect::apply(
          &request_fn,
          &JsValue::UNDEFINED,
          &js_sys::Array::new(),
      )
      .unwrap(),
  );
  ```

`Reflect::apply` returns `Result<JsValue, JsValue>` and genuinely can return
`Err` at runtime — e.g. if the page's Permissions Policy / Feature Policy
blocks notification APIs in an iframe, if a browser extension or CSP
intercepts the call, or if `Notification.requestPermission` throws
synchronously in some browser/embedding edge case. Since this is a
browser-facing WASM module (per CLAUDE.md: "browser thin-client... panics
there are bad UX (crash the whole WASM module)"), an `.unwrap()` panic here
crashes the entire WASM instance rather than just failing the notification
permission request — the rest of the mesh client (messaging, transport)
would go down with it if they share the same WASM module instance.

Not covered by any existing `HANDOFF/todo/*.md` (grepped for
`notification_manager`, `Reflect::apply`, `requestPermission`) and distinct
from the `HANDOFF/DEAD_CODE_TRIAGE_RESULTS.md` finding on this same file
(`save_notification_settings` at line 477, a different, unrelated "reserved
helper" finding that this task does not re-litigate).

## Fix Plan

Replace both `.unwrap()` calls with the same graceful-fallback pattern already
used a few lines below each (`if let Ok(permission_val) = js_future.await { ... }`
silently no-ops on error) — i.e. treat a synchronous `Reflect::apply` failure
the same as "this permission path isn't available," and fall through to the
next fallback branch (the `navigator.permission()` branch already falls
through to the `Notification.requestPermission()` branch on any failure; the
final fallback branch should fall through to returning `false`/`Default`
rather than panicking):

```rust
let apply_result = js_sys::Reflect::apply(
    &req_fn,
    &JsValue::UNDEFINED,
    &js_sys::Array::new(),
);
let Ok(promise_value) = apply_result else {
    // Synchronous failure calling into JS (e.g. blocked by Permissions
    // Policy) — fall through instead of panicking the whole WASM module.
    return false; // or `continue`/fall-through to next branch, matching
                  // the existing control flow at each call site
};
let promise = js_sys::Promise::from(promise_value);
```

Exact fall-through target differs slightly between the two call sites (first
one should proceed to the `Notification.requestPermission()` fallback below
it; second one is the last fallback, so should set
`NotificationPermission::Default` and return `false`) — match the existing
`None`/`Err` handling already present a few lines above each call site in the
same function for consistency.

## Files to Touch

- `wasm/src/notification_manager.rs` [EDIT] — lines ~140-155 and ~180-195

## Verification

```bash
cargo check -p scmessenger-wasm --target wasm32-unknown-unknown
```
Manual: no unit test harness currently exists for this browser-only path
(would need `wasm-pack test --headless --firefox` with a mocked/blocked
Permissions Policy to reproduce the `Err` branch — out of scope for this fix,
note as a possible follow-up).

## Acceptance Criteria

- Neither call site can panic the WASM module if `Reflect::apply` returns
  `Err`.
- Behavior on the `Err` path degrades to the same "permission not granted"
  outcome the function already produces on other failure branches (no new
  panics, no silent success).
- `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` passes.
