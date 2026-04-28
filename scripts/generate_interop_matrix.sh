#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UDL_FILE="$ROOT_DIR/core/src/api.udl"
OUT_FILE="${1:-$ROOT_DIR/docs/INTEROP_MATRIX_V0.2.0_ALPHA.md}"

ANDROID_FILES=(
  "$ROOT_DIR/android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt"
  "$ROOT_DIR/android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt"
  "$ROOT_DIR/android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt"
)

IOS_FILES=(
  "$ROOT_DIR/iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift"
  "$ROOT_DIR/iOS/SCMessenger/SCMessenger/Services/CoreDelegateImpl.swift"
  "$ROOT_DIR/iOS/SCMessenger/SCMessenger/Transport/MultipeerTransport.swift"
)

WASM_FILES=(
  "$ROOT_DIR/wasm/src/lib.rs"
)

CLI_FILES=(
  "$ROOT_DIR/cli/src/main.rs"
  "$ROOT_DIR/cli/src/api.rs"
)

snake_to_camel() {
  local input="$1"
  local result=""
  local first=1
  local part=""
  local first_char=""
  local rest_chars=""
  local IFS="_"
  read -r -a parts <<< "$input"
  for part in "${parts[@]}"; do
    if [ "$first" -eq 1 ]; then
      result+="$part"
      first=0
    else
      first_char="$(printf "%s" "$part" | cut -c1 | tr '[:lower:]' '[:upper:]')"
      rest_chars="$(printf "%s" "$part" | cut -c2-)"
      result+="${first_char}${rest_chars}"
    fi
  done
  printf "%s" "$result"
}

count_hits() {
  local method="$1"
  shift
  local camel
  local matches
  local count
  camel="$(snake_to_camel "$method")"
  matches="$(rg -n -S "\\b(${method}|${camel})\\s*\\(" "$@" 2>/dev/null || true)"
  if [ -z "$matches" ]; then
    echo 0
    return
  fi
  count="$(printf "%s\n" "$matches" | wc -l | tr -d ' ')"
  echo "$count"
}

is_required_method() {
  case "$1" in
    initialize_identity|get_identity_info|set_nickname|export_identity_backup|import_identity_backup|prepare_message_with_id|prepare_receipt|mark_message_sent|add|get|remove|list|set_local_nickname|recent|conversation|clear|stats|get_peers|get_listeners|get_external_addresses|export_diagnostics|get_connection_path_state)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

intentional_note() {
  case "$1" in
    IronCore.set_delegate)
      echo "Core callback registration; consumed indirectly through platform delegates."
      return 0
      ;;
    IronCore.prepare_message)
      echo "Legacy prepare helper; adapters use prepare_message_with_id for deterministic ID tracking."
      return 0
      ;;
    IronCore.contacts_manager|IronCore.history_manager)
      echo "Manager accessors are consumed internally during repository bootstrapping rather than direct adapter calls."
      return 0
      ;;
    MeshService.set_platform_bridge)
      echo "Platform bridge wiring is platform-specific by design."
      return 0
      ;;
    MeshService.on_peer_discovered|MeshService.on_peer_disconnected|MeshService.on_data_received)
      echo "Ingress callback methods are adapter-invoked, not user-facing API calls."
      return 0
      ;;
    MeshService.start_swarm)
      echo "Mobile starts swarm internally during service startup; direct calls are platform-specific."
      return 0
      ;;
    SwarmBridge.shutdown)
      echo "Shutdown is typically lifecycle-owned by service stop paths."
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

has_all_methods_for_platform() {
  local platform="$1"
  local methods_csv="$2"
  local method=""
  local hits=0
  local missing=()
  local IFS=","
  read -r -a methods <<< "$methods_csv"
  for method in "${methods[@]}"; do
    case "$platform" in
      android)
        hits="$(count_hits "$method" "${ANDROID_FILES[@]}")"
        ;;
      ios)
        hits="$(count_hits "$method" "${IOS_FILES[@]}")"
        ;;
      wasm)
        hits="$(count_hits "$method" "${WASM_FILES[@]}")"
        ;;
      cli)
        hits="$(count_hits "$method" "${CLI_FILES[@]}")"
        ;;
      *)
        echo "Gap (unknown platform)"
        return
        ;;
    esac
    if [ "$hits" -eq 0 ]; then
      missing+=("$method")
    fi
  done

  if [ "${#missing[@]}" -eq 0 ]; then
    echo "Covered"
  else
    echo "Gap ($(IFS=", "; echo "${missing[*]}"))"
  fi
}

role_mode_status() {
  local platform="$1"
  local pattern=""
  local files=()
  local hits=""
  case "$platform" in
    android)
      pattern="isMeshParticipationEnabled|requireMeshParticipationEnabled|relay_enabled"
      files=("${ANDROID_FILES[@]}")
      ;;
    ios)
      pattern="isMeshParticipationEnabled|relayOnly|relay_enabled"
      files=("${IOS_FILES[@]}")
      ;;
    wasm)
      pattern="ensure_mesh_participation_enabled|relay_enabled"
      files=("${WASM_FILES[@]}")
      ;;
    cli)
      pattern="Commands::Relay|relay"
      files=("${CLI_FILES[@]}")
      ;;
    *)
      echo "Gap (unknown platform)"
      return
      ;;
  esac
  hits="$(rg -n -S "$pattern" "${files[@]}" 2>/dev/null || true)"
  if [ -n "$hits" ]; then
    echo "Covered"
  else
    echo "Gap (role-mode gates not found)"
  fi
}

interface_methods() {
  awk '
    BEGIN { inside=0; iface="" }
    /^interface (IronCore|MeshService|ContactManager|HistoryManager|LedgerManager|SwarmBridge)[[:space:]]*\{/ {
      inside=1
      iface=$2
      sub(/\{/, "", iface)
      next
    }
    inside && /^};/ {
      inside=0
      next
    }
    inside && /\(/ {
      line=$0
      gsub(/\[Throws=IronCoreError\]/, "", line)
      gsub(/\[Name=[^]]+\]/, "", line)
      if (line ~ /constructor\(/) next
      sub(/\/\/.*/, "", line)
      gsub(/^[ \t]+|[ \t]+$/, "", line)
      split(line, a, "(")
      sig=a[1]
      gsub(/^[ \t]+|[ \t]+$/, "", sig)
      n=split(sig, t, /[ \t]+/)
      method=t[n]
      if (method != "") {
        print iface "|" method
      }
    }
  ' "$UDL_FILE"
}

mkdir -p "$(dirname "$OUT_FILE")"

generated_at="$(date -u +"%Y-%m-%d %H:%M:%SZ")"
generated_date_utc="$(date -u +"%Y-%m-%d")"

{
  echo "# SCMessenger v0.2.0 Alpha Interoperability Matrix"
  echo
  echo "Status: Active  "
  echo "Last updated: ${generated_date_utc}  "
  echo "Generated by: \`scripts/generate_interop_matrix.sh\` at \`$generated_at\`"
  echo
  echo "## Scope"
  echo
  echo "- Surface + path matrix first, with pairwise deep dives only for detected gaps."
  echo "- Interfaces covered: \`IronCore\`, \`MeshService\`, \`ContactManager\`, \`HistoryManager\`, \`LedgerManager\`, \`SwarmBridge\`."
  echo "- Platform files scanned: Android repository/service, iOS repository/delegate/transport, WASM bindings, CLI/relay entrypoints."
  echo
  echo "## Status Legend"
  echo
  echo "- \`Implemented + Consumed\`: interface method is implemented in Core and referenced by platform adapters."
  echo "- \`Implemented + Platform-Specific (Intentional)\`: method is implementation-only or lifecycle/bridge specific by design."
  echo "- \`Implemented + Not Consumed (Gap)\`: method exists but no adapter consumption was found where parity is expected."
  echo "- \`Missing/Drift\`: expected contract behavior was not found in adapter scans."
  echo
  echo "## Surface + Path Matrix (Alpha Gate)"
  echo
  echo "| Capability | Android (full/relay-only) | iOS (full/relay-only) | WASM/Desktop (full/relay-only) | CLI/Relay node | Pairwise Deep Dive Needed |"
  echo "| --- | --- | --- | --- | --- | --- |"

  row_methods_identity="initialize_identity,get_identity_info,set_nickname,export_identity_backup,import_identity_backup"
  row_methods_message="prepare_message_with_id,prepare_receipt,mark_message_sent,send_message"
  row_methods_contacts="add,get,remove,list,set_nickname,set_local_nickname"
  row_methods_history="recent,conversation,clear,stats"
  row_methods_swarm="get_peers,get_listeners,get_external_addresses,export_diagnostics,get_connection_path_state"

  identity_android="$(has_all_methods_for_platform android "$row_methods_identity")"
  identity_ios="$(has_all_methods_for_platform ios "$row_methods_identity")"
  identity_wasm="$(has_all_methods_for_platform wasm "$row_methods_identity")"
  identity_cli="$(has_all_methods_for_platform cli "$row_methods_identity")"
  identity_deep="No"
  [[ "$identity_android$identity_ios$identity_wasm$identity_cli" == *"Gap"* ]] && identity_deep="Yes"
  echo "| Identity init/info/nickname/backup | $identity_android | $identity_ios | $identity_wasm | $identity_cli | $identity_deep |"

  msg_android="$(has_all_methods_for_platform android "$row_methods_message")"
  msg_ios="$(has_all_methods_for_platform ios "$row_methods_message")"
  msg_wasm="$(has_all_methods_for_platform wasm "$row_methods_message")"
  msg_cli="$(has_all_methods_for_platform cli "$row_methods_message")"
  msg_deep="No"
  [[ "$msg_android$msg_ios$msg_wasm$msg_cli" == *"Gap"* ]] && msg_deep="Yes"
  echo "| Message prepare/send/receipt/mark-sent | $msg_android | $msg_ios | $msg_wasm | $msg_cli | $msg_deep |"

  contacts_android="$(has_all_methods_for_platform android "$row_methods_contacts")"
  contacts_ios="$(has_all_methods_for_platform ios "$row_methods_contacts")"
  contacts_wasm="$(has_all_methods_for_platform wasm "$row_methods_contacts")"
  contacts_cli="$(has_all_methods_for_platform cli "$row_methods_contacts")"
  contacts_deep="No"
  [[ "$contacts_android$contacts_ios$contacts_wasm$contacts_cli" == *"Gap"* ]] && contacts_deep="Yes"
  echo "| Contacts CRUD + nickname overrides | $contacts_android | $contacts_ios | $contacts_wasm | $contacts_cli | $contacts_deep |"

  history_android="$(has_all_methods_for_platform android "$row_methods_history")"
  history_ios="$(has_all_methods_for_platform ios "$row_methods_history")"
  history_wasm="$(has_all_methods_for_platform wasm "$row_methods_history")"
  history_cli="$(has_all_methods_for_platform cli "$row_methods_history")"
  history_deep="No"
  [[ "$history_android$history_ios$history_wasm$history_cli" == *"Gap"* ]] && history_deep="Yes"
  echo "| History recent/conversation/clear/stats | $history_android | $history_ios | $history_wasm | $history_cli | $history_deep |"

  swarm_android="$(has_all_methods_for_platform android "$row_methods_swarm")"
  swarm_ios="$(has_all_methods_for_platform ios "$row_methods_swarm")"
  swarm_wasm="$(has_all_methods_for_platform wasm "$row_methods_swarm")"
  swarm_cli="$(has_all_methods_for_platform cli "$row_methods_swarm")"
  swarm_deep="No"
  [[ "$swarm_android$swarm_ios$swarm_wasm$swarm_cli" == *"Gap"* ]] && swarm_deep="Yes"
  echo "| Swarm peer/listener/external visibility + diagnostics | $swarm_android | $swarm_ios | $swarm_wasm | $swarm_cli | $swarm_deep |"

  role_android="$(role_mode_status android)"
  role_ios="$(role_mode_status ios)"
  role_wasm="$(role_mode_status wasm)"
  role_cli="$(role_mode_status cli)"
  role_deep="No"
  [[ "$role_android$role_ios$role_wasm$role_cli" == *"Gap"* ]] && role_deep="Yes"
  echo "| Role-mode gating behavior | $role_android | $role_ios | $role_wasm | $role_cli | $role_deep |"

  echo
  echo "## Function Completeness Audit"
  echo

  current_iface=""
  gap_count=0
  deep_dive_rows=0
  while IFS='|' read -r iface method; do
    if [ "$iface" != "$current_iface" ]; then
      if [ -n "$current_iface" ]; then
        echo
      fi
      current_iface="$iface"
      echo "### $iface"
      echo
      echo "| Method | Android refs | iOS refs | WASM refs | CLI refs | Status | Notes |"
      echo "| --- | --- | --- | --- | --- | --- | --- |"
    fi

    a_hits="$(count_hits "$method" "${ANDROID_FILES[@]}")"
    i_hits="$(count_hits "$method" "${IOS_FILES[@]}")"
    w_hits="$(count_hits "$method" "${WASM_FILES[@]}")"
    c_hits="$(count_hits "$method" "${CLI_FILES[@]}")"
    total_hits=$((a_hits + i_hits + w_hits + c_hits))

    key="${iface}.${method}"
    status=""
    notes=""

    if notes="$(intentional_note "$key")"; then
      status="Implemented + Platform-Specific (Intentional)"
    elif [ "$total_hits" -eq 0 ]; then
      status="Implemented + Not Consumed (Gap)"
      notes="No adapter references found in scanned platform files."
      gap_count=$((gap_count + 1))
    elif is_required_method "$method"; then
      missing=()
      [ "$a_hits" -eq 0 ] && missing+=("Android")
      [ "$i_hits" -eq 0 ] && missing+=("iOS")
      [ "$w_hits" -eq 0 ] && missing+=("WASM/Desktop")
      if [ "${#missing[@]}" -gt 0 ]; then
        status="Implemented + Not Consumed (Gap)"
        notes="Required alpha-parity adapters missing: $(IFS=", "; echo "${missing[*]}")."
        gap_count=$((gap_count + 1))
      else
        status="Implemented + Consumed"
        notes="Required parity method is adapter-referenced across Android/iOS/WASM."
      fi
    else
      status="Implemented + Consumed"
      notes="Adapter references detected."
    fi

    echo "| \`$method\` | $a_hits | $i_hits | $w_hits | $c_hits | $status | $notes |"
  done < <(interface_methods)

  echo
  echo "## Gap Triage"
  echo
  if [ "$gap_count" -eq 0 ]; then
    echo "- No completeness gaps detected by this static adapter-reference scan."
  else
    echo "- Detected gap count: **$gap_count**."
    echo "- Required deep-dive pairs to execute for remaining gaps:"
    echo "  - Core -> Android"
    echo "  - Core -> iOS"
    echo "  - Core -> WASM/Desktop"
    echo "  - Android <-> iOS (direct/relay/BLE-only continuity)"
    deep_dive_rows=1
  fi

  echo
  echo "## Notes"
  echo
  echo "- This matrix is deterministic static-analysis evidence (method-reference scan) and not a replacement for runtime scenario testing."
  echo "- Runtime scenario execution remains tracked in the alpha validation scripts under \`scripts/\`."
} > "$OUT_FILE"

echo "Wrote interoperability matrix to $OUT_FILE"
