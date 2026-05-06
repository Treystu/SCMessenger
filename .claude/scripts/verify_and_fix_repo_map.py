#!/usr/bin/env python3
"""
REPO_MAP Verification and Fix Script

This script:
1. Verifies the integrity of the REPO_MAP index
2. Identifies and fixes missing file_modified_at timestamps
3. Detects stale entries (files modified after indexing)
4. Provides detailed reporting and automated fixes
"""

import os
import json
import sys
from pathlib import Path
from datetime import datetime, timezone
from typing import Dict, List, Tuple

class RepoMapVerifier:
    def __init__(self, repo_root: Path):
        self.repo_root = repo_root
        self.handoff_audit = repo_root / "HANDOFF_AUDIT"
        self.index_path = self.handoff_audit / "repo_map_index.json"
        self.done_dir = self.handoff_audit / "done"
        self.output_dir = self.handoff_audit / "output"
        
        self.issues = {
            "missing_timestamps": [],
            "stale_files": [],
            "missing_files": [],
            "invalid_paths": [],
            "orphaned_entries": []
        }
        
    def load_index(self) -> Dict:
        """Load the repo_map_index.json file"""
        if not self.index_path.exists():
            print(f"[ERROR] Index file not found: {self.index_path}")
            sys.exit(1)
            
        with open(self.index_path, 'r', encoding='utf-8') as f:
            return json.load(f)
    
    def save_index(self, index: Dict):
        """Save the updated index"""
        with open(self.index_path, 'w', encoding='utf-8') as f:
            json.dump(index, f, indent=2)
        print(f"[SUCCESS] Index saved to {self.index_path}")
    
    def verify_file_entry(self, rel_path: str, metadata: Dict) -> List[str]:
        """Verify a single file entry and return list of issues"""
        issues = []
        
        # Check if file_modified_at is missing or empty
        file_mod_at = metadata.get('file_modified_at', '')
        if not file_mod_at or file_mod_at == '':
            issues.append(f"missing_timestamp:{rel_path}")
            self.issues["missing_timestamps"].append(rel_path)
            return issues
        
        # Check if the actual file exists
        file_path = self.repo_root / rel_path
        if not file_path.exists():
            issues.append(f"missing_file:{rel_path}")
            self.issues["missing_files"].append(rel_path)
            return issues
        
        # Check if file is stale (modified after indexing)
        try:
            indexed_dt = datetime.fromisoformat(metadata['indexed_at'].replace('Z', '+00:00'))
            file_mod_dt = datetime.fromisoformat(file_mod_at.replace('Z', '+00:00'))
            actual_mod_dt = datetime.fromtimestamp(file_path.stat().st_mtime, tz=timezone.utc)
            
            # If actual modification time is newer than indexed time, it's stale
            if actual_mod_dt > indexed_dt:
                hours_stale = (datetime.now(timezone.utc) - actual_mod_dt).total_seconds() / 3600
                if hours_stale > 24:
                    issues.append(f"stale:{rel_path}:{hours_stale:.1f}h")
                    self.issues["stale_files"].append((rel_path, hours_stale))
                    
        except (ValueError, KeyError) as e:
            issues.append(f"invalid_timestamp:{rel_path}:{str(e)}")
            self.issues["invalid_paths"].append((rel_path, str(e)))
        
        return issues
    
    def fix_missing_timestamps(self, index: Dict) -> int:
        """Fix entries with missing file_modified_at timestamps"""
        fixed_count = 0
        
        for rel_path, metadata in index["files"].items():
            file_mod_at = metadata.get('file_modified_at', '')
            
            if not file_mod_at or file_mod_at == '':
                # Try to get the actual file modification time
                file_path = self.repo_root / rel_path
                
                if file_path.exists():
                    try:
                        actual_mod_time = datetime.fromtimestamp(
                            file_path.stat().st_mtime, 
                            tz=timezone.utc
                        )
                        metadata['file_modified_at'] = actual_mod_time.isoformat().replace('+00:00', 'Z')
                        fixed_count += 1
                        print(f"  [OK] Fixed timestamp for: {rel_path}")
                    except Exception as e:
                        print(f"  [FAIL] Could not fix {rel_path}: {e}")
                else:
                    # File doesn't exist, use indexed_at as fallback
                    metadata['file_modified_at'] = metadata.get('indexed_at', '')
                    fixed_count += 1
                    print(f"  [WARN] File missing, used indexed_at for: {rel_path}")
        
        return fixed_count
    
    def update_stale_entries(self, index: Dict) -> int:
        """Update file_modified_at for stale entries"""
        updated_count = 0
        
        for rel_path, hours_stale in self.issues["stale_files"]:
            file_path = self.repo_root / rel_path
            
            if file_path.exists():
                try:
                    actual_mod_time = datetime.fromtimestamp(
                        file_path.stat().st_mtime,
                        tz=timezone.utc
                    )
                    index["files"][rel_path]['file_modified_at'] = actual_mod_time.isoformat().replace('+00:00', 'Z')
                    updated_count += 1
                    print(f"  [OK] Updated stale entry: {rel_path} ({hours_stale:.1f}h old)")
                except Exception as e:
                    print(f"  [FAIL] Could not update {rel_path}: {e}")
        
        return updated_count
    
    def generate_report(self, index: Dict) -> str:
        """Generate a detailed verification report"""
        total_files = len(index["files"])
        generated_at = index.get("generated_at", "Unknown")
        
        report = []
        report.append("=" * 80)
        report.append("REPO_MAP VERIFICATION REPORT")
        report.append("=" * 80)
        report.append(f"Generated: {datetime.now(timezone.utc).isoformat()}")
        report.append(f"Index Generated At: {generated_at}")
        report.append(f"Total Indexed Files: {total_files}")
        report.append("")
        
        # Missing timestamps
        if self.issues["missing_timestamps"]:
            report.append(f"[ERROR] MISSING TIMESTAMPS: {len(self.issues['missing_timestamps'])}")
            for path in self.issues["missing_timestamps"][:10]:
                report.append(f"   - {path}")
            if len(self.issues["missing_timestamps"]) > 10:
                report.append(f"   ... and {len(self.issues['missing_timestamps']) - 10} more")
            report.append("")
        
        # Stale files
        if self.issues["stale_files"]:
            report.append(f"[WARNING] STALE FILES (>24h): {len(self.issues['stale_files'])}")
            for path, hours in sorted(self.issues["stale_files"], key=lambda x: x[1], reverse=True)[:10]:
                report.append(f"   - {path} ({hours:.1f}h old)")
            if len(self.issues["stale_files"]) > 10:
                report.append(f"   ... and {len(self.issues['stale_files']) - 10} more")
            report.append("")
        
        # Missing files
        if self.issues["missing_files"]:
            report.append(f"[ERROR] MISSING FILES: {len(self.issues['missing_files'])}")
            for path in self.issues["missing_files"][:10]:
                report.append(f"   - {path}")
            if len(self.issues["missing_files"]) > 10:
                report.append(f"   ... and {len(self.issues['missing_files']) - 10} more")
            report.append("")
        
        # Invalid paths
        if self.issues["invalid_paths"]:
            report.append(f"[ERROR] INVALID PATHS: {len(self.issues['invalid_paths'])}")
            for path, error in self.issues["invalid_paths"][:5]:
                report.append(f"   - {path}: {error}")
            if len(self.issues["invalid_paths"]) > 5:
                report.append(f"   ... and {len(self.issues['invalid_paths']) - 5} more")
            report.append("")
        
        # Summary
        total_issues = (
            len(self.issues["missing_timestamps"]) +
            len(self.issues["stale_files"]) +
            len(self.issues["missing_files"]) +
            len(self.issues["invalid_paths"])
        )
        
        if total_issues == 0:
            report.append("[SUCCESS] NO ISSUES FOUND - REPO_MAP IS HEALTHY")
        else:
            report.append(f"[SUMMARY] TOTAL ISSUES: {total_issues}")
        
        report.append("=" * 80)
        
        return "\n".join(report)
    
    def run_verification(self, fix: bool = False) -> bool:
        """Run full verification and optionally fix issues"""
        print("[INFO] Loading REPO_MAP index...")
        index = self.load_index()
        
        print(f"[INFO] Verifying {len(index['files'])} indexed files...")
        
        # Verify each file entry
        for rel_path, metadata in index["files"].items():
            self.verify_file_entry(rel_path, metadata)
        
        # Generate and print report
        report = self.generate_report(index)
        print("\n" + report)
        
        # Save report to file
        report_path = self.handoff_audit / "repo_map_verification_report.txt"
        with open(report_path, 'w', encoding='utf-8') as f:
            f.write(report)
        print(f"\n[INFO] Report saved to: {report_path}")
        
        # Fix issues if requested
        if fix:
            print("\n[INFO] FIXING ISSUES...")
            
            if self.issues["missing_timestamps"]:
                print(f"\n[INFO] Fixing {len(self.issues['missing_timestamps'])} missing timestamps...")
                fixed = self.fix_missing_timestamps(index)
                print(f"   [SUCCESS] Fixed {fixed} entries")
            
            if self.issues["stale_files"]:
                print(f"\n[INFO] Updating {len(self.issues['stale_files'])} stale entries...")
                updated = self.update_stale_entries(index)
                print(f"   [SUCCESS] Updated {updated} entries")
            
            # Update generated_at timestamp
            index["generated_at"] = datetime.now(timezone.utc).isoformat().replace('+00:00', 'Z')
            
            # Save the fixed index
            self.save_index(index)
            
            print("\n[SUCCESS] ALL FIXES APPLIED")
            return True
        
        return len(self.issues["missing_timestamps"]) == 0 and len(self.issues["stale_files"]) == 0


def main():
    import argparse
    
    parser = argparse.ArgumentParser(description="Verify and fix REPO_MAP index")
    parser.add_argument("--fix", action="store_true", help="Automatically fix issues")
    parser.add_argument("--repo-root", type=str, help="Repository root path (default: auto-detect)")
    
    args = parser.parse_args()
    
    # Determine repo root
    if args.repo_root:
        repo_root = Path(args.repo_root).resolve()
    else:
        # Auto-detect: script is in .claude/scripts/
        repo_root = Path(__file__).resolve().parent.parent.parent
    
    print(f"[INFO] Repository root: {repo_root}")
    
    verifier = RepoMapVerifier(repo_root)
    success = verifier.run_verification(fix=args.fix)
    
    if not success and not args.fix:
        print("\n[INFO] Run with --fix to automatically fix issues")
        sys.exit(1)
    
    sys.exit(0)


if __name__ == "__main__":
    main()
