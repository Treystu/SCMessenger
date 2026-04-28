#!/usr/bin/env python3

import re
import sys

def fix_swift_file(file_path):
    """Fix Swift string interpolation issues in MeshRepository.swift"""
    
    with open(file_path, 'r') as f:
        lines = f.readlines()
    
    # Lines to fix (line_number: [variable_names])
    fixes = {
        3786: ['attemptContext', 'routePeerCandidates.count'],
        3794: ['attemptContext'],
        3803: ['attemptContext'],
        3827: ['attemptContext', 'fallbackPeer'],
        3835: ['attemptContext', 'effectiveBlePeerId', 'requestedBlePeerId'],
        3864: ['attemptContext', 'multipeerAddr'],
        3874: ['attemptContext', 'multipeerAddr', 'error.localizedDescription'],
        3905: ['attemptContext', 'bleAddr', 'target'],
        3922: ['attemptContext', 'bleAddr', 'target'],
        3938: ['attemptContext', 'bleAddr', 'lastFailureReason', 'connectedBlePeerIds.count'],
    }
    
    # Additional fixes for other patterns
    additional_patterns = [
        (r'detail:\s*"ctx=\(attemptContext\s*\?\?\s*""\).*?reason=swarm_bridge_unavailable.*?"',
         'detail: "ctx=" + (attemptContext ?? "") + ", reason=swarm_bridge_unavailable"'),
        (r'detail:\s*"ctx=\(attemptContext\s*\?\?\s*""\).*?route=\$\{(.*?)\}',
         'detail: "ctx=" + (attemptContext ?? "") + ", route=" + String(\1)'),
        (r'detail:\s*"ctx=\(attemptContext\s*\?\?\s*""\).*?route_fallback=\$\{(.*?)\}',
         'detail: "ctx=" + (attemptContext ?? "") + ", route_fallback=" + String(\1)'),
    ]
    
    modified_lines = []
    for i, line in enumerate(lines):
        line_num = i + 1  # Convert to 1-based indexing
        
        if line_num in fixes:
            # This is one of our targeted lines
            vars = fixes[line_num]
            if len(vars) == 1:
                # Simple case: just attemptContext
                new_line = line.replace('detail: "ctx=\(attemptContext) reason=strict_ble_only_mode"',
                                      'detail: "ctx=" + (attemptContext ?? "") + ", reason=strict_ble_only_mode"')
            elif len(vars) == 2:
                if 'route_candidates=' in line:
                    var1, var2 = vars
                    new_line = line.replace('detail: "ctx=\(attemptContext) route_candidates=\$\{' + var2 + '}\$\}',
                                          'detail: "ctx=" + (attemptContext ?? "") + ", route_candidates=" + String(' + var2 + ')')
                elif 'target=' in line and 'reason=ble_peer_missing_connected_device_available' in line:
                    var1, var2 = vars
                    new_line = line.replace('detail: "ctx=\(attemptContext) target=\$\{' + var2 + '}\$ reason=ble_peer_missing_connected_device_available"',
                                          'detail: "ctx=" + (attemptContext ?? "") + ", target=" + String(' + var2 + ') + " reason=ble_peer_missing_connected_device_available"')
                elif 'target=' in line and 'multipeerAddr' in line and 'reason=' not in line:
                    var1, var2 = vars
                    new_line = line.replace('detail: "ctx=\(attemptContext) target=\$\{' + var2 + '}\$\}',
                                          'detail: "ctx=" + (attemptContext ?? "") + ", target=" + String(' + var2 + ')')
                else:
                    new_line = line  # Keep original if pattern doesn't match
            elif len(vars) == 3:
                if 'target=' in line and 'requested_target=' in line:
                    var1, var2, var3 = vars
                    new_line = line.replace('detail: "ctx=\(attemptContext) target=\$\{' + var2 + '}\$ requested_target=\$\{' + var3 + '}\$ reason=prefer_connected_device"',
                                          'detail: "ctx=" + (attemptContext ?? "") + ", target=" + String(' + var2 + ') + " requested_target=" + String(' + var3 + ') + " reason=prefer_connected_device"')
                elif 'role=peripheral' in line:
                    var1, var2, var3 = vars
                    new_line = line.replace('detail: "ctx=\(attemptContext) role=peripheral requested_target=\$\{' + var2 + '}\$ target=\$\{' + var3 + '}\$\}',
                                          'detail: "ctx=" + (attemptContext ?? "") + ", role=peripheral requested_target=" + String(' + var2 + ') + " target=" + String(' + var3 + ')')
                elif 'role=central' in line:
                    var1, var2, var3 = vars
                    new_line = line.replace('detail: "ctx=\(attemptContext) role=central requested_target=\$\{' + var2 + '}\$ target=\$\{' + var3 + '}\$\}',
                                          'detail: "ctx=" + (attemptContext ?? "") + ", role=central requested_target=" + String(' + var2 + ') + " target=" + String(' + var3 + ')')
                elif 'target=' in line and 'reason=' in line:
                    var1, var2, var3 = vars
                    new_line = line.replace('detail: "ctx=\(attemptContext) target=\$\{' + var2 + '}\$ reason=\$\{' + var3 + '}\$\}',
                                          'detail: "ctx=" + (attemptContext ?? "") + ", target=" + String(' + var2 + ') + " reason=" + String(' + var3 + ')')
                else:
                    new_line = line  # Keep original if pattern doesn't match
            elif len(vars) == 4:
                var1, var2, var3, var4 = vars
                new_line = line.replace('detail: "ctx=\(attemptContext) requested_target=\$\{' + var2 + '}\$ reason=\$\{' + var3 + '}\$ connected=\$\{' + var4 + '}\$\}',
                                      'detail: "ctx=" + (attemptContext ?? "") + ", requested_target=" + String(' + var2 + ') + " reason=" + String(' + var3 + ') + " connected=" + String(' + var4 + ')')
            else:
                new_line = line  # Keep original if we don't know how to fix it
            
            modified_lines.append(new_line)
        else:
            # Check if this line matches any of our additional patterns
            modified = False
            for pattern, replacement in additional_patterns:
                if re.search(pattern, line):
                    new_line = re.sub(pattern, replacement, line)
                    modified_lines.append(new_line)
                    modified = True
                    break
            
            if not modified:
                modified_lines.append(line)
    
    with open(file_path, 'w') as f:
        f.writelines(modified_lines)
    
    print(f"Fixed string interpolation issues in {file_path}")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python fix_swift_strings_targeted.py <file_path>")
        sys.exit(1)
    
    fix_swift_file(sys.argv[1])
