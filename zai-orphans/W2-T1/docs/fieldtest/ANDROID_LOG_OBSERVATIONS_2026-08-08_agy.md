# Observations -- Android log audit 2026-08-08

## 1. Errors and warnings observed

**Framework level:**
- `13:41:24.004  1437  3705 W ActivityManager: pid 15783 com.scmessenger.android sent binder code 13 with flags 2 and got error -32`
- `13:42:30.519  1437  2467 W ActivityManager: pid 15783 com.scmessenger.android sent binder code 33 with flags 2 and got error -32`
- Continuous `W PackageConfigPersister: App-specific configuration not found for packageName: com.scmessenger.android and userId: 0` lines throughout the 13:40 - 13:43 period.
- `13:40:38.951 15783 15783 I auditd  : type=1400 audit(0.0:131234): avc:  denied  { search } for  comm="DefaultDispatch" name="/" dev="cgroup2" ino=1 scontext=u:r:untrusted_app:s0:c119,c258,c512,c768 tcontext=u:object_r:cgroup_v2:s0 tclass=dir permissive=0 app=com.scmessenger.android` (and multiple similar SELinux denials for cgroup/cgroup2).
- `13:35:14.493  1437  1700 I AppsFilter: interaction: PackageSetting{95b4a07 com.scmessenger.android/10631} -> PackageSetting{896a927 com.google.android.apps.magazines/10222} BLOCKED` (and blocked interactions for `com.spotify.music`).

**App level (after 14:01:32):**
- `14:01:35.410 16775 18204 W BleGattServer: GATT server already running`
- `14:01:35.842 16775 18182 W MeshRepository: sendHistorySyncIfNeeded called for 12D3KooWJUJ1koSWwSEAX32z6SGaepikyqpJawpojoy6gvQ8k688`
- `14:01:35.843 16775 18182 W MeshRepository: sendHistorySyncIfNeeded shouldSend=true for 12D3KooWJUJ1koSWwSEAX32z6SGaepikyqpJawpojoy6gvQ8k688 (age=248709ms)`
- `14:01:42.835 16775 18185 W MeshRepository: Message ID tracking recreated for cdf35737-704b-4813-a223-9a76dc3014c7 (was missing)`
- `14:01:42.841 16775 17065 W MeshRepository: Bootstrap all-failed (consecutive=21), next attempt in 60000ms`

## 2. Delivery / transport events observed

App logs covering delivery are only present after 14:01:32:
- `14:01:36.162 16775 18182 I MeshRepository$attemptDirectSwarmDelivery$smartResult: [OK] Delivery via BLE client (target=4B:C0:2F:A8:76:AF)`
- `14:01:36.164 16775 18182 I MeshRepository: delivery_attempt msg=unassigned_1786233695917_send medium=ble phase=smart_router outcome=accepted detail=ctx=send role=central requested_target=4B:C0:2F:A8:76:AF target=4B:C0:2F:A8:76:AF`
- `14:01:36.377 16775 16786 D BleGattServer$gattServerCallback$1$onCharacteristicWriteRequest: Reassembled complete message (97 bytes) from 61:C8:0D:BB:39:CD`
- `14:01:37.320 16775 16787 D BleGattServer$gattServerCallback$1$onCharacteristicWriteRequest: Reassembled complete message (4724 bytes) from 61:C8:0D:BB:39:CD`
- `14:01:37.325 16775 18700 I MeshRepository: delivery_attempt msg=2add6ad7-98e4-4cee-8a52-9c82bfb40209 medium=core phase=rx outcome=received detail=sender=8094de3c9dda917c7413e4f14ac6f79e28aed2a76a208c2e690498787942d699`
- `14:01:37.436 16775 18700 I MeshRepository: delivery_attempt msg=2add6ad7-98e4-4cee-8a52-9c82bfb40209 medium=receipt phase=encode outcome=success detail=encoded_bytes=97 attempt=1`
- `14:01:42.675 16775 18185 I MeshRepository: delivery_attempt msg=cdf35737-704b-4813-a223-9a76dc3014c7 medium=ble phase=local_fallback outcome=target_fallback detail=ctx=initial_send target=4B:C0:2F:A8:76:AF reason=ble_peer_missing_connected_device_available`
- `14:01:42.839 16775 18185 I MeshRepository$sendMessage: Message sent (encrypted) to 8094de3c9dda917c7413e4f14ac6f79e28aed2a76a208c2e690498787942d699 (id=cdf35737-704b-4813-a223-9a76dc3014c7)`

## 3. Connectivity and peer-discovery events observed

App logs for connectivity are only present after 14:01:32:
- `14:01:35.413 16775 18204 I MeshRepository: BLE GATT identity beacon updated: c0a682ef... (431 bytes, listeners=0, external=0) p2p_id=12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG`
- `14:01:35.628 16775 18182 I MeshRepository: Dialed /ip4/192.168.0.142/tcp/443 via SwarmBridge`
- `14:01:35.630 16775 18182 I MeshRepository$initializeAndStartSwarm: [OK] Internet transport (Swarm) started and bridge wired`
- `14:01:35.738 16775 18750 D MeshRepository$startMeshService: Core notified identified: 12D3KooWJUJ1koSWwSEAX32z6SGaepikyqpJawpojoy6gvQ8k688 (agent: scmessenger/0.4.0/full/relay/12D3KooWJUJ1koSWwSEAX32z6SGaepikyqpJawpojoy6gvQ8k688) with 111 addresses`
- `14:01:35.739 16775 18182 I MeshRepository$startMeshService$2$onPeerIdentified: TCP/mDNS: LAN peer detected 12D3KooWJUJ1koSWwSEAX32z6SGaepikyqpJawpojoy6gvQ8k688 with 13 local addresses`
- `14:01:35.802 16775 18182 D MeshEventBus: PeerEvent emitted: Connected(peerId=12D3KooWJUJ1koSWwSEAX32z6SGaepikyqpJawpojoy6gvQ8k688, transport=INTERNET)`
- `14:01:42.839 16775 17065 I MeshRepository: Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]`

## 4. The 13:43:28 process death -- what the log does and does not show

The logs do **not** show any FATAL crash, ANR, tombstone, or lowmemorykiller intervention at or immediately preceding the process death.

The logs **do** show that the process was killed intentionally by the system because the user swiped the app away from the recent tasks list (evidenced by `remove task`):
- `13:43:27.262  1437  1761 I wm_destroy_activity: [0,198880577,685,com.scmessenger.android/.ui.MainActivity,finish-imm:remove-by-pid#8962]`
- `13:43:27.696  1437  1642 I ActivityManager: Killing 15783:com.scmessenger.android/u0a631 (adj 905): remove task`
- `13:43:27.696  1437  1642 I am_kill : [0,15783,com.scmessenger.android,905,remove task,577444]`
- `13:43:28.566  1437  3701 I am_proc_died: [0,15783,com.scmessenger.android,905,19]`

This resolves competing observations: it confirms the user manual close ("closed and reopened the app") rather than an unexpected application crash.

## 5. Coverage gaps (what is missing from this capture)

- **Complete app log eviction during test window:** The `MeshRepository`, `BleGattServer`, and all other app-owned log lines are entirely missing for the main test window (13:35 - 13:55). The ring buffer evidently evicted all of pid 15783's and pid 16775's lines from before 14:01:32 due to high-volume system logging.
- **No explicit Bluetooth toggle state-change logs:** There are no `BluetoothManagerService` lines capturing the operator turning Bluetooth off during the test window. There is only a background bluetooth service death (`13:41:18.342 Process com.google.android.bluetooth (pid 9857) has died: cch CEM`) which is due to cached process memory reclaiming, not a toggle.

## 6. Additional logs / telemetry needed, with exact commands or tag names

To accurately resolve what happened during the test window gaps:
1. Increase the log buffer size before the test via `adb logcat -G 16M` (or 64M) to avoid app logs being evicted by system noise.
2. Capture Bluetooth service toggles explicitly: `adb logcat -s BluetoothManagerService AdapterState`
3. Retrieve SCMessenger transport events with `adb logcat -s MeshRepository BleGattServer BleGattClient SwarmBridge`
4. Confirm presence of macOS/Windows CLIs on the same LAN using `tcpdump` or Wireshark to monitor mDNS packets on port 5353, or review their local daemon logs.
