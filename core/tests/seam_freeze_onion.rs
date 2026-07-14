//! Seam Freeze Test: Onion Routing Construction Isolation (farm plan AD-8)
//!
//! Onion routing must compile and its own unit tests must pass in every build,
//! but stays dormant in the live send path until v1.1: exactly ONE documented,
//! config-gated call site (core/src/iron_core.rs, guarded by
//! `PrivacyConfig::onion_routing_enabled`), and that flag must default to false.
//! This test fails loudly if either invariant is broken - a new unguarded call
//! site, or the config flag flipping on by default.

use std::fs;
use std::path::Path;

#[test]
fn seam_freeze_onion_default_disabled() {
    let cfg = scmessenger_core::privacy::PrivacyConfig::default();
    assert!(
        !cfg.onion_routing_enabled,
        "SEAM FREEZE VIOLATION (AD-8): PrivacyConfig::default() must have \
         onion_routing_enabled = false. Onion routing is not yet farm-verified \
         and must stay opt-in only."
    );
}

#[test]
fn seam_freeze_onion_single_documented_call_site() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_root = Path::new(manifest_dir).join("src");

    let mut call_sites = Vec::new();
    collect_construction_call_sites(&src_root, &mut call_sites);

    // The one documented, config-gated automatic wiring point per AD-8
    // (iron_core.rs's prepare_message_internal, gated on onion_routing_enabled).
    let allowed = "iron_core.rs";
    // mobile_bridge.rs and wasm_support/rpc.rs are thin FFI/RPC pass-throughs that
    // expose IronCore's onion methods directly to platform clients, UNGATED by
    // onion_routing_enabled (that flag only wraps the internal auto-send path).
    // This is a real, tracked gap distinct from AD-8's "live send path" concern -
    // see ONION_FFI_RPC_SURFACE_UNGATED.md. Not asserted here to avoid conflating
    // two different questions in one test; do not silently widen this list further
    // without updating that ticket.
    let known_ffi_surface = ["mobile_bridge.rs", "wasm_support"];

    let unexpected: Vec<&String> = call_sites
        .iter()
        .filter(|p| {
            !p.contains(allowed)
                && !p.contains("privacy")
                && !known_ffi_surface.iter().any(|s| p.contains(s))
        })
        .collect();

    assert!(
        unexpected.is_empty(),
        "SEAM FREEZE VIOLATION (AD-8): found onion-construction call site(s) \
         outside the documented single wiring point ({allowed}):\n{}",
        unexpected
            .iter()
            .map(|p| format!("  - {p}"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    assert!(
        call_sites.iter().any(|p| p.contains(allowed)),
        "Expected the documented wiring point in {allowed} to exist and call \
         prepare_onion_message - if it was intentionally removed, update this \
         test to match the new state, do not just delete the assertion."
    );
}

fn collect_construction_call_sites(dir: &Path, sites: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if name_str.starts_with('.') || name_str == "target" {
            continue;
        }

        if path.is_dir() {
            collect_construction_call_sites(&path, sites);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            // Windows path separators normalized so substring checks (e.g. "privacy")
            // work regardless of platform.
            let path_str = path.to_string_lossy().replace('\\', "/");

            if path_str.contains("/privacy/") {
                continue;
            }

            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };

            for pattern in ["prepare_onion_message", "peel_onion_layer"] {
                // Skip the `fn <pattern>` definition line itself; only count call sites.
                for line in content.lines() {
                    if line.contains(pattern) && !line.trim_start().starts_with("pub fn") {
                        sites.push(path_str.clone());
                        break;
                    }
                }
            }
        }
    }
}
