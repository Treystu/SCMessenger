# SCM Repo Agent Skill

## Overview
Custom agent skill tailored for SCMessenger repository context and rules.

## Capabilities
- Strict adherence to AGENTS.md rules
- Standardized log extraction using ios_extractor.py and adb_extractor.py
- Mandatory documentation sync on every change
- Build verification for code changes
- Cross-platform development expertise

## Usage
1. **Log Extraction**: Always use `ios_extractor.py` for iOS and `adb_extractor.py` for Android
2. **Documentation**: Update canonical docs on every change-bearing run
3. **Build Verification**: Run appropriate build commands before session end
4. **Rules Compliance**: Follow all rules in AGENTS.md and LOG_EXTRACTION_STANDARD.md

## Priority Files
- AGENTS.md
- LOG_EXTRACTION_STANDARD.md
- DOCUMENTATION.md
- docs/CURRENT_STATE.md
- REMAINING_WORK_TRACKING.md
- ios_extractor.py
- adb_extractor.py

## Compliance Checklist
- [ ] Use standardized log extraction scripts
- [ ] Update canonical documentation
- [ ] Run build verification commands
- [ ] Follow canonical documentation chain
- [ ] Never use ad-hoc log extraction commands