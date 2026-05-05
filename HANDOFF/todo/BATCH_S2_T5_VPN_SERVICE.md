# S2-T5: MeshVpnService Wiring

## Status
- [ ] TODO

## Task ID
`S2-T5`

## Sprint
Sprint 2: Core Wiring

## LoC Estimate
~100

## Depends
S2-T4 (Relay Bootstrap Infrastructure)

## Files
- `android/app/src/main/java/com/scmessenger/android/service/MeshVpnService.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/settings/PowerSettingsScreen.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt`

## Actions
1. Wire VPN toggle in `PowerSettingsScreen`: switch → save preference → request enable
2. Handle VPN permission request (`VpnService.prepare()` → intent)
3. Implement VPN lifecycle:
   - `MeshVpnService.onCreate()` → configure tunnel
   - `MeshVpnService.onStartCommand()` → connect
   - `MeshVpnService.onDestroy()` → disconnect
4. Route mesh traffic through VPN interface
5. Test: enable VPN → network switch (WiFi↔Cellular) → connection maintained
6. Handle VPN revocation gracefully

## Verification
- VPN mode toggle works and persists
- Connection survives network switch (WiFi↔Cellular)
- VPN notification shows mesh status
- Graceful degradation if VPN revoked

## Notes
- VPN is optional - mesh works without it
- Useful for persistent connection in background
- Must handle VPN permission denial gracefully