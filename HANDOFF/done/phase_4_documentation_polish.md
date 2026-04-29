# Phase 4: Documentation & Polish

**Priority:** P2
**Assigned Agent:** worker (gemma4:31b:cloud)
**Fallback:** triage-router (gemini-3-flash-preview:cloud)
**Status:** TODO
**Depends On:** phase_3_security_hardening

## 4A: Documentation Sync
- [ ] Run `scripts/docs_sync_check.sh` — resolve all failures
- [ ] Update `DOCUMENTATION.md` index
- [ ] Update `docs/DOCUMENT_STATUS_INDEX.md` lifecycle tracking
- [ ] Update `docs/CURRENT_STATE.md` with verified architecture
- [ ] Update `REMAINING_WORK_TRACKING.md` backlog

## 4B: Code Quality
- [ ] Fix all `clippy` warnings (`cargo clippy --workspace -- -D warnings`)
- [ ] Remove or update all `_DEPRECATED` flagged functions
- [ ] Verify no `unwrap()` in production paths
- [ ] Verify all `// SAFETY:` comments on unsafe blocks
- [ ] Final `cargo fmt --all -- --check`

## Success Criteria
- `scripts/docs_sync_check.sh` passes with zero failures
- `cargo clippy` reports zero warnings
- All canonical docs reflect current state
