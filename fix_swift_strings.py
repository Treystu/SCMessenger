#!/usr/bin/env python3

import re
import sys

def fix_swift_file(file_path):
    """Fix Swift string interpolation issues in MeshRepository.swift"""
    
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Pattern to find logDeliveryAttempt calls with problematic string interpolation
    # This pattern matches the detail parameter with complex interpolation
    pattern = r'detail:\s*"ctx=\(attemptContext\s*\?\?\s*""\).*?"'
    
    def fix_detail_match(match):
        """Fix a single detail parameter match"""
        detail_string = match.group(0)
        
        # Extract the variable parts
        if 'route_candidates=' in detail_string:
            # Pattern: "ctx=(attemptContext ?? ""), route_candidates=\{(routePeerCandidates.count)}"
            var_name = detail_string.split('route_candidates=')[1].split(')')[0] + ')'
            return f'detail: "ctx=" + (attemptContext ?? "") + ", route_candidates=" + String({var_name})'
        elif 'reason=strict_ble_only_mode' in detail_string:
            return 'detail: "ctx=" + (attemptContext ?? "") + ", reason=strict_ble_only_mode"'
        elif 'target=' in detail_string and 'reason=ble_peer_missing_connected_device_available' in detail_string:
            # Pattern: "ctx=(attemptContext ?? ""), target=\{(fallbackPeer) reason=ble_peer_missing_connected_device_available"
            var_name = detail_string.split('target=')[1].split(')')[0] + ')'
            return f'detail: "ctx=" + (attemptContext ?? "") + ", target=" + String({var_name}) + " reason=ble_peer_missing_connected_device_available"'
        elif 'target=' in detail_string and 'requested_target=' in detail_string:
            # Pattern: "ctx=(attemptContext ?? ""), target=\{(effectiveBlePeerId) requested_target=\{(requestedBlePeerId) reason=prefer_connected_device"
            parts = detail_string.split('target=')
            target_var = parts[1].split(')')[0] + ')'
            requested_var = detail_string.split('requested_target=')[1].split(')')[0] + ')'
            return f'detail: "ctx=" + (attemptContext ?? "") + ", target=" + String({target_var}) + " requested_target=" + String({requested_var}) + " reason=prefer_connected_device"'
        elif 'role=peripheral' in detail_string:
            # Pattern: "ctx=(attemptContext ?? ""), role=peripheral requested_target=\{(bleAddr) target=\{(target)}"
            ble_var = detail_string.split('requested_target=')[1].split(')')[0] + ')'
            target_var = detail_string.split('requested_target=')[1].split(' target=')[1].split(')')[0] + ')'
            return f'detail: "ctx=" + (attemptContext ?? "") + ", role=peripheral requested_target=" + String({ble_var}) + " target=" + String({target_var})'
        elif 'role=central' in detail_string:
            # Pattern: "ctx=(attemptContext ?? ""), role=central requested_target=\{(bleAddr) target=\{(target)}"
            ble_var = detail_string.split('requested_target=')[1].split(')')[0] + ')'
            target_var = detail_string.split('requested_target=')[1].split(' target=')[1].split(')')[0] + ')'
            return f'detail: "ctx=" + (attemptContext ?? "") + ", role=central requested_target=" + String({ble_var}) + " target=" + String({target_var})'
        elif 'requested_target=' in detail_string and 'reason=' in detail_string and 'connected=' in detail_string:
            # Pattern: "ctx=(attemptContext ?? ""), requested_target=\{(bleAddr) reason=\{(lastFailureReason) connected=\{(connectedBlePeerIds.count)}"
            ble_var = detail_string.split('requested_target=')[1].split(')')[0] + ')'
            reason_var = detail_string.split('reason=')[1].split(')')[0] + ')'
            connected_var = detail_string.split('connected=')[1].split(')')[0] + ')'
            return f'detail: "ctx=" + (attemptContext ?? "") + ", requested_target=" + String({ble_var}) + " reason=" + String({reason_var}) + " connected=" + String({connected_var})'
        elif 'reason=swarm_bridge_unavailable' in detail_string:
            return 'detail: "ctx=" + (attemptContext ?? "") + ", reason=swarm_bridge_unavailable"'
        elif 'route=' in detail_string and 'reason=' not in detail_string:
            # Pattern: "ctx=(attemptContext ?? ""), route=\{(routePeerId)}"
            var_name = detail_string.split('route=')[1].split(')')[0] + ')'
            return f'detail: "ctx=" + (attemptContext ?? "") + ", route=" + String({var_name})'
        elif 'route=' in detail_string and 'reason=' in detail_string:
            # Pattern: "ctx=(attemptContext ?? ""), route=\{(routePeerId) reason=\{(sendError ?? "unknown")}"
            route_var = detail_string.split('route=')[1].split(')')[0] + ')'
            reason_expr = detail_string.split('reason=')[1].split(')')[0] + ')'
            return f'detail: "ctx=" + (attemptContext ?? "") + ", route=" + String({route_var}) + " reason=" + String({reason_expr})'
        elif 'target=' in detail_string and 'reason=' in detail_string and 'role=' not in detail_string:
            # Pattern: "ctx=(attemptContext ?? ""), target=\{(multipeerAddr) reason=\{(error.localizedDescription)}"
            target_var = detail_string.split('target=')[1].split(')')[0] + ')'
            reason_var = detail_string.split('reason=')[1].split(')')[0] + ')'
            return f'detail: "ctx=" + (attemptContext ?? "") + ", target=" + String({target_var}) + " reason=" + String({reason_var})'
        elif 'route_fallback=' in detail_string and 'reason=' not in detail_string:
            # Pattern: "ctx=(attemptContext ?? ""), route_fallback=\{(routePeerFallback)}"
            var_name = detail_string.split('route_fallback=')[1].split(')')[0] + ')'
            return f'detail: "ctx=" + (attemptContext ?? "") + ", route_fallback=" + String({var_name})'
        elif 'route_fallback=' in detail_string and 'reason=' in detail_string:
            # Pattern: "ctx=(attemptContext ?? ""), reason=swarm_bridge_unavailable route_fallback=\{(routePeerFallback)}"
            var_name = detail_string.split('route_fallback=')[1].split(')')[0] + ')'
            return f'detail: "ctx=" + (attemptContext ?? "") + ", reason=swarm_bridge_unavailable route_fallback=" + String({var_name})'
        elif 'reason=no_route_candidates' in detail_string:
            var_name = detail_string.split('route_fallback=')[1].split(')')[0] + ')'
            return f'detail: "ctx=" + (attemptContext ?? "") + ", reason=no_route_candidates route_fallback=" + String({var_name})'
        
        # Default fallback - just wrap the whole thing in String()
        return f'detail: String({detail_string})'
    
    # Replace all problematic detail parameters
    fixed_content = re.sub(pattern, fix_detail_match, content, flags=re.DOTALL)
    
    with open(file_path, 'w') as f:
        f.write(fixed_content)
    
    print(f"Fixed string interpolation issues in {file_path}")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python fix_swift_strings.py <file_path>")
        sys.exit(1)
    
    fix_swift_file(sys.argv[1])
