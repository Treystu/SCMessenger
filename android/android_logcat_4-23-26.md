2026-04-23 08:55:37.657  7368-7386  DisplayManager          com.scmessenger.android              I  Choreographer implicitly registered for the refresh rate.
2026-04-23 08:55:38.406  7368-7368  InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 08:55:54.686  1619-7481  ActivityManager         system_server                        E  ANR in com.scmessenger.android (com.scmessenger.android/.ui.MainActivity)
                                                                                                    PID: 7368
                                                                                                    Reason: Input dispatching timed out (3d3030e com.scmessenger.android/com.scmessenger.android.ui.MainActivity is not responding. Waited 5001ms for MotionEvent).
                                                                                                    Parent: com.scmessenger.android/.ui.MainActivity
                                                                                                    ErrorId: b04e7ee5-5ef2-4011-affe-6deed6bd1ea2
                                                                                                    Frozen: false
                                                                                                    Load: 2.59 / 2.56 / 2.74
                                                                                                    ----- Output from /proc/pressure/memory -----
                                                                                                    some avg10=0.00 avg60=0.06 avg300=0.23 total=1514624900
                                                                                                    full avg10=0.00 avg60=0.00 avg300=0.12 total=875698344
                                                                                                    ----- End output from /proc/pressure/memory -----
                                                                                                    ----- Output from /proc/pressure/cpu -----
                                                                                                    some avg10=5.84 avg60=7.69 avg300=7.27 total=26832770308
                                                                                                    full avg10=0.00 avg60=0.00 avg300=0.00 total=0
                                                                                                    ----- End output from /proc/pressure/cpu -----
                                                                                                    ----- Output from /proc/pressure/io -----
                                                                                                    some avg10=0.20 avg60=1.00 avg300=1.26 total=4751174535
                                                                                                    full avg10=0.09 avg60=0.51 avg300=0.63 total=2471301110
                                                                                                    ----- End output from /proc/pressure/io -----
                                                                                                    
                                                                                                    CPU usage from 1ms to 6551ms later (2026-04-23 08:55:48.078 to 2026-04-23 08:55:54.628) with 99% awake:
                                                                                                      500% 7368/com.scmessenger.android: 497% user + 2.5% kernel / faults: 5987 minor 147 major
                                                                                                      31% 11332/com.lemon.lvoverseas: 15% user + 15% kernel / faults: 8328 minor 5600 major
                                                                                                      28% 1619/system_server: 14% user + 14% kernel / faults: 6509 minor 1088 major
                                                                                                      10% 2261/com.android.systemui: 6.7% user + 3.6% kernel / faults: 3096 minor 403 major
                                                                                                      6.2% 2476/com.android.hbmsvmanager: 4.5% user + 1.6% kernel / faults: 2353 minor 848 major
                                                                                                      5.7% 92/kswapd0: 0% user + 5.7% kernel
                                                                                                      3.5% 21975/com.google.android.inputmethod.latin: 2.4% user + 1% kernel / faults: 3068 minor 1518 major
                                                                                                      2.4% 546/surfaceflinger: 1.5% user + 0.9% kernel / faults: 14 minor 1 major
                                                                                                      1.8% 2578/com.google.android.nfc: 1% user + 0.7% kernel / faults: 1134 minor 740 major
                                                                                                      1.6% 2451/com.shannon.qualifiednetworksservice: 1% user + 0.6% kernel / faults: 1294 minor 674 major
                                                                                                    79% TOTAL: 69% user + 7.6% kernel + 0.4% iowait + 0.6% irq + 0.3% softirq
                                                                                                    CPU usage from 36ms to 587ms later (2026-04-23 08:55:48.113 to 2026-04-23 08:55:48.664):
                                                                                                      405% 7368/com.scmessenger.android: 402% user + 2.7% kernel / faults: 171 minor 10 major
                                                                                                        98% 7368/ssenger.android: 98% user + 0% kernel
                                                                                                        98% 7429/DefaultDispatch: 98% user + 0% kernel
                                                                                                        95% 7383/DefaultDispatch: 95% user + 0% kernel
                                                                                                        95% 7385/DefaultDispatch: 95% user + 0% kernel
                                                                                                        8.2% 7386/RenderThread: 8.2% user + 0% kernel
                                                                                                        5.4% 7369/Signal Catcher: 5.4% user + 0% kernel
                                                                                                      118% 1619/system_server: 55% user + 62% kernel / faults: 3282 minor 823 major
                                                                                                        51% 7482/AnrAuxiliaryTas: 15% user + 35% kernel
                                                                                                        48% 1630/Signal Catcher: 33% user + 15% kernel
                                                                                                        4.4% 7481/AnrConsumer: 0% user + 4.4% kernel
                                                                                                        2.2% 1734/PackageManagerB: 0% user + 2.2% kernel
                                                                                                        2.2% 2162/tworkPolicy.uid: 2.2% user + 0% kernel
                                                                                                        2.2% 2695/com.android.ser: 0% user + 2.2% kernel
                                                                                                        2.2% 7480/AnrMainProcessD: 0% user + 2.2% kernel
                                                                                                        2.2% 9496/binder:1619_1C: 2.2% user + 0% kernel
                                                                                                      9.2% 92/kswapd0: 0% user + 9.2% kernel
                                                                                                      10% 546/surfaceflinger: 8% user + 2% kernel
                                                                                                        6% 546/surfaceflinger: 4% user + 2% kernel
                                                                                                        4% 647/TimerDispatch: 2% user + 2% kernel
                                                                                                      1.8% 13/ksoftirqd/0: 0% user + 1.8% kernel
                                                                                                      1.8% 73/rcuop/7: 0% user + 1.8% kernel
                                                                                                      1.8% 157/eh_comp_thread: 0% user + 1.8% kernel
                                                                                                      1.9% 275/decon0_kthread: 0% user + 1.9% kernel
                                                                                                      2% 548/android.hardware.graphics.composer@2.4-service: 0% user + 2% kernel / faults: 2 major
                                                                                                        2% 548/composer@2.4-se: 0% user + 2% kernel
                                                                                                      2% 687/tombstoned: 0% user + 2% kernel / faults: 1 minor 2 major
                                                                                                      2.1% 1253/vendor.google.wifi_ext-service-vendor: 0% user + 2.1% kernel / faults: 54 minor 70 major
                                                                                                      2.1% 1396/dhd_rpm_state_thread: 0% user + 2.1% kernel
                                                                                                      2.6% 5624/kworker/1:3H-kverityd: 0% user + 2.6% kernel
                                                                                                      2.6% 6956/kworker/2:0-events: 0% user + 2.6% kernel
                                                                                                      2.9% 11332/com.lemon.lvoverseas: 2.9% user + 0% kernel / faults: 1 minor
                                                                                                        2.9% 11424/Thread-37: 0% user + 2.9% kernel
2026-04-23 08:55:55.424  7368-7368  InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 08:56:07.701  7368-7368  InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 08:56:32.041  7368-7420  NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 08:56:36.427  7368-7383  MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 08:56:36.436  7368-7383  MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=45))
2026-04-23 08:56:37.693  7368-7368  MainActivity            com.scmessenger.android              D  MainActivity paused
2026-04-23 08:56:37.698  7368-7368  VRI[MainActivity]       com.scmessenger.android              D  visibilityChanged oldVisibility=true newVisibility=false
2026-04-23 08:56:37.797  7368-7373  ssenger.android         com.scmessenger.android              W  Cleared Reference was only reachable from finalizer (only reported once)
2026-04-23 08:56:41.357  7368-7420  NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 08:56:41.439  7368-7429  MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 08:56:41.445  7368-7383  MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=50))
2026-04-23 08:56:44.415  7368-7420  NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 08:56:46.449  7368-7383  MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 08:56:46.626  7368-7409  MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=55))
2026-04-23 08:59:49.049  7368-7368  BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 08:59:49.049  7368-7368  BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 08:59:49.050  7368-7368  BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 08:59:49.050  7368-7420  NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 08:59:49.059  7368-7428  MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 08:59:49.069  7368-7429  MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 08:59:49.087  7368-7429  MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=238))
2026-04-23 08:59:49.088  7368-7428  NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 08:59:49.088  7368-7428  MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 08:59:49.089  7368-7428  CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 08:59:49.090  7368-7428  NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 08:59:49.092  7368-7428  NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 08:59:49.092  7368-7428  MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 08:59:49.093  7368-7428  CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 08:59:49.094  7368-7428  NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 08:59:49.097  7368-7428  NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 08:59:49.098  7368-7428  MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 08:59:49.099  7368-7428  CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 08:59:49.100  7368-7428  NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 08:59:49.102  7368-7428  NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 08:59:49.103  7368-7428  MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 08:59:49.103  7368-7428  CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 08:59:49.105  7368-7428  NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 08:59:49.107  7368-7428  NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 08:59:49.108  7368-7428  MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 08:59:49.108  7368-7428  CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 08:59:49.110  7368-7428  NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 08:59:49.111  7368-7428  NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 08:59:49.113  7368-7368  MainActivity            com.scmessenger.android              D  MainActivity resumed
2026-04-23 08:59:49.114  7368-7428  MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 08:59:49.115  7368-7428  CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 08:59:49.115  7368-7368  MainActivity            com.scmessenger.android              D  All permissions already granted
2026-04-23 08:59:49.116  7368-7428  NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 08:59:49.117  7368-7428  MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=3), next attempt in 30000ms
2026-04-23 08:59:49.119  7368-7368  WifiDirect...2pReceiver com.scmessenger.android              D  This device changed
2026-04-23 08:59:49.119  7368-7429  BleGattServer           com.scmessenger.android              W  GATT server already running
2026-04-23 08:59:49.119  7368-7427  BleScanner              com.scmessenger.android              D  BLE scan already in progress, reusing existing session
2026-04-23 08:59:49.121  7368-7425  WifiTransportManager    com.scmessenger.android              D  WifiTransportManager initialized
2026-04-23 08:59:49.121  7368-7429  BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 08:59:49.125  7368-7425  WifiTransportManager    com.scmessenger.android              D  WiFi P2P discovery already active; skipping duplicate start
2026-04-23 08:59:49.127  7368-7429  BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 08:59:49.128  7368-7429  MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 08:59:49.205  7368-7368  InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 08:59:49.206  7368-7368  ImeTracker              com.scmessenger.android              I  com.scmessenger.android:c1261083: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
---------------------------- PROCESS STARTED (10908) for package com.scmessenger.android ----------------------------
2026-04-23 09:00:02.210 10908-10908 ssenger.android         com.scmessenger.android              I  Using CollectorTypeCMC GC.
2026-04-23 09:00:02.216 10908-10908 nativeloader            com.scmessenger.android              D  Load libframework-connectivity-tiramisu-jni.so using APEX ns com_android_tethering for caller /apex/com.android.tethering/javalib/framework-connectivity-t.jar: ok
2026-04-23 09:00:02.578 10908-10908 nativeloader            com.scmessenger.android              D  Configuring clns-9 for other apk /data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk. target_sdk_version=35, uses_libraries=, library_path=/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/lib/arm64:/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk!/lib/arm64-v8a, permitted_path=/data:/mnt/expand:/data/user/0/com.scmessenger.android
2026-04-23 09:00:02.579 10908-10908 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 202956589; UID 10499; state: ENABLED
2026-04-23 09:00:02.592 10908-10908 GraphicsEnvironment     com.scmessenger.android              V  Currently set values for:
2026-04-23 09:00:02.592 10908-10908 GraphicsEnvironment     com.scmessenger.android              V    angle_gl_driver_selection_pkgs=[com.android.angle]
2026-04-23 09:00:02.592 10908-10908 GraphicsEnvironment     com.scmessenger.android              V    angle_gl_driver_selection_values=[angle]
2026-04-23 09:00:02.592 10908-10908 GraphicsEnvironment     com.scmessenger.android              V  com.scmessenger.android is not listed in per-application setting
2026-04-23 09:00:02.592 10908-10908 GraphicsEnvironment     com.scmessenger.android              V  No special selections for ANGLE, returning default driver choice
2026-04-23 09:00:02.592 10908-10908 GraphicsEnvironment     com.scmessenger.android              V  Neither updatable production driver nor prerelease driver is supported.
2026-04-23 09:00:02.617 10908-10908 WM-WrkMgrInitializer    com.scmessenger.android              D  Initializing WorkManager with default configuration.
2026-04-23 09:00:02.654 10908-10908 MeshApplication         com.scmessenger.android              I  SCMessenger application started
2026-04-23 09:00:02.656 10908-10977 StorageManager          com.scmessenger.android              D  Performing startup storage maintenance...
2026-04-23 09:00:02.664 10908-10977 StorageManager          com.scmessenger.android              D  Logs rotated successfully.
2026-04-23 09:00:02.667 10908-10977 StorageManager          com.scmessenger.android              D  Startup maintenance complete. Available storage: 17845 MB
2026-04-23 09:00:02.668 10908-10977 MeshApplic...n$onCreate com.scmessenger.android              I  Startup storage maintenance completed
2026-04-23 09:00:02.677 10908-10908 BootReceiver            com.scmessenger.android              I  Boot completed, checking auto-start preference
2026-04-23 09:00:02.682 10908-10980 DisplayManager          com.scmessenger.android              I  Choreographer implicitly registered for the refresh rate.
2026-04-23 09:00:02.685 10908-10980 vulkan                  com.scmessenger.android              D  searching for layers in '/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/lib/arm64'
2026-04-23 09:00:02.685 10908-10980 vulkan                  com.scmessenger.android              D  searching for layers in '/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk!/lib/arm64-v8a'
2026-04-23 09:00:02.691 10908-10908 DesktopExperienceFlags  com.scmessenger.android              D  Toggle override initialized to: false
2026-04-23 09:00:02.711 10908-10908 MeshRepository          com.scmessenger.android              D  MeshRepository initialized with storage: /data/user/0/com.scmessenger.android/files
2026-04-23 09:00:02.716 10908-10908 MeshRepository          com.scmessenger.android              D  AND-CONTACTS-WIPE-001: Reinstall check - hasIdentityBackup=true, contactsOnDisk=true, historyOnDisk=true
2026-04-23 09:00:02.717 10908-10908 MeshRepository          com.scmessenger.android              D  AND-CONTACTS-WIPE-001: Normal startup - all data present
2026-04-23 09:00:02.719 10908-10908 MeshRepository          com.scmessenger.android              D  Contacts migration already completed, skipping
2026-04-23 09:00:02.724 10908-10908 nativeloader            com.scmessenger.android              D  Load /data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/lib/arm64/libjnidispatch.so using class loader ns clns-9 (caller=/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk!classes17.dex): ok
2026-04-23 09:00:02.846 10908-10908 MeshRepository          com.scmessenger.android              I  all_managers_init_success
2026-04-23 09:00:02.846 10908-10908 MeshRepository          com.scmessenger.android              I  All managers initialized successfully
2026-04-23 09:00:02.848 10908-10908 MeshRepository          com.scmessenger.android              D  AND-CONTACTS-WIPE-001: Contact data verification - Found 0 contacts
2026-04-23 09:00:02.848 10908-10908 MeshRepository          com.scmessenger.android              W  AND-CONTACTS-WIPE-001: WARNING - No contacts found. If this is unexpected, data recovery may be needed.
2026-04-23 09:00:02.850 10908-10908 MainActivity            com.scmessenger.android              D  MainActivity created
2026-04-23 09:00:02.851 10908-10908 AnrWatchdog             com.scmessenger.android              I  ANR watchdog started (check=5000ms, threshold=10000ms)
2026-04-23 09:00:02.851 10908-10908 MainActivity            com.scmessenger.android              I  ANR watchdog started for UI thread monitoring
2026-04-23 09:00:02.852 10908-10977 MeshRepository          com.scmessenger.android              D  Corruption check: contacts=0, messages=0
2026-04-23 09:00:02.852 10908-10977 MeshRepository          com.scmessenger.android              D  Database integrity check passed
2026-04-23 09:00:02.853 10908-10908 MainActivity            com.scmessenger.android              D  All permissions already granted
2026-04-23 09:00:02.858 10908-10908 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 309578419; UID 10499; state: ENABLED
2026-04-23 09:00:02.858 10908-10908 DesktopModeFlags        com.scmessenger.android              D  Toggle override initialized to: OVERRIDE_UNSET
2026-04-23 09:00:02.870 10908-10977 MeshRepository          com.scmessenger.android              I  Repository background initialization completed
2026-04-23 09:00:02.870 10908-10977 MainActivity            com.scmessenger.android              D  UI components initialization completed
2026-04-23 09:00:02.871 10908-10908 ContentCaptureHelper    com.scmessenger.android              I  Setting logging level to OFF
2026-04-23 09:00:02.871 10908-10979 MeshReposi...aintenance com.scmessenger.android              D  Storage maintenance check: free=17845MB / total=112912MB
2026-04-23 09:00:02.876 10908-10908 MainActivity            com.scmessenger.android              D  MainActivity resumed
2026-04-23 09:00:02.877 10908-10908 MainActivity            com.scmessenger.android              D  All permissions already granted
2026-04-23 09:00:02.877 10908-10908 MeshRepository          com.scmessenger.android              D  Permission refresh skipped: mesh service is not running
2026-04-23 09:00:02.885 10908-10908 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 349153669; UID 10499; state: ENABLED
2026-04-23 09:00:02.982 10908-10908 MainViewModel           com.scmessenger.android              D  MainViewModel init
2026-04-23 09:00:02.983 10908-10979 MainViewMo...rageStatus com.scmessenger.android              D  Storage refreshed: 17845 MB available (Low=false)
2026-04-23 09:00:02.983 10908-10908 MainViewModel           com.scmessenger.android              D  Preference onboardingCompleted: true
2026-04-23 09:00:02.984 10908-10908 MainViewModel           com.scmessenger.android              D  Preference installChoiceCompleted: true
2026-04-23 09:00:02.984 10908-10908 MainViewModel           com.scmessenger.android              D  MeshRepository service state: STOPPED
2026-04-23 09:00:02.985 10908-10979 MainViewMo...ntityState com.scmessenger.android              D  refreshIdentityState() called
2026-04-23 09:00:02.985 10908-10979 MainViewMo...ntityState com.scmessenger.android              D  Identity initialized state: true
2026-04-23 09:00:03.003 10908-10908 VRI[MainActivity]       com.scmessenger.android              D  WindowInsets changed: 1080x2400 statusBars:[0,132,0,0] navigationBars:[0,0,0,63] mandatorySystemGestures:[0,164,0,84] 
2026-04-23 09:00:03.005 10908-10908 WindowOnBackDispatcher  com.scmessenger.android              D  setTopOnBackInvokedCallback (unwrapped): android.view.ViewRootImpl$$ExternalSyntheticLambda25@6b5e2de
2026-04-23 09:00:03.031 10908-10933 ssenger.android         com.scmessenger.android              I  Compiler allocated 5049KB to compile void android.view.ViewRootImpl.performTraversals()
2026-04-23 09:00:03.048 10908-10908 HWUI                    com.scmessenger.android              I  Using FreeType backend (prop=Auto)
2026-04-23 09:00:03.083 10908-10908 MeshServiceViewModel    com.scmessenger.android              D  MeshServiceViewModel initialized
2026-04-23 09:00:03.089 10908-10979 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 09:00:03.089 10908-10977 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 09:00:03.090 10908-10979 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=null, initialized=null, nickname=null
2026-04-23 09:00:03.090 10908-11004 MeshReposi...edDeferred com.scmessenger.android              D  Lazy starting MeshService for Identity access...
2026-04-23 09:00:03.091 10908-11003 MeshReposi...edDeferred com.scmessenger.android              D  MeshService is already starting, skipping redundant init
2026-04-23 09:00:03.091 10908-10977 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=null, initialized=null, nickname=null
2026-04-23 09:00:03.091 10908-10979 DashboardViewModel      com.scmessenger.android              D  Topology built: 0 nodes, 0 edges
2026-04-23 09:00:03.092 10908-10977 DashboardViewModel      com.scmessenger.android              D  Topology built: 0 nodes, 0 edges
2026-04-23 09:00:03.092 10908-10979 DashboardV...efreshData com.scmessenger.android              D  Dashboard data refreshed
2026-04-23 09:00:03.092 10908-11004 MeshRepository          com.scmessenger.android              I  service_start_requested
2026-04-23 09:00:03.092 10908-10977 DashboardV...efreshData com.scmessenger.android              D  Dashboard data refreshed
2026-04-23 09:00:03.092 10908-11004 MeshRepository          com.scmessenger.android              D  Starting MeshService...
2026-04-23 09:00:03.095 10908-10908 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:195849): avc:  denied  { search } for  name="/" dev="cgroup2" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup_v2:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:00:03.095 10908-10908 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:195850): avc:  denied  { search } for  name="/" dev="cgroup2" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup_v2:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:00:03.095 10908-10908 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:195851): avc:  denied  { search } for  name="/" dev="cgroup" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:00:03.095 10908-10908 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:195852): avc:  denied  { search } for  name="/" dev="cgroup" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:00:03.128 10908-11004 MeshRepository          com.scmessenger.android              I  SmartTransportRouter initialized for intelligent transport selection
2026-04-23 09:00:03.133 10908-11004 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:00:03.133 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:00:03.133 10908-11004 NetworkDetector         com.scmessenger.android              I  NetworkDetector monitoring started
2026-04-23 09:00:03.134 10908-11004 MeshRepository          com.scmessenger.android              I  NetworkDetector started — cellular-aware transport fallback active
2026-04-23 09:00:03.134 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:00:03.136 10908-11014 NetworkDetector         com.scmessenger.android              W  Cellular network detected (CELLULAR) — blocking ports: [9001, 9010, 4001, 5001]
2026-04-23 09:00:03.136 10908-10977 MeshReposi...hangeWatch com.scmessenger.android              I  Network type changed: WIFI → CELLULAR, resetting circuit breakers and re-bootstrapping
2026-04-23 09:00:03.137 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: CELLULAR, blocked ports: [9001, 9010, 4001, 5001]
2026-04-23 09:00:03.139 10908-10977 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:00:03.140 10908-11014 NetworkDetector         com.scmessenger.android              W  Cellular network detected (CELLULAR) — blocking ports: [9001, 9010, 4001, 5001]
2026-04-23 09:00:03.143 10908-10977 MeshRepository          com.scmessenger.android              I  Racing bootstrap: network=CELLULAR, transports=wss→tcp→quic→tcp→ws
2026-04-23 09:00:03.143 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: CELLULAR, blocked ports: [9001, 9010, 4001, 5001]
2026-04-23 09:00:03.149 10908-10977 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:00:03.160 10908-10977 MeshReposi...$Companion com.scmessenger.android              I  Using bootstrap source: StaticFallback with 4 nodes
2026-04-23 09:00:03.313 10908-11018 MainViewMo...ntityState com.scmessenger.android              D  refreshIdentityState() called
2026-04-23 09:00:03.318 10908-11018 MainViewMo...ntityState com.scmessenger.android              D  Identity initialized state: true
2026-04-23 09:00:03.367 10908-10908 BootReceiver$onReceive  com.scmessenger.android              D  Auto-start disabled, not starting service
2026-04-23 09:00:03.438 10908-10908 Conversati...adMessages com.scmessenger.android              D  Loaded 0 messages
2026-04-23 09:00:03.440 10908-10908 Conversati...$loadStats com.scmessenger.android              D  Loaded stats: HistoryStats(totalMessages=0, sentCount=0, receivedCount=0, undeliveredCount=0)
2026-04-23 09:00:03.442 10908-10908 Conversati...ockedPeers com.scmessenger.android              D  Loaded 0 blocked peers
2026-04-23 09:00:03.573 10908-10908 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:00:03.573 10908-10908 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:d054e30f: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:00:04.669 10908-10978 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:443 = blocked
2026-04-23 09:00:04.669 10908-11002 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:9010 = blocked
2026-04-23 09:00:04.669 10908-11003 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:443 = blocked
2026-04-23 09:00:04.671 10908-10979 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:9001 = blocked
2026-04-23 09:00:04.691 10908-11018 MeshRepository          com.scmessenger.android              W  All bootstrap addresses failed or timed out, falling back to mDNS
2026-04-23 09:00:04.693 10908-11018 MeshRepository          com.scmessenger.android              I  All relay bootstrap failed, attempting mDNS local discovery
2026-04-23 09:00:05.518 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:00:08.115 10908-11081 ProfileInstaller        com.scmessenger.android              D  Installing profile for com.scmessenger.android
2026-04-23 09:00:08.606 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:00:09.702 10908-11018 MeshRepository          com.scmessenger.android              E  mDNS fallback: no LAN peers discovered within timeout
2026-04-23 09:00:09.715 10908-11018 MeshReposi...hangeWatch com.scmessenger.android              I  Network type changed: CELLULAR → WIFI, resetting circuit breakers and re-bootstrapping
2026-04-23 09:00:09.719 10908-11018 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:00:09.725 10908-11018 MeshRepository          com.scmessenger.android              I  Racing bootstrap: network=WIFI, transports=quic→tcp→wss→ws
2026-04-23 09:00:09.730 10908-11018 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:00:09.740 10908-11018 MeshReposi...$Companion com.scmessenger.android              I  Using bootstrap source: StaticFallback with 4 nodes
2026-04-23 09:00:11.245 10908-11020 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:9010 = blocked
2026-04-23 09:00:11.245 10908-10978 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:9001 = blocked
2026-04-23 09:00:11.246 10908-11003 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:443 = blocked
2026-04-23 09:00:11.246 10908-11018 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:443 = blocked
2026-04-23 09:00:11.282 10908-11003 MeshRepository          com.scmessenger.android              W  All bootstrap addresses failed or timed out, falling back to mDNS
2026-04-23 09:00:11.288 10908-11003 MeshRepository          com.scmessenger.android              I  All relay bootstrap failed, attempting mDNS local discovery
2026-04-23 09:00:14.071 10908-10908 MainViewModel           com.scmessenger.android              D  MeshRepository service state: RUNNING
2026-04-23 09:00:14.071 10908-11004 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:00:14.073 10908-11018 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=0))
2026-04-23 09:00:14.073 10908-11004 MeshRepository          com.scmessenger.android              I  SC_IDENTITY_OWN p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9 pk=374d73ebc21d677f889b806d6065a7177a2c5748f368036bd073e949bba5a2d4
2026-04-23 09:00:14.074 10908-11020 MainViewMo...ntityState com.scmessenger.android              D  refreshIdentityState() called
2026-04-23 09:00:14.075 10908-11004 MeshRepository          com.scmessenger.android              I  Mesh service started successfully
2026-04-23 09:00:14.079 10908-11004 MeshReposi...edDeferred com.scmessenger.android              D  MeshService started lazily
2026-04-23 09:00:14.086 10908-11020 MainViewMo...ntityState com.scmessenger.android              D  Identity initialized state: true
2026-04-23 09:00:14.101 10908-10934 ssenger.android         com.scmessenger.android              W  userfaultfd: MOVE ioctl seems unsupported: Try again
2026-04-23 09:00:14.119 10908-11020 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:00:14.119 10908-11020 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:00:14.120 10908-11003 AdvertiseSettings       com.scmessenger.android              D  setTxPowerLevel: 2
2026-04-23 09:00:14.120 10908-11020 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:00:14.120 10908-11003 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:00:14.120 10908-11003 BluetoothLeAdvertiser   com.scmessenger.android              D  TxPower == ADVERTISE_TX_POWER_MEDIUM
2026-04-23 09:00:14.120 10908-11003 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:00:14.126 10908-11020 BackoffStrategy         com.scmessenger.android              D  Backoff strategy reset
2026-04-23 09:00:14.128 10908-11020 BleScanner              com.scmessenger.android              I  BLE Scanning started (background=false, fallback=false)
2026-04-23 09:00:14.128 10908-11003 BluetoothGattServer     com.scmessenger.android              D  registerCallback()
2026-04-23 09:00:14.129 10908-11003 BluetoothGattServer     com.scmessenger.android              D  registerCallback() - UUID=0cc85d55-b764-427a-b77f-5fa376db7a02
2026-04-23 09:00:14.133 10908-11020 BleScanner              com.scmessenger.android              D  Duty cycle started: 10000ms scan / 30000ms interval
2026-04-23 09:00:14.137 10908-10938 BluetoothGattServer     com.scmessenger.android              D  onServerRegistered(0)
2026-04-23 09:00:14.137 10908-11003 BluetoothGattServer     com.scmessenger.android              D  addService() - service: 0000df01-0000-1000-8000-00805f9b34fb
2026-04-23 09:00:14.138 10908-10938 BluetoothLeScanner      com.scmessenger.android              D  onScannerRegistered(status=0, scannerId=3): mScannerId=0
2026-04-23 09:00:14.143 10908-11003 BleGattServer           com.scmessenger.android              I  GATT server started with SCMessenger service
2026-04-23 09:00:14.147 10908-11003 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (2 bytes)
2026-04-23 09:00:14.150 10908-10938 BluetoothGattServer     com.scmessenger.android              D  onServiceAdded() - handle=147 uuid=0000df01-0000-1000-8000-00805f9b34fb status=0
2026-04-23 09:00:14.154 10908-11003 BleAdvertiser           com.scmessenger.android              D  Identity data set: 313 bytes
2026-04-23 09:00:14.156 10908-11003 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:00:14.159 10908-11003 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:00:14.171 10908-11003 WifiTransportManager    com.scmessenger.android              D  WifiTransportManager initialized
2026-04-23 09:00:14.178 10908-10908 WifiDirect...2pReceiver com.scmessenger.android              D  WiFi P2P state changed: enabled=true
2026-04-23 09:00:14.180 10908-11003 WifiDirectTransport     com.scmessenger.android              I  WiFi Direct started
2026-04-23 09:00:14.181 10908-10908 BleAdverti...seCallback com.scmessenger.android              I  BLE Advertising started successfully
2026-04-23 09:00:14.406 10908-10908 WifiDirect...2pReceiver com.scmessenger.android              D  Disconnected from WiFi P2P group
2026-04-23 09:00:14.414 10908-10908 WifiDirect...2pReceiver com.scmessenger.android              D  This device changed
2026-04-23 09:00:14.419 10908-10908 WifiP2pManager          com.scmessenger.android              D  Ignored { when=0 what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:00:14.419 10908-10908 WifiP2pManager          com.scmessenger.android              D  Ignored { when=0 what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:00:14.419 10908-10908 WifiDirect...eDiscovery com.scmessenger.android              D  Service discovery request added
2026-04-23 09:00:14.424 10908-10908 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-1ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:00:14.424 10908-10908 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-1ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:00:14.425 10908-10908 WifiDirect...terService com.scmessenger.android              D  WiFi Direct service registered: scmessenger
2026-04-23 09:00:14.429 10908-10908 WifiTransp...tDiscovery com.scmessenger.android              I  WiFi P2P Discovery started
2026-04-23 09:00:14.466 10908-10908 WifiDirect...rDiscovery com.scmessenger.android              D  Peer discovery started
2026-04-23 09:00:14.741 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:00:16.299 10908-11020 MeshRepository          com.scmessenger.android              E  mDNS fallback: no LAN peers discovered within timeout
2026-04-23 09:00:17.830 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:00:19.078 10908-11020 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:00:19.087 10908-11018 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=5))
2026-04-23 09:00:19.186 10908-11018 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:00:19.191 10908-11018 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:00:20.905 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:00:23.985 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:00:24.094 10908-11018 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:00:24.102 10908-10977 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=10))
2026-04-23 09:00:24.135 10908-10908 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:00:24.142 10908-10908 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:00:25.824 10908-11003 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:00:25.825 10908-11003 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:00:25.826 10908-11003 MeshRepository          com.scmessenger.android              I  ✓ Internet transport (Swarm) initiated and bridge wired
2026-04-23 09:00:25.827 10908-11003 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:00:25.830 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:00:25.831 10908-11003 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:00:25.832 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:00:25.834 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:00:25.834 10908-11003 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:00:25.835 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:00:25.837 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:00:25.837 10908-11003 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:00:25.838 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:00:25.840 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:00:25.841 10908-11003 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:00:25.842 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:00:25.843 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:00:25.844 10908-11003 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:00:25.845 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:00:25.846 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:00:25.847 10908-11003 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:00:25.848 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:00:25.849 10908-11003 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=1), next attempt in 10000ms
2026-04-23 09:00:29.109 10908-11004 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:00:29.113 10908-11004 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=15))
2026-04-23 09:00:30.173 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:00:30.899 10908-11004 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:00:30.907 10908-11004 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:00:34.127 10908-11004 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:00:34.136 10908-10908 BleScanner...d$Runnable com.scmessenger.android              W  BLE scan fallback enabled after 20015 ms without mesh advertisements; switching to unfiltered scan
2026-04-23 09:00:34.143 10908-11020 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=20))
2026-04-23 09:00:34.147 10908-10908 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:00:34.147 10908-10908 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:00:34.147 10908-10908 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:00:34.157 10908-10908 BleScanner              com.scmessenger.android              I  BLE scan restarted (background=false, fallback=true)
2026-04-23 09:00:34.159 10908-10938 BluetoothLeScanner      com.scmessenger.android              D  onScannerRegistered(status=0, scannerId=3): mScannerId=0
2026-04-23 09:00:36.446 10908-10908 SettingsViewModel       com.scmessenger.android              D  Loaded mesh settings: MeshSettings(relayEnabled=true, maxRelayBudget=200, batteryFloor=20, bleEnabled=true, wifiAwareEnabled=true, wifiDirectEnabled=true, internetEnabled=true, discoveryMode=NORMAL, onionRouting=false, coverTrafficEnabled=false, messagePaddingEnabled=false, timingObfuscationEnabled=false, notificationsEnabled=true, notifyDmEnabled=true, notifyDmRequestEnabled=true, notifyDmInForeground=false, notifyDmRequestInForeground=true, soundEnabled=true, badgeEnabled=true)
2026-04-23 09:00:39.145 10908-11020 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:00:39.148 10908-11020 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=25))
2026-04-23 09:00:41.859 10908-11020 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:00:41.869 10908-11020 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:00:41.874 10908-11020 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:00:41.880 10908-11020 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:00:41.887 10908-11020 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:00:41.890 10908-11020 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:00:41.893 10908-11020 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:00:41.899 10908-11020 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:00:41.902 10908-11020 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:00:41.905 10908-11020 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:00:41.911 10908-11020 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:00:41.913 10908-11020 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:00:41.915 10908-11020 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:00:41.920 10908-11020 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:00:41.921 10908-11020 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:00:41.923 10908-11020 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:00:41.925 10908-11020 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:00:41.926 10908-11020 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:00:41.928 10908-11020 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:00:41.929 10908-11020 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=2), next attempt in 30000ms
2026-04-23 09:00:42.858 10908-10995 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5001ms (threshold: 10000ms)
2026-04-23 09:00:44.155 10908-11020 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:00:44.158 10908-11020 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=30))
2026-04-23 09:00:45.593 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:00:47.154 10908-10908 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:00:47.159 10908-10908 MeshServiceViewModel    com.scmessenger.android              D  MeshServiceViewModel initialized
2026-04-23 09:00:47.289 10908-10908 WindowOnBackDispatcher  com.scmessenger.android              W  OnBackInvokedCallback is not enabled for the application.
                                                                                                    Set 'android:enableOnBackInvokedCallback="true"' in the application manifest.
2026-04-23 09:00:47.431 10908-10938 HWUI                    com.scmessenger.android              I  Davey! duration=10995ms; Flags=0, FrameTimelineVsyncId=75166958, IntendedVsync=282468309525668, Vsync=282468309525668, InputEventId=230028044, HandleInputStart=282468309865378, AnimationStart=282468309890728, PerformTraversalsStart=282479177897691, DrawStart=282479177957994, FrameDeadline=282475324773962, FrameStartTime=282468309862611, FrameInterval=16710050, WorkloadTarget=16600000, SyncQueued=282479303923001, SyncStart=282479303952867, IssueDrawCommandsStart=282479304056302, SwapBuffers=282479305026761, FrameCompleted=282479305448025, DequeueBufferDuration=9725, QueueBufferDuration=112711, GpuCompleted=282479305448025, SwapBuffersCompleted=282479305174181, DisplayPresentTime=282473337056868, CommandSubmissionCompleted=282479305026761, 
2026-04-23 09:00:49.166 10908-11020 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:00:49.175 10908-11020 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=35))
2026-04-23 09:00:51.755 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:00:54.182 10908-11020 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:00:54.193 10908-11004 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=40))
2026-04-23 09:00:57.870 10908-10995 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5001ms (threshold: 10000ms)
2026-04-23 09:00:58.116 10908-10908 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:00:58.175 10908-10908 Choreographer           com.scmessenger.android              I  Skipped 652 frames!  The application may be doing too much work on its main thread.
2026-04-23 09:00:58.224 10908-10938 HWUI                    com.scmessenger.android              I  Davey! duration=10955ms; Flags=0, FrameTimelineVsyncId=75171547, IntendedVsync=282479152100007, Vsync=282490059891791, InputEventId=0, HandleInputStart=282490064221631, AnimationStart=282490064222811, PerformTraversalsStart=282490094932690, DrawStart=282490094978751, FrameDeadline=282479354715285, FrameStartTime=282490063780713, FrameInterval=16666667, WorkloadTarget=16600000, SyncQueued=282490106010652, SyncStart=282490106115714, IssueDrawCommandsStart=282490106342236, SwapBuffers=282490107764193, FrameCompleted=282490108200472, DequeueBufferDuration=13143, QueueBufferDuration=173502, GpuCompleted=282490108200472, SwapBuffersCompleted=282490107984814, DisplayPresentTime=282473370623315, CommandSubmissionCompleted=282490107764193, 
2026-04-23 09:00:58.239 10908-10908 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:00:58.240 10908-10908 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:00:59.197 10908-11004 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:00:59.203 10908-11004 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=45))
2026-04-23 09:01:01.072 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:01:04.213 10908-11004 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:01:04.221 10908-11004 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=50))
2026-04-23 09:01:07.274 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:01:09.237 10908-11004 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:01:09.250 10908-11003 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=55))
2026-04-23 09:01:10.370 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:01:13.953 10908-11003 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:01:13.999 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:01:14.009 10908-11003 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:01:14.017 10908-11003 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:01:14.027 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:01:14.040 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:01:14.045 10908-11003 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:01:14.048 10908-11003 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:01:14.053 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:01:14.061 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:01:14.064 10908-11003 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:01:14.066 10908-11003 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:01:14.068 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:01:14.073 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:01:14.075 10908-11003 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:01:14.077 10908-11003 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:01:14.079 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:01:14.084 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:01:14.085 10908-11003 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:01:14.087 10908-11003 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:01:14.089 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:01:14.093 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:01:14.094 10908-11003 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:01:14.096 10908-11003 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:01:14.098 10908-11003 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:01:14.103 10908-11003 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=3), next attempt in 30000ms
2026-04-23 09:01:14.253 10908-11003 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:01:14.257 10908-11003 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=60))
2026-04-23 09:01:19.265 10908-11003 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:01:19.273 10908-11003 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=65))
2026-04-23 09:01:22.598 10908-10908 MeshServiceViewModel    com.scmessenger.android              D  MeshServiceViewModel initialized
2026-04-23 09:01:22.610 10908-11020 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 09:01:22.645 10908-11003 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 09:01:22.765 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:01:24.280 10908-11095 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:01:24.287 10908-11095 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=70))
2026-04-23 09:01:25.840 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:01:28.250 10908-10908 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:01:28.250 10908-10908 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:01:28.250 10908-10908 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:01:28.776 10908-10908 SettingsViewModel       com.scmessenger.android              D  Loaded mesh settings: MeshSettings(relayEnabled=true, maxRelayBudget=200, batteryFloor=20, bleEnabled=true, wifiAwareEnabled=true, wifiDirectEnabled=true, internetEnabled=true, discoveryMode=NORMAL, onionRouting=false, coverTrafficEnabled=false, messagePaddingEnabled=false, timingObfuscationEnabled=false, notificationsEnabled=true, notifyDmEnabled=true, notifyDmRequestEnabled=true, notifyDmInForeground=false, notifyDmRequestInForeground=true, soundEnabled=true, badgeEnabled=true)
2026-04-23 09:01:28.923 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:01:29.296 10908-11095 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:01:29.305 10908-11018 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=75))
2026-04-23 09:01:32.010 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:01:34.316 10908-11018 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:01:34.817 10908-11020 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:01:34.819 10908-11020 DashboardViewModel      com.scmessenger.android              D  Topology built: 1 nodes, 0 edges
2026-04-23 09:01:34.820 10908-11020 DashboardV...efreshData com.scmessenger.android              D  Dashboard data refreshed
2026-04-23 09:01:34.982 10908-10930 ssenger.android         com.scmessenger.android              I  Thread[2,tid=10930,WaitingInMainSignalCatcherLoop,Thread*=0xb4000076a7917f50,peer=0x2003c78,"Signal Catcher"]: reacting to signal 3
2026-04-23 09:01:34.982 10908-10930 ssenger.android         com.scmessenger.android              I  
2026-04-23 09:01:35.083 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:01:35.214 10908-11003 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:01:35.215 10908-11003 DashboardViewModel      com.scmessenger.android              D  Topology built: 1 nodes, 0 edges
2026-04-23 09:01:35.216 10908-11003 DashboardV...efreshData com.scmessenger.android              D  Dashboard data refreshed
2026-04-23 09:01:35.326 10908-10930 ssenger.android         com.scmessenger.android              I  Wrote stack traces to tombstoned
2026-04-23 09:01:37.889 10908-10995 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5001ms (threshold: 10000ms)
2026-04-23 09:01:38.184 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:01:39.330 10908-11020 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:01:40.283 10908-10908 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:01:40.354 10908-10942 HWUI                    com.scmessenger.android              I  Davey! duration=11622ms; Flags=0, FrameTimelineVsyncId=75174163, IntendedVsync=282520613605796, Vsync=282520613605796, InputEventId=734910611, HandleInputStart=282520624180101, AnimationStart=282520624194505, PerformTraversalsStart=282532182325859, DrawStart=282532182372612, FrameDeadline=282520630205796, FrameStartTime=282520624177253, FrameInterval=16707034, WorkloadTarget=16600000, SyncQueued=282532233820528, SyncStart=282532233852999, IssueDrawCommandsStart=282532233955904, SwapBuffers=282532235630790, FrameCompleted=282532236376192, DequeueBufferDuration=9196, QueueBufferDuration=169718, GpuCompleted=282532236376192, SwapBuffersCompleted=282532235833793, DisplayPresentTime=282515718731856, CommandSubmissionCompleted=282532235630790, 
2026-04-23 09:01:40.374 10908-11095 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=80))
2026-04-23 09:01:40.374 10908-11020 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=85))
2026-04-23 09:01:44.335 10908-11020 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:01:44.338 10908-11095 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=90))
2026-04-23 09:01:46.119 10908-11095 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:01:46.125 10908-11095 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:01:46.129 10908-11095 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:01:46.133 10908-11095 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:01:46.137 10908-11095 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:01:46.142 10908-11095 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:01:46.146 10908-11095 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:01:46.151 10908-11095 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:01:46.155 10908-11095 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:01:46.156 10908-11095 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:01:46.157 10908-11095 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:01:46.158 10908-11095 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:01:46.160 10908-11095 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:01:46.160 10908-11095 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=4), next attempt in 16000ms
2026-04-23 09:01:47.379 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:01:48.546  1619-11532 ActivityManager         system_server                        E  ANR in com.scmessenger.android (com.scmessenger.android/.ui.MainActivity)
                                                                                                    PID: 10908
                                                                                                    Reason: Input dispatching timed out (4916c72 com.scmessenger.android/com.scmessenger.android.ui.MainActivity is not responding. Waited 5003ms for MotionEvent).
                                                                                                    Parent: com.scmessenger.android/.ui.MainActivity
                                                                                                    ErrorId: b18281c1-d5d4-4561-a277-3332f00b6f46
                                                                                                    Frozen: false
                                                                                                    Load: 2.86 / 3.49 / 3.16
                                                                                                    ----- Output from /proc/pressure/memory -----
                                                                                                    some avg10=0.15 avg60=0.32 avg300=0.35 total=1516856552
                                                                                                    full avg10=0.15 avg60=0.24 avg300=0.24 total=877403549
                                                                                                    ----- End output from /proc/pressure/memory -----
                                                                                                    ----- Output from /proc/pressure/cpu -----
                                                                                                    some avg10=2.61 avg60=4.47 avg300=6.46 total=26856931058
                                                                                                    full avg10=0.00 avg60=0.00 avg300=0.00 total=0
                                                                                                    ----- End output from /proc/pressure/cpu -----
                                                                                                    ----- Output from /proc/pressure/io -----
                                                                                                    some avg10=0.02 avg60=1.40 avg300=1.81 total=4759760879
                                                                                                    full avg10=0.01 avg60=0.84 avg300=1.05 total=2476764073
                                                                                                    ----- End output from /proc/pressure/io -----
                                                                                                    
                                                                                                    CPU usage from 0ms to 13538ms later (2026-04-23 09:01:34.950 to 2026-04-23 09:01:48.488):
                                                                                                      70% 10908/com.scmessenger.android: 68% user + 2.5% kernel / faults: 10963 minor 127 major
                                                                                                      21% 1619/system_server: 10% user + 11% kernel / faults: 10041 minor 1138 major
                                                                                                      4.5% 546/surfaceflinger: 2.8% user + 1.6% kernel / faults: 27 minor
                                                                                                      4% 2639/com.android.phone: 2.7% user + 1.3% kernel / faults: 3479 minor 426 major
                                                                                                      3.1% 2261/com.android.systemui: 1.9% user + 1.1% kernel / faults: 2967 minor 2393 major
                                                                                                      2.9% 19870/com.life360.android.safetymapd: 1.8% user + 1.1% kernel / faults: 619 minor 542 major
                                                                                                      2.5% 8374/com.google.android.gms.persistent: 1.9% user + 0.5% kernel / faults: 1654 minor 192 major
                                                                                                      2.3% 10769/com.google.android.apps.messaging: 1.9% user + 0.4% kernel / faults: 1848 minor 2 major
                                                                                                      2.2% 11332/com.lemon.lvoverseas: 1% user + 1.2% kernel / faults: 621 minor 182 major
                                                                                                      2.1% 92/kswapd0: 0% user + 2.1% kernel
                                                                                                    20% TOTAL: 13% user + 5.8% kernel + 0.3% iowait + 0.8% irq + 0.4% softirq
                                                                                                    CPU usage from 48ms to 595ms later (2026-04-23 09:01:34.998 to 2026-04-23 09:01:35.544):
                                                                                                      111% 1619/system_server: 34% user + 77% kernel / faults: 3057 minor 777 major
                                                                                                        55% 11534/AnrAuxiliaryTas: 14% user + 40% kernel
                                                                                                        23% 1630/Signal Catcher: 12% user + 10% kernel
                                                                                                        12% 11532/AnrConsumer: 0% user + 12% kernel
                                                                                                        2.1% 1619/system_server: 2.1% user + 0% kernel
                                                                                                        2.1% 1676/android.ui: 2.1% user + 0% kernel
                                                                                                        2.1% 2453/MobileDataStats: 2.1% user + 0% kernel
                                                                                                        2.1% 3764/binder:1619_12: 2.1% user + 0% kernel
                                                                                                      116% 10908/com.scmessenger.android: 106% user + 9.7% kernel / faults: 505 minor 15 major
                                                                                                        100% 10908/ssenger.android: 100% user + 0% kernel
                                                                                                        16% 10930/Signal Catcher: 6.4% user + 9.7% kernel
                                                                                                      3.9% 546/surfaceflinger: 1.9% user + 1.9% kernel / faults: 9 minor
                                                                                                      1.8% 63/ksoftirqd/6: 0% user + 1.8% kernel
                                                                                                      1.8% 70/ksoftirqd/7: 0% user + 1.8% kernel
                                                                                                      1.8% 73/rcuop/7: 0% user + 1.8% kernel
                                                                                                      1.8% 92/kswapd0: 0% user + 1.8% kernel
                                                                                                      1.8% 275/decon0_kthread: 0% user + 1.8% kernel
                                                                                                      1.9% 487/sugov:0: 0% user + 1.9% kernel
                                                                                                      2% 1222/android.hardware.sensors-service.multihal: 2% user + 0% kernel
                                                                                                      2% 1254/audioserver: 2% user + 0% kernel
                                                                                                      2% 1307/kworker/7:2-events: 0% user + 2% kernel
                                                                                                      2.1% 1484/adbd: 0% user + 2.1% kernel
                                                                                                      3.3% 11332/com.lemon.lvoverseas: 0% user + 3.3% kernel / faults: 4 minor
                                                                                                        3.3% 11464/NPTH-AnrInfoPol: 0% user + 3.3% kernel
                                                                                                      3.3% 11462/kworker/u16:0-async_vote_wq: 0% user + 3.3% kernel
                                                                                                      3.9% 30178/kworker/4:0H-kblockd: 0% user + 3.9% kernel
                                                                                                    42% TOTAL: 24% user + 15% kernel + 0.9% iowait + 0.6% irq + 0.6% softirq
2026-04-23 09:01:49.342 10908-11095 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:01:49.346 10908-11095 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=95))
2026-04-23 09:01:50.460 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:01:52.895 10908-10995 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5001ms (threshold: 10000ms)
2026-04-23 09:01:53.500 10908-10930 ssenger.android         com.scmessenger.android              I  Thread[2,tid=10930,WaitingInMainSignalCatcherLoop,Thread*=0xb4000076a7917f50,peer=0x20039e0,"Signal Catcher"]: reacting to signal 3
2026-04-23 09:01:53.500 10908-10930 ssenger.android         com.scmessenger.android              I  
2026-04-23 09:01:53.552 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:01:53.694 10908-10930 ssenger.android         com.scmessenger.android              I  Wrote stack traces to tombstoned
2026-04-23 09:01:54.347 10908-11095 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:01:56.641  1619-11666 ActivityManager         system_server                        E  ANR in com.scmessenger.android (com.scmessenger.android/.ui.MainActivity)
                                                                                                    PID: 10908
                                                                                                    Reason: Input dispatching timed out (4916c72 com.scmessenger.android/com.scmessenger.android.ui.MainActivity is not responding. Waited 5001ms for MotionEvent).
                                                                                                    Parent: com.scmessenger.android/.ui.MainActivity
                                                                                                    ErrorId: c0d1633a-334f-4702-a812-028a602f25f0
                                                                                                    Frozen: false
                                                                                                    Load: 2.69 / 3.41 / 3.14
                                                                                                    ----- Output from /proc/pressure/memory -----
                                                                                                    some avg10=0.18 avg60=0.29 avg300=0.34 total=1516913700
                                                                                                    full avg10=0.18 avg60=0.23 avg300=0.23 total=877455079
                                                                                                    ----- End output from /proc/pressure/memory -----
                                                                                                    ----- Output from /proc/pressure/cpu -----
                                                                                                    some avg10=4.25 avg60=4.47 avg300=6.34 total=26857873409
                                                                                                    full avg10=0.00 avg60=0.00 avg300=0.00 total=0
                                                                                                    ----- End output from /proc/pressure/cpu -----
                                                                                                    ----- Output from /proc/pressure/io -----
                                                                                                    some avg10=0.16 avg60=1.09 avg300=1.72 total=4759855538
                                                                                                    full avg10=0.16 avg60=0.67 avg300=1.00 total=2476831195
                                                                                                    ----- End output from /proc/pressure/io -----
                                                                                                    
                                                                                                    CPU usage from 5006ms to 0ms ago (2026-04-23 09:01:48.488 to 2026-04-23 09:01:53.494):
                                                                                                      122% 10908/com.scmessenger.android: 115% user + 7.8% kernel / faults: 2496 minor
                                                                                                      21% 546/surfaceflinger: 14% user + 7.2% kernel / faults: 77 minor 66 major
                                                                                                      21% 1619/system_server: 13% user + 8.2% kernel / faults: 3558 minor 770 major
                                                                                                      8.6% 548/android.hardware.graphics.composer@2.4-service: 4.6% user + 4% kernel / faults: 4 major
                                                                                                      7% 365/ueventd: 6% user + 1% kernel
                                                                                                      5.2% 1223/android.hardware.usb-service.gs101: 4.8% user + 0.4% kernel
                                                                                                      3.6% 609/mali_jd_thread: 0% user + 3.6% kernel
                                                                                                      3.4% 1993/irq/439-fts: 0% user + 3.4% kernel
                                                                                                      2.8% 1484/adbd: 1% user + 1.8% kernel / faults: 363 minor
                                                                                                      2.8% 2261/com.android.systemui: 2.2% user + 0.6% kernel / faults: 127 minor 59 major
                                                                                                    30% TOTAL: 21% user + 6.6% kernel + 0.1% iowait + 1.5% irq + 0.5% softirq
                                                                                                    CPU usage from 28ms to 584ms later (2026-04-23 09:01:53.521 to 2026-04-23 09:01:54.078):
                                                                                                      139% 1619/system_server: 66% user + 72% kernel / faults: 4054 minor 30 major
                                                                                                        75% 1630/Signal Catcher: 49% user + 25% kernel
                                                                                                        53% 11667/AnrAuxiliaryTas: 10% user + 42% kernel
                                                                                                      100% 10908/com.scmessenger.android: 100% user + 0% kernel
                                                                                                        100% 10908/ssenger.android: 100% user + 0% kernel
                                                                                                      1.8% 63/ksoftirqd/6: 0% user + 1.8% kernel
                                                                                                      1.8% 275/decon0_kthread: 0% user + 1.8% kernel
                                                                                                      2.5% 5648/kworker/u17:4-ufs_clk_gating_0: 0% user + 2.5% kernel
                                                                                                      2.8% 8374/com.google.android.gms.persistent: 2.8% user + 0% kernel / faults: 10 minor 4 major
                                                                                                      3.6% 19870/com.life360.android.safetymapd: 0% user + 3.6% kernel / faults: 2 minor
                                                                                                    37% TOTAL: 23% user + 12% kernel + 0.2% iowait + 0.4% irq + 0.4% softirq
2026-04-23 09:01:56.644  1619-1676  ActivityManager         system_server                        E  App already has anr dialog: ProcessRecord{792aa2c 10908:com.scmessenger.android/u0a499}
2026-04-23 09:01:56.657 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:01:57.051 10908-10908 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:01:57.059 10908-10908 Choreographer           com.scmessenger.android              I  Skipped 651 frames!  The application may be doing too much work on its main thread.
2026-04-23 09:01:57.103 10908-10977 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=100))
2026-04-23 09:01:57.109 10908-10942 HWUI                    com.scmessenger.android              I  Davey! duration=10918ms; Flags=0, FrameTimelineVsyncId=75175022, IntendedVsync=282538069333669, Vsync=282548943637669, InputEventId=545120960, HandleInputStart=282548947995585, AnimationStart=282548964089254, PerformTraversalsStart=282548983761088, DrawStart=282548983795919, FrameDeadline=282544984889349, FrameStartTime=282548947616679, FrameInterval=16705042, WorkloadTarget=16600000, SyncQueued=282548985567403, SyncStart=282548985626200, IssueDrawCommandsStart=282548985731181, SwapBuffers=282548987135722, FrameCompleted=282548987650574, DequeueBufferDuration=9115, QueueBufferDuration=149048, GpuCompleted=282548987650574, SwapBuffersCompleted=282548987319845, DisplayPresentTime=282542996978940, CommandSubmissionCompleted=282548987135722, 
2026-04-23 09:01:57.145 10908-10908 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:01:57.145 10908-10908 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:bd57c61e: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:01:58.253 10908-10908 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:01:58.253 10908-10908 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:01:58.254 10908-10908 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:01:59.354 10908-10977 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:01:59.363 10908-11095 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=105))
2026-04-23 09:01:59.767 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:02:02.173 10908-11095 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:02:02.189 10908-11095 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:02:02.198 10908-11095 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:02:02.205 10908-11095 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:02:02.213 10908-11095 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:02:02.220 10908-11095 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:02:02.226 10908-11095 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:02:02.231 10908-11095 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:02:02.237 10908-11095 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:02:02.243 10908-11095 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:02:02.247 10908-11095 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:02:02.252 10908-11095 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:02:02.255 10908-11095 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:02:02.258 10908-11095 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=5), next attempt in 32000ms
2026-04-23 09:02:02.855 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:02:04.369 10908-11095 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:02:04.376 10908-11095 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=110))
2026-04-23 09:02:09.029 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:02:09.384 10908-11095 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:02:09.391 10908-11095 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=115))
2026-04-23 09:02:10.523 10908-10908 SettingsViewModel       com.scmessenger.android              D  Loaded mesh settings: MeshSettings(relayEnabled=true, maxRelayBudget=200, batteryFloor=20, bleEnabled=true, wifiAwareEnabled=true, wifiDirectEnabled=true, internetEnabled=true, discoveryMode=NORMAL, onionRouting=false, coverTrafficEnabled=false, messagePaddingEnabled=false, timingObfuscationEnabled=false, notificationsEnabled=true, notifyDmEnabled=true, notifyDmRequestEnabled=true, notifyDmInForeground=false, notifyDmRequestInForeground=true, soundEnabled=true, badgeEnabled=true)
2026-04-23 09:02:12.103 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:02:14.398 10908-11095 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:02:14.406 10908-11020 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=120))
2026-04-23 09:02:17.909 10908-10995 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5001ms (threshold: 10000ms)
2026-04-23 09:02:18.269 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:02:19.416 10908-11020 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:02:20.822 10908-10908 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:02:20.875 10908-11020 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=125))
2026-04-23 09:02:20.885 10908-10941 HWUI                    com.scmessenger.android              I  Davey! duration=10386ms; Flags=0, FrameTimelineVsyncId=75181246, IntendedVsync=282562376742466, Vsync=282562376742466, InputEventId=89274519, HandleInputStart=282562377346951, AnimationStart=282562377383450, PerformTraversalsStart=282572720208569, DrawStart=282572720245922, FrameDeadline=282562393342466, FrameStartTime=282562377340318, FrameInterval=16678200, WorkloadTarget=16600000, SyncQueued=282572761962434, SyncStart=282572762107698, IssueDrawCommandsStart=282572762208284, SwapBuffers=282572762755891, FrameCompleted=282572763470491, DequeueBufferDuration=8789, QueueBufferDuration=132161, GpuCompleted=282572763470491, SwapBuffersCompleted=282572762917065, DisplayPresentTime=282543765359556, CommandSubmissionCompleted=282572762755891, 
2026-04-23 09:02:21.334 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:02:24.444 10908-11020 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:02:24.460 10908-11095 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=130))
2026-04-23 09:02:28.260 10908-10908 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:02:28.261 10908-10908 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:02:28.262 10908-10908 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:02:29.468 10908-11095 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:02:29.481 10908-10977 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=135))
2026-04-23 09:02:30.655 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:02:34.277 10908-10977 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:02:34.287 10908-10977 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:02:34.293 10908-10977 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:02:34.298 10908-10977 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:02:34.304 10908-10977 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:02:34.310 10908-10977 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:02:34.314 10908-10977 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:02:34.320 10908-10977 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:02:34.325 10908-10977 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:02:34.329 10908-10977 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:02:34.334 10908-10977 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:02:34.336 10908-10977 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:02:34.338 10908-10977 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:02:34.339 10908-10977 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=6), next attempt in 60000ms
2026-04-23 09:02:34.480 10908-10977 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:02:34.482 10908-10977 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=140))
2026-04-23 09:02:39.485 10908-10977 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:02:39.491 10908-11020 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=145))
2026-04-23 09:02:44.499 10908-11020 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:02:44.513 10908-11095 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=150))
2026-04-23 09:02:44.573 10908-10908 WifiDirect...2pReceiver com.scmessenger.android              D  This device changed
2026-04-23 09:02:46.110 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:02:49.227 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:02:49.515 10908-11095 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:02:49.521 10908-10977 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=155))
2026-04-23 09:02:54.529 10908-10977 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:02:54.540 10908-10977 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=160))
2026-04-23 09:02:58.278 10908-10908 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:02:58.278 10908-10908 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:02:58.280 10908-10908 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:02:58.503 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:02:59.555 10908-10977 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:02:59.571 10908-11095 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=165))
2026-04-23 09:03:01.597 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:03:04.582 10908-11095 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:03:04.597 10908-11020 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=170))
2026-04-23 09:03:07.806 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:03:09.618 10908-11020 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:03:09.642 10908-10977 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=175))
2026-04-23 09:03:10.913 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:03:14.661 10908-10977 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:03:14.686 10908-11095 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=180))
2026-04-23 09:03:19.708 10908-11095 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:03:19.738 10908-11020 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=185))
2026-04-23 09:03:24.757 10908-11020 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:03:24.786 10908-10977 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=190))
2026-04-23 09:03:26.418 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:03:28.295 10908-10908 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:03:28.295 10908-10908 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:03:28.296 10908-10908 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:03:29.804 10908-10977 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:03:29.830 10908-11095 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=195))
2026-04-23 09:03:32.618 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:03:34.834 10908-11095 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:03:34.842 10908-11020 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=200))
2026-04-23 09:03:38.401 10908-11020 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:03:38.427 10908-11020 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:03:38.444 10908-11020 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:03:38.460 10908-11020 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:03:38.476 10908-11020 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:03:38.483 10908-11020 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:03:38.489 10908-11020 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:03:38.495 10908-11020 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:03:38.501 10908-11020 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:03:38.506 10908-11020 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:03:38.510 10908-11020 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:03:38.514 10908-11020 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:03:38.518 10908-11020 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:03:38.522 10908-11020 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=7), next attempt in 60000ms
2026-04-23 09:03:38.826 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:03:39.845 10908-11020 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:03:39.859 10908-11020 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=205))
2026-04-23 09:03:41.977 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:03:44.859 10908-11020 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:03:44.863 10908-11020 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=210))
2026-04-23 09:03:49.873 10908-11020 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:03:49.884 10908-11020 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=215))
2026-04-23 09:03:54.374 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:03:54.893 10908-11020 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:03:54.906 10908-10977 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=220))
2026-04-23 09:03:57.469 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:03:58.309 10908-10908 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:03:58.309 10908-10908 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:03:58.310 10908-10908 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:03:59.925 10908-10977 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:03:59.952 10908-11095 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=225))
2026-04-23 09:04:03.645 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:04:04.967 10908-11095 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:04:04.992 10908-11020 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=230))
2026-04-23 09:04:06.744 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:04:10.005 10908-11020 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:04:10.021 10908-10977 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=236))
2026-04-23 09:04:12.942 10908-11014 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:04:14.962 10908-10908 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 312399441; UID 10499; state: ENABLED
2026-04-23 09:04:14.988 10908-10908 AndroidRuntime          com.scmessenger.android              E  FATAL EXCEPTION: main (Fix with AI)
                                                                                                    Process: com.scmessenger.android, PID: 10908
                                                                                                    java.lang.IllegalArgumentException: Failed to find configured root that contains /data/data/com.scmessenger.android/cache/scmessenger_diagnostics_bundle.txt
                                                                                                    	at androidx.core.content.FileProvider$SimplePathStrategy.getUriForFile(FileProvider.java:867)
                                                                                                    	at androidx.core.content.FileProvider.getUriForFile(FileProvider.java:467)
                                                                                                    	at com.scmessenger.android.ui.screens.DiagnosticsScreenKt.shareDiagnosticsBundle(DiagnosticsScreen.kt:154)
                                                                                                    	at com.scmessenger.android.ui.screens.DiagnosticsScreenKt.access$shareDiagnosticsBundle(DiagnosticsScreen.kt:1)
                                                                                                    	at com.scmessenger.android.ui.screens.DiagnosticsScreenKt$DiagnosticsScreen$4$2$3.invoke(DiagnosticsScreen.kt:83)
                                                                                                    	at com.scmessenger.android.ui.screens.DiagnosticsScreenKt$DiagnosticsScreen$4$2$3.invoke(DiagnosticsScreen.kt:83)
                                                                                                    	at androidx.compose.foundation.ClickableNode$clickPointerInput$3.invoke-k-4lQ0M(Clickable.kt:639)
                                                                                                    	at androidx.compose.foundation.ClickableNode$clickPointerInput$3.invoke(Clickable.kt:633)
                                                                                                    	at androidx.compose.foundation.gestures.TapGestureDetectorKt$detectTapAndPress$2$1.invokeSuspend(TapGestureDetector.kt:255)
                                                                                                    	at kotlin.coroutines.jvm.internal.BaseContinuationImpl.resumeWith(ContinuationImpl.kt:33)
                                                                                                    	at kotlinx.coroutines.DispatchedTaskKt.resume(DispatchedTask.kt:179)
                                                                                                    	at kotlinx.coroutines.DispatchedTaskKt.dispatch(DispatchedTask.kt:168)
                                                                                                    	at kotlinx.coroutines.CancellableContinuationImpl.dispatchResume(CancellableContinuationImpl.kt:474)
                                                                                                    	at kotlinx.coroutines.CancellableContinuationImpl.resumeImpl(CancellableContinuationImpl.kt:508)
                                                                                                    	at kotlinx.coroutines.CancellableContinuationImpl.resumeImpl$default(CancellableContinuationImpl.kt:497)
                                                                                                    	at kotlinx.coroutines.CancellableContinuationImpl.resumeWith(CancellableContinuationImpl.kt:368)
                                                                                                    	at androidx.compose.ui.input.pointer.SuspendingPointerInputModifierNodeImpl$PointerEventHandlerCoroutine.offerPointerEvent(SuspendingPointerInputFilter.kt:719)
                                                                                                    	at androidx.compose.ui.input.pointer.SuspendingPointerInputModifierNodeImpl.dispatchPointerEvent(SuspendingPointerInputFilter.kt:598)
                                                                                                    	at androidx.compose.ui.input.pointer.SuspendingPointerInputModifierNodeImpl.onPointerEvent-H0pRuoY(SuspendingPointerInputFilter.kt:620)
                                                                                                    	at androidx.compose.foundation.AbstractClickableNode.onPointerEvent-H0pRuoY(Clickable.kt:1044)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:387)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:373)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:373)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:373)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:373)
                                                                                                    	at androidx.compose.ui.input.pointer.NodeParent.dispatchMainEventPass(HitPathTracker.kt:229)
                                                                                                    	at androidx.compose.ui.input.pointer.HitPathTracker.dispatchChanges(HitPathTracker.kt:144)
                                                                                                    	at androidx.compose.ui.input.pointer.PointerInputEventProcessor.process-BIzXfog(PointerInputEventProcessor.kt:120)
                                                                                                    	at androidx.compose.ui.platform.AndroidComposeView.sendMotionEvent-8iAsVTc(AndroidComposeView.android.kt:1994)
                                                                                                    	at androidx.compose.ui.platform.AndroidComposeView.handleMotionEvent-8iAsVTc(AndroidComposeView.android.kt:1945)
                                                                                                    	at androidx.compose.ui.platform.AndroidComposeView.dispatchTouchEvent(AndroidComposeView.android.kt:1829)
                                                                                                    	at android.view.ViewGroup.dispatchTransformedTouchEvent(ViewGroup.java:3143)
                                                                                                    	at android.view.ViewGroup.dispatchTouchEvent(ViewGroup.java:2826)
                                                                                                    	at android.view.ViewGroup.dispatchTransformedTouchEvent(ViewGroup.java:3143)
                                                                                                    	at android.view.ViewGroup.dispatchTouchEvent(ViewGroup.java:2826)
                                                                                                    	at android.view.ViewGroup.dispatchTransformedTouchEvent(ViewGroup.java:3143)
                                                                                                    	at android.view.ViewGroup.dispatchTouchEvent(ViewGroup.java:2826)
                                                                                                    	at android.view.ViewGroup.dispatchTransformedTouchEvent(ViewGroup.java:3143)
                                                                                                    	at android.view.ViewGroup.dispatchTouchEvent(ViewGroup.java:2826)
2026-04-23 09:04:14.990 10908-10908 AndroidRuntime          com.scmessenger.android              E  	at com.android.internal.policy.DecorView.superDispatchTouchEvent(DecorView.java:503) (Fix with AI)
                                                                                                    	at com.android.internal.policy.PhoneWindow.superDispatchTouchEvent(PhoneWindow.java:2017)
                                                                                                    	at android.app.Activity.dispatchTouchEvent(Activity.java:4666)
                                                                                                    	at com.android.internal.policy.DecorView.dispatchTouchEvent(DecorView.java:441)
                                                                                                    	at android.view.View.dispatchPointerEvent(View.java:17196)
                                                                                                    	at android.view.ViewRootImpl$ViewPostImeInputStage.processPointerEvent(ViewRootImpl.java:8585)
                                                                                                    	at android.view.ViewRootImpl$ViewPostImeInputStage.onProcess(ViewRootImpl.java:8335)
                                                                                                    	at android.view.ViewRootImpl$InputStage.deliver(ViewRootImpl.java:7702)
                                                                                                    	at android.view.ViewRootImpl$InputStage.onDeliverToNext(ViewRootImpl.java:7759)
                                                                                                    	at android.view.ViewRootImpl$InputStage.forward(ViewRootImpl.java:7725)
                                                                                                    	at android.view.ViewRootImpl$AsyncInputStage.forward(ViewRootImpl.java:7896)
                                                                                                    	at android.view.ViewRootImpl$InputStage.apply(ViewRootImpl.java:7733)
                                                                                                    	at android.view.ViewRootImpl$AsyncInputStage.apply(ViewRootImpl.java:7953)
                                                                                                    	at android.view.ViewRootImpl$InputStage.deliver(ViewRootImpl.java:7706)
                                                                                                    	at android.view.ViewRootImpl$InputStage.onDeliverToNext(ViewRootImpl.java:7759)
                                                                                                    	at android.view.ViewRootImpl$InputStage.forward(ViewRootImpl.java:7725)
                                                                                                    	at android.view.ViewRootImpl$InputStage.apply(ViewRootImpl.java:7733)
                                                                                                    	at android.view.ViewRootImpl$InputStage.deliver(ViewRootImpl.java:7706)
                                                                                                    	at android.view.ViewRootImpl.deliverInputEvent(ViewRootImpl.java:11033)
                                                                                                    	at android.view.ViewRootImpl.doProcessInputEvents(ViewRootImpl.java:10973)
                                                                                                    	at android.view.ViewRootImpl.enqueueInputEvent(ViewRootImpl.java:10941)
                                                                                                    	at android.view.ViewRootImpl.processRawInputEvent(ViewRootImpl.java:11377)
                                                                                                    	at android.view.ViewRootImpl$WindowInputEventReceiver.onInputEvent(ViewRootImpl.java:11161)
                                                                                                    	at android.view.InputEventReceiver.dispatchInputEvent(InputEventReceiver.java:284)
                                                                                                    	at android.os.MessageQueue.nativePollOnce(Native Method)
                                                                                                    	at android.os.MessageQueue.nextDeliQueue(MessageQueue.java:780)
                                                                                                    	at android.os.MessageQueue.next(MessageQueue.java:760)
                                                                                                    	at android.os.Looper.loopOnce(Looper.java:196)
                                                                                                    	at android.os.Looper.loop(Looper.java:367)
                                                                                                    	at android.app.ActivityThread.main(ActivityThread.java:9333)
                                                                                                    	at java.lang.reflect.Method.invoke(Native Method)
                                                                                                    	at com.android.internal.os.RuntimeInit$MethodAndArgsCaller.run(RuntimeInit.java:566)
                                                                                                    	at com.android.internal.os.ZygoteInit.main(ZygoteInit.java:929)
                                                                                                    	Suppressed: kotlinx.coroutines.internal.DiagnosticCoroutineContextException: [androidx.compose.ui.platform.MotionDurationScaleImpl@d5c2735, androidx.compose.runtime.BroadcastFrameClock@c2d07ca, StandaloneCoroutine{Cancelling}@7c80d3b, AndroidUiDispatcher@ad7cc58]
2026-04-23 09:04:15.002 10908-10908 Process                 com.scmessenger.android              I  Sending signal. PID: 10908 SIG: 9
---------------------------- PROCESS ENDED (10908) for package com.scmessenger.android ----------------------------
---------------------------- PROCESS STARTED (12114) for package com.scmessenger.android ----------------------------
2026-04-23 09:04:16.545 12114-12114 Zygote                  com.scmessenger.android              I  Process 12114 created for com.scmessenger.android
2026-04-23 09:04:16.545 12114-12114 ssenger.android         com.scmessenger.android              I  Late-enabling -Xcheck:jni
2026-04-23 09:04:16.572 12114-12114 ssenger.android         com.scmessenger.android              I  Using CollectorTypeCMC GC.
2026-04-23 09:04:16.576 12114-12114 nativeloader            com.scmessenger.android              D  Load libframework-connectivity-tiramisu-jni.so using APEX ns com_android_tethering for caller /apex/com.android.tethering/javalib/framework-connectivity-t.jar: ok
2026-04-23 09:04:16.918 12114-12114 nativeloader            com.scmessenger.android              D  Configuring clns-9 for other apk /data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk. target_sdk_version=35, uses_libraries=, library_path=/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/lib/arm64:/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk!/lib/arm64-v8a, permitted_path=/data:/mnt/expand:/data/user/0/com.scmessenger.android
2026-04-23 09:04:16.919 12114-12114 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 202956589; UID 10499; state: ENABLED
2026-04-23 09:04:16.926 12114-12114 GraphicsEnvironment     com.scmessenger.android              V  Currently set values for:
2026-04-23 09:04:16.926 12114-12114 GraphicsEnvironment     com.scmessenger.android              V    angle_gl_driver_selection_pkgs=[com.android.angle]
2026-04-23 09:04:16.926 12114-12114 GraphicsEnvironment     com.scmessenger.android              V    angle_gl_driver_selection_values=[angle]
2026-04-23 09:04:16.926 12114-12114 GraphicsEnvironment     com.scmessenger.android              V  com.scmessenger.android is not listed in per-application setting
2026-04-23 09:04:16.926 12114-12114 GraphicsEnvironment     com.scmessenger.android              V  No special selections for ANGLE, returning default driver choice
2026-04-23 09:04:16.926 12114-12114 GraphicsEnvironment     com.scmessenger.android              V  Neither updatable production driver nor prerelease driver is supported.
2026-04-23 09:04:16.944 12114-12114 WM-WrkMgrInitializer    com.scmessenger.android              D  Initializing WorkManager with default configuration.
2026-04-23 09:04:16.975 12114-12114 MeshApplication         com.scmessenger.android              I  SCMessenger application started
2026-04-23 09:04:16.976 12114-12138 StorageManager          com.scmessenger.android              D  Performing startup storage maintenance...
2026-04-23 09:04:16.985 12114-12138 StorageManager          com.scmessenger.android              D  Logs rotated successfully.
2026-04-23 09:04:16.987 12114-12141 DisplayManager          com.scmessenger.android              I  Choreographer implicitly registered for the refresh rate.
2026-04-23 09:04:16.987 12114-12138 StorageManager          com.scmessenger.android              D  Clearing cache (51 KB)...
2026-04-23 09:04:16.988 12114-12141 vulkan                  com.scmessenger.android              D  searching for layers in '/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/lib/arm64'
2026-04-23 09:04:16.989 12114-12141 vulkan                  com.scmessenger.android              D  searching for layers in '/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk!/lib/arm64-v8a'
2026-04-23 09:04:16.995 12114-12114 DesktopExperienceFlags  com.scmessenger.android              D  Toggle override initialized to: false
2026-04-23 09:04:16.996 12114-12138 StorageManager          com.scmessenger.android              D  Startup maintenance complete. Available storage: 17870 MB
2026-04-23 09:04:17.000 12114-12138 MeshApplic...n$onCreate com.scmessenger.android              I  Startup storage maintenance completed
2026-04-23 09:04:17.017 12114-12114 MeshRepository          com.scmessenger.android              D  MeshRepository initialized with storage: /data/user/0/com.scmessenger.android/files
2026-04-23 09:04:17.021 12114-12114 MeshRepository          com.scmessenger.android              D  AND-CONTACTS-WIPE-001: Reinstall check - hasIdentityBackup=true, contactsOnDisk=true, historyOnDisk=true
2026-04-23 09:04:17.022 12114-12114 MeshRepository          com.scmessenger.android              D  AND-CONTACTS-WIPE-001: Normal startup - all data present
2026-04-23 09:04:17.024 12114-12114 MeshRepository          com.scmessenger.android              D  Contacts migration already completed, skipping
2026-04-23 09:04:17.029 12114-12114 nativeloader            com.scmessenger.android              D  Load /data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/lib/arm64/libjnidispatch.so using class loader ns clns-9 (caller=/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk!classes17.dex): ok
2026-04-23 09:04:17.159 12114-12114 MeshRepository          com.scmessenger.android              I  all_managers_init_success
2026-04-23 09:04:17.160 12114-12114 MeshRepository          com.scmessenger.android              I  All managers initialized successfully
2026-04-23 09:04:17.162 12114-12114 MeshRepository          com.scmessenger.android              D  AND-CONTACTS-WIPE-001: Contact data verification - Found 0 contacts
2026-04-23 09:04:17.162 12114-12114 MeshRepository          com.scmessenger.android              W  AND-CONTACTS-WIPE-001: WARNING - No contacts found. If this is unexpected, data recovery may be needed.
2026-04-23 09:04:17.163 12114-12114 MainActivity            com.scmessenger.android              D  MainActivity created
2026-04-23 09:04:17.164 12114-12114 AnrWatchdog             com.scmessenger.android              I  ANR watchdog started (check=5000ms, threshold=10000ms)
2026-04-23 09:04:17.165 12114-12114 MainActivity            com.scmessenger.android              I  ANR watchdog started for UI thread monitoring
2026-04-23 09:04:17.165 12114-12138 MeshRepository          com.scmessenger.android              D  Corruption check: contacts=0, messages=0
2026-04-23 09:04:17.165 12114-12138 MeshRepository          com.scmessenger.android              D  Database integrity check passed
2026-04-23 09:04:17.166 12114-12114 MainActivity            com.scmessenger.android              D  All permissions already granted
2026-04-23 09:04:17.173 12114-12114 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 309578419; UID 10499; state: ENABLED
2026-04-23 09:04:17.173 12114-12114 DesktopModeFlags        com.scmessenger.android              D  Toggle override initialized to: OVERRIDE_UNSET
2026-04-23 09:04:17.184 12114-12138 MeshRepository          com.scmessenger.android              I  Repository background initialization completed
2026-04-23 09:04:17.184 12114-12140 MeshReposi...aintenance com.scmessenger.android              D  Storage maintenance check: free=17870MB / total=112912MB
2026-04-23 09:04:17.184 12114-12138 MainActivity            com.scmessenger.android              D  UI components initialization completed
2026-04-23 09:04:17.184 12114-12114 ContentCaptureHelper    com.scmessenger.android              I  Setting logging level to OFF
2026-04-23 09:04:17.190 12114-12114 MainActivity            com.scmessenger.android              D  MainActivity resumed
2026-04-23 09:04:17.191 12114-12114 MainActivity            com.scmessenger.android              D  All permissions already granted
2026-04-23 09:04:17.191 12114-12114 MeshRepository          com.scmessenger.android              D  Permission refresh skipped: mesh service is not running
2026-04-23 09:04:17.197 12114-12114 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 349153669; UID 10499; state: ENABLED
2026-04-23 09:04:17.301 12114-12114 MainViewModel           com.scmessenger.android              D  MainViewModel init
2026-04-23 09:04:17.302 12114-12140 MainViewMo...rageStatus com.scmessenger.android              D  Storage refreshed: 17870 MB available (Low=false)
2026-04-23 09:04:17.303 12114-12114 MainViewModel           com.scmessenger.android              D  MeshRepository service state: STOPPED
2026-04-23 09:04:17.304 12114-12140 MainViewMo...ntityState com.scmessenger.android              D  refreshIdentityState() called
2026-04-23 09:04:17.304 12114-12140 MainViewMo...ntityState com.scmessenger.android              D  Identity initialized state: true
2026-04-23 09:04:17.305 12114-12140 MainViewMo...ntityState com.scmessenger.android              D  Identity is initialized but install choice not completed, fixing preference...
2026-04-23 09:04:17.315 12114-12138 PreferencesRepository   com.scmessenger.android              I  Install choice completed: true
2026-04-23 09:04:17.315 12114-12138 MainViewMo...ntityState com.scmessenger.android              D  Identity is initialized but onboarding not completed, fixing preference...
2026-04-23 09:04:17.317 12114-12138 PreferencesRepository   com.scmessenger.android              I  Onboarding completed: true
2026-04-23 09:04:17.321 12114-12114 VRI[MainActivity]       com.scmessenger.android              D  WindowInsets changed: 1080x2400 statusBars:[0,132,0,0] navigationBars:[0,0,0,63] mandatorySystemGestures:[0,164,0,84] 
2026-04-23 09:04:17.322 12114-12114 WindowOnBackDispatcher  com.scmessenger.android              D  setTopOnBackInvokedCallback (unwrapped): android.view.ViewRootImpl$$ExternalSyntheticLambda25@c456cbf
2026-04-23 09:04:17.344 12114-12119 ssenger.android         com.scmessenger.android              I  Compiler allocated 5049KB to compile void android.view.ViewRootImpl.performTraversals()
2026-04-23 09:04:17.360 12114-12114 HWUI                    com.scmessenger.android              I  Using FreeType backend (prop=Auto)
2026-04-23 09:04:17.393 12114-12114 MeshServiceViewModel    com.scmessenger.android              D  MeshServiceViewModel initialized
2026-04-23 09:04:17.400 12114-12140 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 09:04:17.401 12114-12138 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 09:04:17.402 12114-12138 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=null, initialized=null, nickname=null
2026-04-23 09:04:17.402 12114-12140 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=null, initialized=null, nickname=null
2026-04-23 09:04:17.402 12114-12180 MeshReposi...edDeferred com.scmessenger.android              D  Lazy starting MeshService for Identity access...
2026-04-23 09:04:17.402 12114-12179 MeshReposi...edDeferred com.scmessenger.android              D  Lazy starting MeshService for Identity access...
2026-04-23 09:04:17.402 12114-12138 DashboardViewModel      com.scmessenger.android              D  Topology built: 0 nodes, 0 edges
2026-04-23 09:04:17.402 12114-12140 DashboardViewModel      com.scmessenger.android              D  Topology built: 0 nodes, 0 edges
2026-04-23 09:04:17.403 12114-12138 DashboardV...efreshData com.scmessenger.android              D  Dashboard data refreshed
2026-04-23 09:04:17.403 12114-12140 DashboardV...efreshData com.scmessenger.android              D  Dashboard data refreshed
2026-04-23 09:04:17.405 12114-12180 MeshRepository          com.scmessenger.android              I  service_start_requested
2026-04-23 09:04:17.405 12114-12180 MeshRepository          com.scmessenger.android              D  Starting MeshService...
2026-04-23 09:04:17.403 12114-12114 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:195902): avc:  denied  { search } for  name="/" dev="cgroup2" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup_v2:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:04:17.403 12114-12114 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:195903): avc:  denied  { search } for  name="/" dev="cgroup2" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup_v2:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:04:17.407 12114-12114 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:195904): avc:  denied  { search } for  name="/" dev="cgroup" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:04:17.407 12114-12114 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:195905): avc:  denied  { search } for  name="/" dev="cgroup" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:04:17.441 12114-12180 MeshRepository          com.scmessenger.android              I  SmartTransportRouter initialized for intelligent transport selection
2026-04-23 09:04:17.444 12114-12180 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:04:17.444 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:04:17.445 12114-12180 NetworkDetector         com.scmessenger.android              I  NetworkDetector monitoring started
2026-04-23 09:04:17.445 12114-12180 MeshRepository          com.scmessenger.android              I  NetworkDetector started — cellular-aware transport fallback active
2026-04-23 09:04:17.445 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:04:17.447 12114-12196 NetworkDetector         com.scmessenger.android              W  Cellular network detected (CELLULAR) — blocking ports: [9001, 9010, 4001, 5001]
2026-04-23 09:04:17.447 12114-12140 MeshReposi...hangeWatch com.scmessenger.android              I  Network type changed: WIFI → CELLULAR, resetting circuit breakers and re-bootstrapping
2026-04-23 09:04:17.450 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: CELLULAR, blocked ports: [9001, 9010, 4001, 5001]
2026-04-23 09:04:17.452 12114-12140 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:04:17.454 12114-12196 NetworkDetector         com.scmessenger.android              W  Cellular network detected (CELLULAR) — blocking ports: [9001, 9010, 4001, 5001]
2026-04-23 09:04:17.455 12114-12140 MeshRepository          com.scmessenger.android              I  Racing bootstrap: network=CELLULAR, transports=wss→tcp→quic→tcp→ws
2026-04-23 09:04:17.456 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: CELLULAR, blocked ports: [9001, 9010, 4001, 5001]
2026-04-23 09:04:17.457 12114-12140 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:04:17.468 12114-12140 MeshReposi...$Companion com.scmessenger.android              I  Using bootstrap source: StaticFallback with 4 nodes
2026-04-23 09:04:17.536 12114-12203 MainViewMo...ntityState com.scmessenger.android              D  refreshIdentityState() called
2026-04-23 09:04:17.542 12114-12203 MainViewMo...ntityState com.scmessenger.android              D  Identity initialized state: true
2026-04-23 09:04:17.544 12114-12203 MainViewMo...ntityState com.scmessenger.android              D  Identity is initialized but install choice not completed, fixing preference...
2026-04-23 09:04:17.547 12114-12203 PreferencesRepository   com.scmessenger.android              I  Install choice completed: true
2026-04-23 09:04:17.551 12114-12203 MainViewMo...ntityState com.scmessenger.android              D  Identity is initialized but onboarding not completed, fixing preference...
2026-04-23 09:04:17.554 12114-12203 PreferencesRepository   com.scmessenger.android              I  Onboarding completed: true
2026-04-23 09:04:17.588 12114-12114 MainViewModel           com.scmessenger.android              D  Preference onboardingCompleted: true
2026-04-23 09:04:17.590 12114-12114 MainViewModel           com.scmessenger.android              D  Preference installChoiceCompleted: true
2026-04-23 09:04:17.649 12114-12114 Conversati...adMessages com.scmessenger.android              D  Loaded 0 messages
2026-04-23 09:04:17.651 12114-12114 Conversati...$loadStats com.scmessenger.android              D  Loaded stats: HistoryStats(totalMessages=0, sentCount=0, receivedCount=0, undeliveredCount=0)
2026-04-23 09:04:17.653 12114-12114 Conversati...ockedPeers com.scmessenger.android              D  Loaded 0 blocked peers
2026-04-23 09:04:17.754 12114-12114 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:04:17.754 12114-12114 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:d950caca: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:04:18.976 12114-12181 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:9010 = blocked
2026-04-23 09:04:18.976 12114-12138 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:9001 = blocked
2026-04-23 09:04:18.976 12114-12139 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:443 = blocked
2026-04-23 09:04:18.976 12114-12205 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:443 = blocked
2026-04-23 09:04:19.020 12114-12138 MeshRepository          com.scmessenger.android              W  All bootstrap addresses failed or timed out, falling back to mDNS
2026-04-23 09:04:19.025 12114-12138 MeshRepository          com.scmessenger.android              I  All relay bootstrap failed, attempting mDNS local discovery
2026-04-23 09:04:19.094 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:04:20.391 12114-12114 SettingsViewModel       com.scmessenger.android              D  Loaded mesh settings: MeshSettings(relayEnabled=true, maxRelayBudget=200, batteryFloor=20, bleEnabled=true, wifiAwareEnabled=true, wifiDirectEnabled=true, internetEnabled=true, discoveryMode=NORMAL, onionRouting=false, coverTrafficEnabled=false, messagePaddingEnabled=false, timingObfuscationEnabled=false, notificationsEnabled=true, notifyDmEnabled=true, notifyDmRequestEnabled=true, notifyDmInForeground=false, notifyDmRequestInForeground=true, soundEnabled=true, badgeEnabled=true)
2026-04-23 09:04:22.182 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:04:24.033 12114-12138 MeshRepository          com.scmessenger.android              E  mDNS fallback: no LAN peers discovered within timeout
2026-04-23 09:04:24.038 12114-12138 MeshReposi...hangeWatch com.scmessenger.android              I  Network type changed: CELLULAR → WIFI, resetting circuit breakers and re-bootstrapping
2026-04-23 09:04:24.040 12114-12138 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:04:24.044 12114-12138 MeshRepository          com.scmessenger.android              I  Racing bootstrap: network=WIFI, transports=quic→tcp→wss→ws
2026-04-23 09:04:24.048 12114-12138 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:04:24.227 12114-12138 MeshReposi...$Companion com.scmessenger.android              I  Using bootstrap source: StaticFallback with 4 nodes
2026-04-23 09:04:25.264 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:04:25.735 12114-12209 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:9001 = blocked
2026-04-23 09:04:25.735 12114-12139 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:9010 = blocked
2026-04-23 09:04:25.735 12114-12201 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:443 = blocked
2026-04-23 09:04:25.736 12114-12205 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:443 = blocked
2026-04-23 09:04:25.773 12114-12209 MeshRepository          com.scmessenger.android              W  All bootstrap addresses failed or timed out, falling back to mDNS
2026-04-23 09:04:25.780 12114-12209 MeshRepository          com.scmessenger.android              I  All relay bootstrap failed, attempting mDNS local discovery
2026-04-23 09:04:28.348 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:04:28.381 12114-12120 ssenger.android         com.scmessenger.android              W  userfaultfd: MOVE ioctl seems unsupported: Connection timed out
2026-04-23 09:04:28.570 12114-12180 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:04:28.573 12114-12210 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=0))
2026-04-23 09:04:28.573 12114-12180 MeshRepository          com.scmessenger.android              I  SC_IDENTITY_OWN p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9 pk=374d73ebc21d677f889b806d6065a7177a2c5748f368036bd073e949bba5a2d4
2026-04-23 09:04:28.573 12114-12180 MeshRepository          com.scmessenger.android              I  Mesh service started successfully
2026-04-23 09:04:28.574 12114-12180 MeshReposi...edDeferred com.scmessenger.android              D  MeshService started lazily
2026-04-23 09:04:28.575 12114-12179 MeshRepository          com.scmessenger.android              I  service_start_requested
2026-04-23 09:04:28.581 12114-12179 MeshRepository          com.scmessenger.android              D  MeshService is already running
2026-04-23 09:04:28.582 12114-12179 MeshReposi...edDeferred com.scmessenger.android              D  MeshService started lazily
2026-04-23 09:04:28.593 12114-12210 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:04:28.594 12114-12210 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:04:28.594 12114-12210 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:04:28.594 12114-12209 AdvertiseSettings       com.scmessenger.android              D  setTxPowerLevel: 2
2026-04-23 09:04:28.594 12114-12209 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:04:28.596 12114-12209 BluetoothLeAdvertiser   com.scmessenger.android              D  TxPower == ADVERTISE_TX_POWER_MEDIUM
2026-04-23 09:04:28.596 12114-12209 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:04:28.597 12114-12210 BackoffStrategy         com.scmessenger.android              D  Backoff strategy reset
2026-04-23 09:04:28.598 12114-12215 BluetoothLeScanner      com.scmessenger.android              D  onScannerRegistered(status=0, scannerId=3): mScannerId=0
2026-04-23 09:04:28.601 12114-12209 BluetoothGattServer     com.scmessenger.android              D  registerCallback()
2026-04-23 09:04:28.602 12114-12209 BluetoothGattServer     com.scmessenger.android              D  registerCallback() - UUID=8348223e-1e9c-4f48-a30e-9218eb702be8
2026-04-23 09:04:28.602 12114-12210 BleScanner              com.scmessenger.android              I  BLE Scanning started (background=false, fallback=false)
2026-04-23 09:04:28.603 12114-12215 BluetoothGattServer     com.scmessenger.android              D  onServerRegistered(0)
2026-04-23 09:04:28.604 12114-12209 BluetoothGattServer     com.scmessenger.android              D  addService() - service: 0000df01-0000-1000-8000-00805f9b34fb
2026-04-23 09:04:28.605 12114-12209 BleGattServer           com.scmessenger.android              I  GATT server started with SCMessenger service
2026-04-23 09:04:28.606 12114-12215 BluetoothGattServer     com.scmessenger.android              D  onServiceAdded() - handle=147 uuid=0000df01-0000-1000-8000-00805f9b34fb status=0
2026-04-23 09:04:28.607 12114-12210 BleScanner              com.scmessenger.android              D  Duty cycle started: 10000ms scan / 30000ms interval
2026-04-23 09:04:28.610 12114-12209 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (2 bytes)
2026-04-23 09:04:28.623 12114-12209 BleAdvertiser           com.scmessenger.android              D  Identity data set: 313 bytes
2026-04-23 09:04:28.628 12114-12209 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:04:28.632 12114-12209 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:04:28.641 12114-12209 WifiTransportManager    com.scmessenger.android              D  WifiTransportManager initialized
2026-04-23 09:04:28.647 12114-12209 WifiDirectTransport     com.scmessenger.android              I  WiFi Direct started
2026-04-23 09:04:30.787 12114-12210 MeshRepository          com.scmessenger.android              E  mDNS fallback: no LAN peers discovered within timeout
2026-04-23 09:04:31.868 12114-12114 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:04:31.873 12114-12114 MeshServiceViewModel    com.scmessenger.android              D  MeshServiceViewModel initialized
2026-04-23 09:04:32.030 12114-12114 WindowOnBackDispatcher  com.scmessenger.android              W  OnBackInvokedCallback is not enabled for the application.
                                                                                                    Set 'android:enableOnBackInvokedCallback="true"' in the application manifest.
2026-04-23 09:04:32.167 12114-12170 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5001ms (threshold: 10000ms)
2026-04-23 09:04:32.196 12114-12114 MainViewModel           com.scmessenger.android              D  MeshRepository service state: RUNNING
2026-04-23 09:04:32.197 12114-12114 WifiDirect...2pReceiver com.scmessenger.android              D  WiFi P2P state changed: enabled=true
2026-04-23 09:04:32.198 12114-12210 MainViewMo...ntityState com.scmessenger.android              D  refreshIdentityState() called
2026-04-23 09:04:32.198 12114-12261 ProfileInstaller        com.scmessenger.android              D  Installing profile for com.scmessenger.android
2026-04-23 09:04:32.199 12114-12114 WifiDirect...2pReceiver com.scmessenger.android              D  Disconnected from WiFi P2P group
2026-04-23 09:04:32.201 12114-12215 HWUI                    com.scmessenger.android              I  Davey! duration=11824ms; Flags=0, FrameTimelineVsyncId=75192475, IntendedVsync=282692259982002, Vsync=282692259982002, InputEventId=814896920, HandleInputStart=282692262773731, AnimationStart=282692262784514, PerformTraversalsStart=282703919511449, DrawStart=282703919623306, FrameDeadline=282692276582002, FrameStartTime=282692262772144, FrameInterval=16696272, WorkloadTarget=16600000, SyncQueued=282704082662246, SyncStart=282704082708023, IssueDrawCommandsStart=282704082827407, SwapBuffers=282704083940933, FrameCompleted=282704084423273, DequeueBufferDuration=10539, QueueBufferDuration=179484, GpuCompleted=282704084423273, SwapBuffersCompleted=282704084162816, DisplayPresentTime=282697221340506, CommandSubmissionCompleted=282704083940933, 
2026-04-23 09:04:32.204 12114-12114 WifiDirect...2pReceiver com.scmessenger.android              D  This device changed
2026-04-23 09:04:32.205 12114-12114 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-3s427ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:04:32.205 12114-12114 WifiDirect...eDiscovery com.scmessenger.android              D  Service discovery request added
2026-04-23 09:04:32.206 12114-12114 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-3s428ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:04:32.206 12114-12114 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-3s423ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:04:32.206 12114-12114 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-3s423ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:04:32.206 12114-12114 WifiDirect...terService com.scmessenger.android              D  WiFi Direct service registered: scmessenger
2026-04-23 09:04:32.207 12114-12114 WifiTransp...tDiscovery com.scmessenger.android              I  WiFi P2P Discovery started
2026-04-23 09:04:32.209 12114-12210 MainViewMo...ntityState com.scmessenger.android              D  Identity initialized state: true
2026-04-23 09:04:33.582 12114-12210 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:04:33.593 12114-12139 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=5))
2026-04-23 09:04:33.655 12114-12139 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:04:33.659 12114-12139 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:04:38.603 12114-12139 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:04:38.611 12114-12201 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=10))
2026-04-23 09:04:42.177 12114-12170 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5002ms (threshold: 10000ms)
2026-04-23 09:04:42.366 12114-12209 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:04:42.367 12114-12209 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:04:42.369 12114-12209 MeshRepository          com.scmessenger.android              I  ✓ Internet transport (Swarm) initiated and bridge wired
2026-04-23 09:04:42.370 12114-12209 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:04:42.374 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:04:42.375 12114-12209 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:04:42.377 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:04:42.380 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:04:42.381 12114-12209 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:04:42.382 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:04:42.385 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:04:42.386 12114-12209 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:04:42.387 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:04:42.390 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:04:42.391 12114-12209 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:04:42.393 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:04:42.395 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:04:42.396 12114-12209 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:04:42.398 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:04:42.400 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:04:42.402 12114-12209 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:04:42.403 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:04:42.404 12114-12209 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=1), next attempt in 10000ms
2026-04-23 09:04:43.615 12114-12179 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:04:43.619 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=15))
2026-04-23 09:04:45.286 12114-12114 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:04:45.421 12114-12114 Choreographer           com.scmessenger.android              I  Skipped 806 frames!  The application may be doing too much work on its main thread.
2026-04-23 09:04:45.454 12114-12114 ContactsVi...adContacts com.scmessenger.android              D  Loaded 0 contacts, filtered nearby peers to 0
2026-04-23 09:04:45.541 12114-12114 BleAdverti...seCallback com.scmessenger.android              I  BLE Advertising started successfully
2026-04-23 09:04:45.542 12114-12114 WifiDirect...rDiscovery com.scmessenger.android              D  Peer discovery started
2026-04-23 09:04:45.543 12114-12114 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:04:45.545 12114-12114 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:04:45.552 12114-12132 HWUI                    com.scmessenger.android              I  Davey! duration=13562ms; Flags=0, FrameTimelineVsyncId=75195530, IntendedVsync=282703867776354, Vsync=282717301109956, InputEventId=1749416676, HandleInputStart=282717309542542, AnimationStart=282717309543967, PerformTraversalsStart=282717370883159, DrawStart=282717370960064, FrameDeadline=282704130905517, FrameStartTime=282717309358664, FrameInterval=16666667, WorkloadTarget=16600000, SyncQueued=282717428997214, SyncStart=282717429166485, IssueDrawCommandsStart=282717429290671, SwapBuffers=282717430249085, FrameCompleted=282717430562359, DequeueBufferDuration=8870, QueueBufferDuration=137655, GpuCompleted=282717430562359, SwapBuffersCompleted=282717430419211, DisplayPresentTime=282697254900972, CommandSubmissionCompleted=282717430249085, 
2026-04-23 09:04:45.640 12114-12114 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:04:45.640 12114-12114 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:c5ec9d61: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:04:46.844 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:04:47.450 12114-12205 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:04:47.474 12114-12205 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:04:48.610 12114-12114 BleScanner...d$Runnable com.scmessenger.android              W  BLE scan fallback enabled after 20014 ms without mesh advertisements; switching to unfiltered scan
2026-04-23 09:04:48.624 12114-12114 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:04:48.624 12114-12114 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:04:48.625 12114-12114 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:04:48.628 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:04:48.632 12114-12114 BleScanner              com.scmessenger.android              I  BLE scan restarted (background=false, fallback=true)
2026-04-23 09:04:48.634 12114-12132 BluetoothLeScanner      com.scmessenger.android              D  onScannerRegistered(status=0, scannerId=3): mScannerId=0
2026-04-23 09:04:48.640 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=20))
2026-04-23 09:04:49.940 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:04:53.645 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:04:53.654 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=25))
2026-04-23 09:04:58.420 12114-12205 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:04:58.438 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:04:58.443 12114-12205 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:04:58.450 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:04:58.462 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:04:58.466 12114-12205 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:04:58.471 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:04:58.480 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:04:58.483 12114-12205 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:04:58.487 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:04:58.493 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:04:58.495 12114-12205 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:04:58.498 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:04:58.503 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:04:58.505 12114-12205 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:04:58.507 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:04:58.513 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:04:58.515 12114-12205 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:04:58.517 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:04:58.519 12114-12205 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=2), next attempt in 30000ms
2026-04-23 09:04:58.661 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:04:58.664 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=30))
2026-04-23 09:05:03.674 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:05:03.681 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=35))
2026-04-23 09:05:08.553 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:05:08.688 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:05:08.695 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=40))
2026-04-23 09:05:13.702 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:05:13.712 12114-12180 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=45))
2026-04-23 09:05:14.702 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:05:15.547 12114-12114 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:05:15.556 12114-12114 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:05:18.726 12114-12180 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:05:18.736 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=50))
2026-04-23 09:05:23.751 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:05:23.765 12114-12180 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=55))
2026-04-23 09:05:24.051 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:05:27.185 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:05:28.782 12114-12180 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:05:28.805 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=60))
2026-04-23 09:05:30.549 12114-12205 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:05:30.560 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:05:30.564 12114-12205 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:05:30.567 12114-12205 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:05:30.573 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:05:30.582 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:05:30.585 12114-12205 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:05:30.589 12114-12205 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:05:30.594 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:05:30.603 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:05:30.607 12114-12205 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:05:30.610 12114-12205 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:05:30.614 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:05:30.626 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:05:30.629 12114-12205 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:05:30.631 12114-12205 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:05:30.633 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:05:30.637 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:05:30.639 12114-12205 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:05:30.640 12114-12205 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:05:30.643 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:05:30.646 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:05:30.647 12114-12205 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:05:30.649 12114-12205 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:05:30.650 12114-12205 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:05:30.652 12114-12205 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=3), next attempt in 30000ms
2026-04-23 09:05:33.812 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:05:33.818 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=65))
2026-04-23 09:05:38.826 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:05:38.831 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=70))
2026-04-23 09:05:43.840 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:05:43.847 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=75))
2026-04-23 09:05:45.572 12114-12114 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:05:45.572 12114-12114 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:05:45.575 12114-12114 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:05:48.856 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:05:48.863 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=80))
2026-04-23 09:05:53.873 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:05:53.881 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=85))
2026-04-23 09:05:55.225 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:05:58.333 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:05:58.892 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:05:58.903 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=90))
2026-04-23 09:06:01.459 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:06:02.678 12114-12205 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:06:02.688 12114-12205 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:02.696 12114-12205 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:02.705 12114-12205 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:02.713 12114-12205 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:02.721 12114-12205 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:06:02.726 12114-12205 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:06:02.732 12114-12205 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:06:02.737 12114-12205 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:06:02.742 12114-12205 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:02.746 12114-12205 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:02.751 12114-12205 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:02.754 12114-12205 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:02.758 12114-12205 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=4), next attempt in 16000ms
2026-04-23 09:06:03.910 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:06:03.916 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=95))
2026-04-23 09:06:04.560 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:06:08.926 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:06:08.933 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=100))
2026-04-23 09:06:13.945 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:06:13.956 12114-12180 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=105))
2026-04-23 09:06:15.605 12114-12114 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:06:15.605 12114-12114 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:06:15.608 12114-12114 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:06:18.780 12114-12180 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:06:18.809 12114-12180 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:18.828 12114-12180 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:18.843 12114-12180 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:18.852 12114-12180 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:18.860 12114-12180 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:06:18.867 12114-12180 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:06:18.872 12114-12180 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:06:18.877 12114-12180 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:06:18.882 12114-12180 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:18.886 12114-12180 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:18.891 12114-12180 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:18.894 12114-12180 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:18.898 12114-12180 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=5), next attempt in 32000ms
2026-04-23 09:06:18.958 12114-12180 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:06:18.964 12114-12180 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=110))
2026-04-23 09:06:23.972 12114-12180 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:06:23.979 12114-12180 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=115))
2026-04-23 09:06:28.990 12114-12180 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:06:29.001 12114-12180 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=120))
2026-04-23 09:06:29.411 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:06:32.516 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:06:34.012 12114-12180 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:06:34.022 12114-12180 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=126))
2026-04-23 09:06:39.036 12114-12180 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:06:39.049 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=131))
2026-04-23 09:06:44.072 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:06:44.091 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=136))
2026-04-23 09:06:45.639 12114-12114 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:06:45.639 12114-12114 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:06:45.640 12114-12114 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:06:49.112 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:06:49.137 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=141))
2026-04-23 09:06:50.937 12114-12209 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:06:50.958 12114-12209 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:50.970 12114-12209 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:50.981 12114-12209 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:50.990 12114-12209 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:50.998 12114-12209 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:06:51.005 12114-12209 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:06:51.011 12114-12209 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:06:51.017 12114-12209 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:06:51.021 12114-12209 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:51.026 12114-12209 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:51.031 12114-12209 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:51.034 12114-12209 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:06:51.038 12114-12209 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=6), next attempt in 60000ms
2026-04-23 09:06:54.147 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:06:54.154 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=146))
2026-04-23 09:06:59.159 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:06:59.164 12114-12180 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=151))
2026-04-23 09:07:03.632 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:07:04.171 12114-12180 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:07:04.180 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=156))
2026-04-23 09:07:07.235 12114-12170 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5001ms (threshold: 10000ms)
2026-04-23 09:07:08.263 12114-12114 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:07:08.270 12114-12114 Choreographer           com.scmessenger.android              I  Skipped 612 frames!  The application may be doing too much work on its main thread.
2026-04-23 09:07:08.312 12114-12215 HWUI                    com.scmessenger.android              I  Davey! duration=10260ms; Flags=0, FrameTimelineVsyncId=75199211, IntendedVsync=282849926598520, Vsync=282860148448348, InputEventId=0, HandleInputStart=282860159183074, AnimationStart=282860159184376, PerformTraversalsStart=282860180570566, DrawStart=282860180615570, FrameDeadline=282849943198520, FrameStartTime=282860158760630, FrameInterval=16700831, WorkloadTarget=16600000, SyncQueued=282860183890675, SyncStart=282860184089487, IssueDrawCommandsStart=282860184364674, SwapBuffers=282860186501352, FrameCompleted=282860187070159, DequeueBufferDuration=18433, QueueBufferDuration=409628, GpuCompleted=282860186938486, SwapBuffersCompleted=282860187070159, DisplayPresentTime=282854787376472, CommandSubmissionCompleted=282860186501352, 
2026-04-23 09:07:08.335 12114-12114 WifiDirect...2pReceiver com.scmessenger.android              D  This device changed
2026-04-23 09:07:09.190 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:07:09.213 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=161))
2026-04-23 09:07:09.833 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:07:14.223 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:07:14.233 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=166))
2026-04-23 09:07:15.650 12114-12114 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:07:15.650 12114-12114 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:07:15.650 12114-12114 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:07:15.986 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:07:19.078 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:07:19.236 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:07:19.242 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=171))
2026-04-23 09:07:22.181 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:07:24.253 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:07:24.265 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=176))
2026-04-23 09:07:25.263 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:07:27.251 12114-12170 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5001ms (threshold: 10000ms)
2026-04-23 09:07:28.857 12114-12116 ssenger.android         com.scmessenger.android              I  Thread[2,tid=12116,WaitingInMainSignalCatcherLoop,Thread*=0xb4000076a7917f50,peer=0x20039a8,"Signal Catcher"]: reacting to signal 3
2026-04-23 09:07:28.857 12114-12116 ssenger.android         com.scmessenger.android              I  
2026-04-23 09:07:29.072 12114-12116 ssenger.android         com.scmessenger.android              I  Wrote stack traces to tombstoned
2026-04-23 09:07:29.265 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:07:29.266 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=181))
2026-04-23 09:07:31.411 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:07:31.845 12114-12114 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:07:31.846 12114-12114 IdentityVi...adIdentity com.scmessenger.android              D  Identity loaded: caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747
2026-04-23 09:07:32.264 12114-12170 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5001ms (threshold: 10000ms)
2026-04-23 09:07:34.277 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:07:34.284 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=186))
2026-04-23 09:07:34.513 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:07:37.276 12114-12170 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5002ms (threshold: 10000ms)
2026-04-23 09:07:39.291 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:07:39.302 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=191))
2026-04-23 09:07:41.837  1619-12558 ActivityManager         system_server                        E  ANR in com.scmessenger.android (com.scmessenger.android/.ui.MainActivity)
                                                                                                    PID: 12114
                                                                                                    Reason: Input dispatching timed out (1e0d494 com.scmessenger.android/com.scmessenger.android.ui.MainActivity is not responding. Waited 5001ms for MotionEvent).
                                                                                                    Parent: com.scmessenger.android/.ui.MainActivity
                                                                                                    ErrorId: d3dd7f83-bec6-469d-8139-4915e0c4c2a2
                                                                                                    Frozen: false
                                                                                                    Load: 1.49 / 2.02 / 2.58
                                                                                                    ----- Output from /proc/pressure/memory -----
                                                                                                    some avg10=0.00 avg60=0.00 avg300=0.08 total=1517039558
                                                                                                    full avg10=0.00 avg60=0.00 avg300=0.05 total=877553244
                                                                                                    ----- End output from /proc/pressure/memory -----
                                                                                                    ----- Output from /proc/pressure/cpu -----
                                                                                                    some avg10=2.47 avg60=3.56 avg300=4.76 total=26873533131
                                                                                                    full avg10=0.00 avg60=0.00 avg300=0.00 total=0
                                                                                                    ----- End output from /proc/pressure/cpu -----
                                                                                                    ----- Output from /proc/pressure/io -----
                                                                                                    some avg10=0.00 avg60=0.01 avg300=0.64 total=4761120387
                                                                                                    full avg10=0.00 avg60=0.00 avg300=0.37 total=2477738390
                                                                                                    ----- End output from /proc/pressure/io -----
                                                                                                    
                                                                                                    CPU usage from 1ms to 12943ms later (2026-04-23 09:07:28.836 to 2026-04-23 09:07:41.778):
                                                                                                      101% 12114/com.scmessenger.android: 100% user + 0.6% kernel / faults: 5911 minor 7 major
                                                                                                      17% 1619/system_server: 7.3% user + 9.8% kernel / faults: 7086 minor 18 major
                                                                                                      2.3% 2261/com.android.systemui: 1.3% user + 0.9% kernel / faults: 2274 minor 18 major
                                                                                                      2.2% 8374/com.google.android.gms.persistent: 1.3% user + 0.8% kernel / faults: 1845 minor 63 major
                                                                                                      1.6% 11332/com.lemon.lvoverseas: 0.5% user + 1.1% kernel / faults: 798 minor 4 major
                                                                                                      1.6% 21975/com.google.android.inputmethod.latin: 1.1% user + 0.5% kernel / faults: 1886 minor 6 major
                                                                                                      1.6% 92/kswapd0: 0% user + 1.6% kernel
                                                                                                      1.2% 2476/com.android.hbmsvmanager: 1% user + 0.2% kernel / faults: 1265 minor
                                                                                                      1.1% 148/sched_pmu_wq: 0% user + 1.1% kernel
                                                                                                      1% 136/perf_mon_update_client_task: 0% user + 1% kernel
                                                                                                    18% TOTAL: 15% user + 3% kernel + 0% iowait + 0.3% irq + 0.2% softirq
                                                                                                    CPU usage from 38ms to 583ms later (2026-04-23 09:07:28.873 to 2026-04-23 09:07:29.418) with 99% awake:
                                                                                                      126% 1619/system_server: 58% user + 68% kernel / faults: 3812 minor 14 major
                                                                                                        55% 1630/Signal Catcher: 38% user + 17% kernel
                                                                                                        53% 12560/AnrAuxiliaryTas: 17% user + 36% kernel
                                                                                                        10% 12558/AnrConsumer: 0% user + 10% kernel
                                                                                                        2.1% 2218/BackgroundInsta: 0% user + 2.1% kernel
                                                                                                        2.1% 2238/HealthServiceBi: 0% user + 2.1% kernel
                                                                                                      103% 12114/com.scmessenger.android: 103% user + 0% kernel / faults: 18 minor
                                                                                                        100% 12114/ssenger.android: 100% user + 0% kernel
                                                                                                        3.4% 12180/DefaultDispatch: 3.4% user + 0% kernel
                                                                                                      5.5% 92/kswapd0: 0% user + 5.5% kernel
                                                                                                      3.6% 52/rcuop/4: 0% user + 3.6% kernel
                                                                                                      1.8% 51/rcuog/4: 0% user + 1.8% kernel
                                                                                                      1.8% 136/perf_mon_update_client_task: 0% user + 1.8% kernel
                                                                                                      1.8% 157/eh_comp_thread: 0% user + 1.8% kernel
                                                                                                      1.8% 218/thermal_LITTLE: 0% user + 1.8% kernel
                                                                                                      1.9% 487/sugov:0: 0% user + 1.9% kernel
                                                                                                      3.3% 11332/com.lemon.lvoverseas: 0% user + 3.3% kernel
                                                                                                    33% TOTAL: 20% user + 12% kernel + 0.2% iowait + 0.4% irq + 0.2% softirq
2026-04-23 09:07:42.279 12114-12114 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:07:42.574 12114-12215 HWUI                    com.scmessenger.android              I  Davey! duration=21107ms; Flags=0, FrameTimelineVsyncId=75206148, IntendedVsync=282873346840896, Vsync=282873346840896, InputEventId=83111966, HandleInputStart=282873354202856, AnimationStart=282873354243221, PerformTraversalsStart=282883746505229, DrawStart=282883746542664, FrameDeadline=282873363440896, FrameStartTime=282873354196305, FrameInterval=16713805, WorkloadTarget=16600000, SyncQueued=282894447277614, SyncStart=282894448493801, IssueDrawCommandsStart=282894449785386, SwapBuffers=282894454425401, FrameCompleted=282894455246405, DequeueBufferDuration=22420, QueueBufferDuration=512248, GpuCompleted=282894455246405, SwapBuffersCompleted=282894455081203, DisplayPresentTime=282870456363337, CommandSubmissionCompleted=282894454425401, 
2026-04-23 09:07:42.598 12114-12114 Choreographer           com.scmessenger.android              I  Skipped 644 frames!  The application may be doing too much work on its main thread.
2026-04-23 09:07:44.311 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:07:44.319 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=196))
2026-04-23 09:07:46.871 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:07:47.285 12114-12170 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5001ms (threshold: 10000ms)
2026-04-23 09:07:47.675 12114-12116 ssenger.android         com.scmessenger.android              I  Thread[2,tid=12116,WaitingInMainSignalCatcherLoop,Thread*=0xb4000076a7917f50,peer=0x20039a8,"Signal Catcher"]: reacting to signal 3
2026-04-23 09:07:47.675 12114-12116 ssenger.android         com.scmessenger.android              I  
2026-04-23 09:07:47.873 12114-12116 ssenger.android         com.scmessenger.android              I  Wrote stack traces to tombstoned
2026-04-23 09:07:49.322 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:07:49.328 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=201))
2026-04-23 09:07:49.955 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:07:50.964  1619-12616 ActivityManager         system_server                        E  ANR in com.scmessenger.android (com.scmessenger.android/.ui.MainActivity)
                                                                                                    PID: 12114
                                                                                                    Reason: Input dispatching timed out (1e0d494 com.scmessenger.android/com.scmessenger.android.ui.MainActivity is not responding. Waited 5001ms for FocusEvent(hasFocus=true)).
                                                                                                    Parent: com.scmessenger.android/.ui.MainActivity
                                                                                                    ErrorId: f11eb815-e3c6-47c5-a7e7-d30ce51ee212
                                                                                                    Frozen: false
                                                                                                    Load: 1.54 / 2.0 / 2.57
                                                                                                    ----- Output from /proc/pressure/memory -----
                                                                                                    some avg10=0.07 avg60=0.04 avg300=0.09 total=1517100270
                                                                                                    full avg10=0.07 avg60=0.04 avg300=0.06 total=877606526
                                                                                                    ----- End output from /proc/pressure/memory -----
                                                                                                    ----- Output from /proc/pressure/cpu -----
                                                                                                    some avg10=1.29 avg60=2.88 avg300=4.52 total=26873855586
                                                                                                    full avg10=0.00 avg60=0.00 avg300=0.00 total=0
                                                                                                    ----- End output from /proc/pressure/cpu -----
                                                                                                    ----- Output from /proc/pressure/io -----
                                                                                                    some avg10=0.00 avg60=0.00 avg300=0.59 total=4761140911
                                                                                                    full avg10=0.00 avg60=0.00 avg300=0.34 total=2477756752
                                                                                                    ----- End output from /proc/pressure/io -----
                                                                                                    
                                                                                                    CPU usage from 5867ms to -1ms ago (2026-04-23 09:07:41.778 to 2026-04-23 09:07:47.645):
                                                                                                      101% 12114/com.scmessenger.android: 100% user + 0.8% kernel / faults: 2854 minor 47 major
                                                                                                      12% 1619/system_server: 7.1% user + 5.4% kernel / faults: 1860 minor 30 major
                                                                                                      6.8% 546/surfaceflinger: 4% user + 2.7% kernel / faults: 58 minor
                                                                                                      1% 365/ueventd: 0.9% user + 0% kernel
                                                                                                      0.9% 1223/android.hardware.usb-service.gs101: 0.8% user + 0.1% kernel
                                                                                                      2.9% 2261/com.android.systemui: 2% user + 0.8% kernel / faults: 377 minor 7 major
                                                                                                      2.3% 19870/com.life360.android.safetymapd: 1.1% user + 1.1% kernel / faults: 270 minor 20 major
                                                                                                      2.2% 548/android.hardware.graphics.composer@2.4-service: 0.6% user + 1.5% kernel / faults: 1 major
                                                                                                      2% 1484/adbd: 0.6% user + 1.3% kernel
                                                                                                      1.7% 11332/com.lemon.lvoverseas: 1.1% user + 0.5% kernel / faults: 373 minor 18 major
                                                                                                    19% TOTAL: 15% user + 2.8% kernel + 0.1% iowait + 0.6% irq + 0.3% softirq
                                                                                                    CPU usage from 39ms to 580ms later (2026-04-23 09:07:47.684 to 2026-04-23 09:07:48.224):
                                                                                                      144% 1619/system_server: 54% user + 89% kernel / faults: 3779 minor 27 major
                                                                                                        54% 1630/Signal Catcher: 34% user + 19% kernel
                                                                                                        54% 12617/AnrAuxiliaryTas: 10% user + 43% kernel
                                                                                                        13% 12616/AnrConsumer: 2.1% user + 10% kernel
                                                                                                        4.3% 1980/binder:1619_4: 2.1% user + 2.1% kernel
                                                                                                        2.1% 1676/android.ui: 2.1% user + 0% kernel
                                                                                                        2.1% 1689/ActivityManager: 0% user + 2.1% kernel
                                                                                                        2.1% 2054/SensorService: 0% user + 2.1% kernel
                                                                                                        2.1% 2065/AccountManagerS: 2.1% user + 0% kernel
                                                                                                        2.1% 2085/IntrusionDetect: 0% user + 2.1% kernel
                                                                                                        2.1% 2601/binder:1619_A: 2.1% user + 0% kernel
                                                                                                      99% 12114/com.scmessenger.android: 99% user + 0% kernel
                                                                                                        99% 12114/ssenger.android: 99% user + 0% kernel
                                                                                                      44% 12113/com.life360.android.safetymapd:service: 40% user + 3.3% kernel / faults: 354 minor 2 major
                                                                                                        40% 12127/HeapTaskDaemon: 37% user + 3.3% kernel
                                                                                                        3.3% 12323/GpiDataControll: 3.3% user + 0% kernel
                                                                                                      15% 92/kswapd0: 0% user + 15% kernel
                                                                                                      8.5% 8374/com.google.android.gms.persistent: 8.5% user + 0% kernel / faults: 111 minor 6 major
                                                                                                        2.8% 8374/.gms.persistent: 2.8% user + 0% kernel
                                                                                                        2.8% 8554/GoogleLocationS: 2.8% user + 0% kernel
                                                                                                       +0% 12618/-Executor] idle: 0% user + 0% kernel
                                                                                                       +0% 12619/-Executor] idle: 0% user + 0% kernel
                                                                                                      1.8% 52/rcuop/4: 0% user + 1.8% kernel
                                                                                                      1.8% 136/perf_mon_update_client_task: 0% user + 1.8% kernel
                                                                                                      1.8% 148/sched_pmu_wq: 0% user + 1.8% kernel
                                                                                                      1.9% 482/lmkd: 1.9% user + 0% kernel
                                                                                                      1.9% 534/android.system.suspend-service: 1.9% user + 0% kernel
                                                                                                      2% 1238/android.hardware.nfc-service.st: 0% user + 2% kernel
                                                                                                      2.2% 2261/com.android.systemui: 2.2% user + 0% kernel / faults: 135 minor
                                                                                                      2.9% 9178/logcat: 0% user + 2.9% kernel
                                                                                                    40% TOTAL: 25% user + 14% kernel + 0.2% iowait + 0.6% irq + 0.2% softirq
2026-04-23 09:07:52.289 12114-12170 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5001ms (threshold: 10000ms)
2026-04-23 09:07:53.028 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:07:53.419 12114-12114 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:07:53.433 12114-12114 Choreographer           com.scmessenger.android              I  Skipped 649 frames!  The application may be doing too much work on its main thread.
2026-04-23 09:07:53.448 12114-12134 HWUI                    com.scmessenger.android              I  Davey! duration=21569ms; Flags=0, FrameTimelineVsyncId=75206466, IntendedVsync=282883752068349, Vsync=282894485401897, InputEventId=37240734, HandleInputStart=282894486679836, AnimationStart=282894486701687, PerformTraversalsStart=282905316370922, DrawStart=282905316406973, FrameDeadline=282894472976611, FrameStartTime=282894486516262, FrameInterval=16666667, WorkloadTarget=16600000, SyncQueued=282905318842968, SyncStart=282905319108186, IssueDrawCommandsStart=282905319224519, SwapBuffers=282905321056387, FrameCompleted=282905322322053, DequeueBufferDuration=10579, QueueBufferDuration=217367, GpuCompleted=282905322322053, SwapBuffersCompleted=282905321335766, DisplayPresentTime=282870489850601, CommandSubmissionCompleted=282905321056387, 
2026-04-23 09:07:54.338 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:07:54.349 12114-12180 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=206))
2026-04-23 09:07:55.078 12114-12180 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:07:55.089 12114-12180 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:07:55.093 12114-12180 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:07:55.096 12114-12180 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:07:55.100 12114-12180 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:07:55.103 12114-12180 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:07:55.106 12114-12180 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:07:55.109 12114-12180 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:07:55.114 12114-12180 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:07:55.118 12114-12180 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:07:55.121 12114-12180 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:07:55.123 12114-12180 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:07:55.126 12114-12180 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:07:55.129 12114-12180 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=7), next attempt in 60000ms
2026-04-23 09:07:56.108 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:07:57.298 12114-12170 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5001ms (threshold: 10000ms)
2026-04-23 09:07:58.691 12114-12116 ssenger.android         com.scmessenger.android              I  Thread[2,tid=12116,WaitingInMainSignalCatcherLoop,Thread*=0xb4000076a7917f50,peer=0x20039a8,"Signal Catcher"]: reacting to signal 3
2026-04-23 09:07:58.691 12114-12116 ssenger.android         com.scmessenger.android              I  
2026-04-23 09:07:58.883 12114-12116 ssenger.android         com.scmessenger.android              I  Wrote stack traces to tombstoned
2026-04-23 09:07:59.350 12114-12180 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:07:59.352 12114-12180 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=211))
2026-04-23 09:08:01.384  1619-12655 ActivityManager         system_server                        E  ANR in com.scmessenger.android (com.scmessenger.android/.ui.MainActivity)
                                                                                                    PID: 12114
                                                                                                    Reason: Input dispatching timed out (1e0d494 com.scmessenger.android/com.scmessenger.android.ui.MainActivity is not responding. Waited 5005ms for MotionEvent).
                                                                                                    Parent: com.scmessenger.android/.ui.MainActivity
                                                                                                    ErrorId: 1bc6a4a2-f5a5-4709-8c8b-8928785c062a
                                                                                                    Frozen: false
                                                                                                    Load: 1.53 / 1.98 / 2.56
                                                                                                    ----- Output from /proc/pressure/memory -----
                                                                                                    some avg10=0.18 avg60=0.09 avg300=0.10 total=1517168981
                                                                                                    full avg10=0.18 avg60=0.09 avg300=0.07 total=877671910
                                                                                                    ----- End output from /proc/pressure/memory -----
                                                                                                    ----- Output from /proc/pressure/cpu -----
                                                                                                    some avg10=1.69 avg60=2.80 avg300=4.45 total=26874137608
                                                                                                    full avg10=0.00 avg60=0.00 avg300=0.00 total=0
                                                                                                    ----- End output from /proc/pressure/cpu -----
                                                                                                    ----- Output from /proc/pressure/io -----
                                                                                                    some avg10=0.00 avg60=0.00 avg300=0.57 total=4761154826
                                                                                                    full avg10=0.00 avg60=0.00 avg300=0.33 total=2477769022
                                                                                                    ----- End output from /proc/pressure/io -----
                                                                                                    
                                                                                                    CPU usage from 11025ms to -1ms ago (2026-04-23 09:07:47.645 to 2026-04-23 09:07:58.672) with 99% awake:
                                                                                                      102% 12114/com.scmessenger.android: 101% user + 0.8% kernel / faults: 7194 minor 14 major
                                                                                                      26% 1619/system_server: 11% user + 14% kernel / faults: 10775 minor 44 major
                                                                                                      4.1% 2261/com.android.systemui: 3.1% user + 0.9% kernel / faults: 2189 minor 10 major
                                                                                                      4% 546/surfaceflinger: 1.8% user + 2.2% kernel / faults: 95 minor
                                                                                                      4% 8374/com.google.android.gms.persistent: 2.9% user + 1.1% kernel / faults: 3181 minor 38 major
                                                                                                      1.8% 12113/com.life360.android.safetymapd:service: 1.4% user + 0.4% kernel / faults: 2020 minor 305 major
                                                                                                      1.5% 92/kswapd0: 0% user + 1.5% kernel
                                                                                                      1.4% 21975/com.google.android.inputmethod.latin: 0.9% user + 0.4% kernel / faults: 2061 minor 1 major
                                                                                                      2% 365/ueventd: 1.9% user + 0.1% kernel
                                                                                                      1.7% 1484/adbd: 0.7% user + 0.9% kernel / faults: 364 minor
                                                                                                    22% TOTAL: 17% user + 4.4% kernel + 0% iowait + 0.5% irq + 0.2% softirq
                                                                                                    CPU usage from 46ms to 589ms later (2026-04-23 09:07:58.717 to 2026-04-23 09:07:59.260):
                                                                                                      134% 1619/system_server: 58% user + 75% kernel / faults: 3921 minor 27 major
                                                                                                        67% 1630/Signal Catcher: 43% user + 23% kernel
                                                                                                        54% 12656/AnrAuxiliaryTas: 10% user + 43% kernel
                                                                                                        6.5% 12655/AnrConsumer: 0% user + 6.5% kernel
                                                                                                        2.1% 1696/OomAdjuster: 0% user + 2.1% kernel
                                                                                                        2.1% 2228/AdServicesManag: 0% user + 2.1% kernel
                                                                                                        2.1% 3984/backup-0: 2.1% user + 0% kernel
                                                                                                        2.1% 12654/AnrMainProcessD: 0% user + 2.1% kernel
                                                                                                      97% 12114/com.scmessenger.android: 97% user + 0% kernel
                                                                                                        100% 12114/ssenger.android: 100% user + 0% kernel
                                                                                                      1.8% 56/ksoftirqd/5: 0% user + 1.8% kernel
                                                                                                      1.8% 59/rcuop/5: 0% user + 1.8% kernel
                                                                                                      1.8% 136/perf_mon_update_client_task: 0% user + 1.8% kernel
                                                                                                      1.8% 218/thermal_LITTLE: 0% user + 1.8% kernel
                                                                                                      1.9% 487/sugov:0: 0% user + 1.9% kernel
                                                                                                      2.2% 1905/android.hardware.thermal-service.pixel: 0% user + 2.2% kernel
                                                                                                        2.2% 1911/FileWatcherThre: 0% user + 2.2% kernel
                                                                                                    33% TOTAL: 21% user + 11% kernel + 0.2% iowait + 0.4% irq + 0.2% softirq
2026-04-23 09:08:02.309 12114-12170 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5001ms (threshold: 10000ms)
2026-04-23 09:08:04.177 12114-12114 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:08:04.204 12114-12114 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:08:04.204 12114-12114 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:08:04.204 12114-12114 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:08:04.217 12114-12114 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:08:04.217 12114-12114 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:4a08a825: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:08:04.217 12114-12114 Choreographer           com.scmessenger.android              I  Skipped 646 frames!  The application may be doing too much work on its main thread.
2026-04-23 09:08:04.245 12114-12215 HWUI                    com.scmessenger.android              I  Davey! duration=21608ms; Flags=0, FrameTimelineVsyncId=75206957, IntendedVsync=282894494268668, Vsync=282905310935551, InputEventId=0, HandleInputStart=282905321577221, AnimationStart=282905321578116, PerformTraversalsStart=282916086564449, DrawStart=282916086605017, FrameDeadline=282905352002114, FrameStartTime=282905321413850, FrameInterval=16666667, WorkloadTarget=16600000, SyncQueued=282916088881181, SyncStart=282916089056759, IssueDrawCommandsStart=282916089201249, SwapBuffers=282916102064083, FrameCompleted=282916102479814, DequeueBufferDuration=9725, QueueBufferDuration=131795, GpuCompleted=282916102479814, SwapBuffersCompleted=282916102234453, DisplayPresentTime=282870523189956, CommandSubmissionCompleted=282916102064083, 
2026-04-23 09:08:04.358 12114-12180 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:08:04.366 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=216))
2026-04-23 09:08:05.365 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:08:07.321 12114-12170 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5001ms (threshold: 10000ms)
2026-04-23 09:08:08.465 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:08:09.375 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:08:09.384 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=221))
2026-04-23 09:08:12.332 12114-12170 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5001ms (threshold: 10000ms)
2026-04-23 09:08:14.393 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:08:14.401 12114-12180 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=226))
2026-04-23 09:08:15.006 12114-12114 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:08:15.015 12114-12114 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:08:15.015 12114-12114 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:148339c2: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:08:15.035 12114-12215 HWUI                    com.scmessenger.android              I  Davey! duration=21568ms; Flags=0, FrameTimelineVsyncId=75209754, IntendedVsync=282905334423413, Vsync=282916101090295, InputEventId=838922117, HandleInputStart=282916106290768, AnimationStart=282916106292029, PerformTraversalsStart=282926900667726, DrawStart=282926900720298, FrameDeadline=282916127535767, FrameStartTime=282916106139075, FrameInterval=16666667, WorkloadTarget=16600000, SyncQueued=282926900844321, SyncStart=282926900902101, IssueDrawCommandsStart=282926900986614, SwapBuffers=282926902599936, FrameCompleted=282926903144858, DequeueBufferDuration=13021, QueueBufferDuration=124268, GpuCompleted=282926903144858, SwapBuffersCompleted=282926902760296, DisplayPresentTime=282870539966812, CommandSubmissionCompleted=282926902599936, 
2026-04-23 09:08:16.991 12114-12114 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:08:16.992 12114-12114 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:982df92b: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:08:18.060 12114-12114 MainActivity            com.scmessenger.android              D  MainActivity paused
2026-04-23 09:08:18.063 12114-12114 VRI[MainActivity]       com.scmessenger.android              D  visibilityChanged oldVisibility=true newVisibility=false
2026-04-23 09:08:19.416 12114-12180 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:08:19.430 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=231))
2026-04-23 09:08:23.909 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:08:24.433 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:08:24.861 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=236))
2026-04-23 09:08:27.226 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:08:29.867 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:08:29.875 12114-12180 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=241))
2026-04-23 09:08:30.075 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:08:33.160 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:08:34.227 12114-12114 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:08:34.227 12114-12114 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:08:34.229 12114-12114 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:08:34.890 12114-12180 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:08:34.905 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=246))
2026-04-23 09:08:36.222 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:08:39.301 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:08:39.913 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:08:39.927 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=251))
2026-04-23 09:08:44.944 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:08:44.970 12114-12180 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=256))
2026-04-23 09:08:45.482 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:08:48.571 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:08:49.975 12114-12180 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:08:49.992 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=261))
2026-04-23 09:08:55.013 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:08:55.044 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=267))
2026-04-23 09:08:59.258 12114-12209 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:08:59.268 12114-12209 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:08:59.296 12114-12209 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:08:59.307 12114-12209 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:08:59.315 12114-12209 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:08:59.321 12114-12209 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:08:59.330 12114-12209 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:08:59.343 12114-12209 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:08:59.349 12114-12209 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:08:59.361 12114-12209 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:08:59.366 12114-12209 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:08:59.380 12114-12209 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:08:59.393 12114-12209 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:08:59.401 12114-12209 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=8), next attempt in 60000ms
2026-04-23 09:09:00.045 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:09:00.059 12114-12180 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=272))
2026-04-23 09:09:04.240 12114-12114 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:09:04.240 12114-12114 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:09:04.243 12114-12114 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:09:05.064 12114-12180 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:09:05.077 12114-12180 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=277))
2026-04-23 09:09:10.597 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:09:10.789 12114-12180 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:09:11.439 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=282))
2026-04-23 09:09:13.418 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:09:16.519 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:09:16.528 12114-12205 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=288))
2026-04-23 09:09:19.598 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:09:21.534 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:09:21.545 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=293))
2026-04-23 09:10:55.118 12114-12114 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:10:55.118 12114-12114 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:10:55.124 12114-12114 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:10:55.124 12114-12209 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:10:55.126 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:10:55.140 12114-12205 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:10:55.145 12114-12209 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:10:55.149 12114-12179 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=387))
2026-04-23 09:10:55.155 12114-12114 WindowOnBackDispatcher  com.scmessenger.android              W  OnBackInvokedCallback is not enabled for the application.
                                                                                                    Set 'android:enableOnBackInvokedCallback="true"' in the application manifest.
2026-04-23 09:10:55.155 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=7
2026-04-23 09:10:55.155 12114-12114 MainActivity            com.scmessenger.android              D  MainActivity resumed
2026-04-23 09:10:55.156 12114-12209 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:10:55.157 12114-12114 MainActivity            com.scmessenger.android              D  All permissions already granted
2026-04-23 09:10:55.158 12114-12209 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:10:55.159 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=8
2026-04-23 09:10:55.160 12114-12209 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:10:55.160 12114-12179 BleGattServer           com.scmessenger.android              W  GATT server already running
2026-04-23 09:10:55.161 12114-12210 BleScanner              com.scmessenger.android              D  BLE scan already in progress, reusing existing session
2026-04-23 09:10:55.162 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=7
2026-04-23 09:10:55.162 12114-12179 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:10:55.164 12114-12180 WifiTransportManager    com.scmessenger.android              D  WifiTransportManager initialized
2026-04-23 09:10:55.164 12114-12209 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:10:55.166 12114-12209 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:10:55.166 12114-12180 WifiTransportManager    com.scmessenger.android              D  WiFi P2P discovery already active; skipping duplicate start
2026-04-23 09:10:55.169 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=8
2026-04-23 09:10:55.170 12114-12209 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9
2026-04-23 09:10:55.174 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=7
2026-04-23 09:10:55.174 12114-12179 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:10:55.174 12114-12209 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:10:55.175 12114-12209 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:10:55.176 12114-12179 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:10:55.176 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=8
2026-04-23 09:10:55.177 12114-12209 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9
2026-04-23 09:10:55.179 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=7
2026-04-23 09:10:55.179 12114-12209 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:10:55.181 12114-12209 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:10:55.183 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=8
2026-04-23 09:10:55.185 12114-12209 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:10:55.188 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=7
2026-04-23 09:10:55.190 12114-12209 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:10:55.190 12114-12209 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:10:55.193 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=8
2026-04-23 09:10:55.194 12114-12209 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:10:55.195 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=7
2026-04-23 09:10:55.196 12114-12209 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:10:55.196 12114-12209 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:10:55.197 12114-12209 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=8
2026-04-23 09:10:55.199 12114-12209 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=9), next attempt in 60000ms
2026-04-23 09:10:55.244 12114-12114 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:10:55.244 12114-12114 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:ca9e6d87: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:10:55.473 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:10:56.639 12114-12209 MainViewMo...ntityState com.scmessenger.android              D  refreshIdentityState() called
2026-04-23 09:10:56.641 12114-12209 MainViewMo...ntityState com.scmessenger.android              D  Identity initialized state: true
2026-04-23 09:10:58.536 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:11:00.157 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:11:00.167 12114-12179 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=392))
2026-04-23 09:11:00.219 12114-12179 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:11:00.225 12114-12179 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:11:01.631 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:11:03.276 12114-12116 ssenger.android         com.scmessenger.android              I  Thread[2,tid=12116,WaitingInMainSignalCatcherLoop,Thread*=0xb4000076a7917f50,peer=0x20039a8,"Signal Catcher"]: reacting to signal 3
2026-04-23 09:11:03.276 12114-12116 ssenger.android         com.scmessenger.android              I  
2026-04-23 09:11:03.623 12114-12116 ssenger.android         com.scmessenger.android              I  Wrote stack traces to tombstoned
2026-04-23 09:11:04.705 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:11:05.169 12114-12179 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:11:05.175 12114-12210 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=397))
2026-04-23 09:11:06.201 12114-12139 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:11:06.202 12114-12139 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:11:06.203 12114-12139 MeshRepository          com.scmessenger.android              I  ✓ Internet transport (Swarm) initiated and bridge wired
2026-04-23 09:11:07.975 12114-12114 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:11:08.097 12114-12789 HWUI                    com.scmessenger.android              I  Davey! duration=11448ms; Flags=0, FrameTimelineVsyncId=75261042, IntendedVsync=283088526140740, Vsync=283088526140740, InputEventId=380876690, HandleInputStart=283088531188801, AnimationStart=283088531210448, PerformTraversalsStart=283099891812952, DrawStart=283099891849329, FrameDeadline=283088542740740, FrameStartTime=283088531184732, FrameInterval=16709782, WorkloadTarget=16600000, SyncQueued=283099971936935, SyncStart=283099971975102, IssueDrawCommandsStart=283099972118047, SwapBuffers=283099973757410, FrameCompleted=283099974670740, DequeueBufferDuration=11434, QueueBufferDuration=300171, GpuCompleted=283099974670740, SwapBuffersCompleted=283099974107549, DisplayPresentTime=283093487204469, CommandSubmissionCompleted=283099973757410, 
2026-04-23 09:11:10.126 12114-12170 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5001ms (threshold: 10000ms)
2026-04-23 09:11:10.179 12114-12210 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:11:10.185 12114-12210 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=402))
2026-04-23 09:11:10.879 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:11:11.258 12114-12210 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:11:11.262 12114-12210 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:11:14.027 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:11:15.193 12114-12210 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:11:15.204 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=407))
2026-04-23 09:11:16.735  1619-13949 ActivityManager         system_server                        E  ANR in com.scmessenger.android (com.scmessenger.android/.ui.MainActivity)
                                                                                                    PID: 12114
                                                                                                    Reason: Input dispatching timed out (1e0d494 com.scmessenger.android/com.scmessenger.android.ui.MainActivity is not responding. Waited 5001ms for MotionEvent).
                                                                                                    Parent: com.scmessenger.android/.ui.MainActivity
                                                                                                    ErrorId: a479ad12-5fd1-4972-99c7-a8e88d1adffd
                                                                                                    Frozen: false
                                                                                                    Load: 1.55 / 2.02 / 2.49
                                                                                                    ----- Output from /proc/pressure/memory -----
                                                                                                    some avg10=0.09 avg60=0.41 avg300=0.50 total=1519667840
                                                                                                    full avg10=0.09 avg60=0.26 avg300=0.32 total=879441298
                                                                                                    ----- End output from /proc/pressure/memory -----
                                                                                                    ----- Output from /proc/pressure/cpu -----
                                                                                                    some avg10=3.88 avg60=5.35 avg300=5.42 total=26887560321
                                                                                                    full avg10=0.00 avg60=0.00 avg300=0.00 total=0
                                                                                                    ----- End output from /proc/pressure/cpu -----
                                                                                                    ----- Output from /proc/pressure/io -----
                                                                                                    some avg10=0.10 avg60=1.12 avg300=1.56 total=4767801658
                                                                                                    full avg10=0.00 avg60=0.57 avg300=0.88 total=2481787221
                                                                                                    ----- End output from /proc/pressure/io -----
                                                                                                    
                                                                                                    CPU usage from 2ms to 13443ms later (2026-04-23 09:11:03.239 to 2026-04-23 09:11:16.680) with 99% awake:
                                                                                                      124% 12114/com.scmessenger.android: 123% user + 1.2% kernel / faults: 7131 minor 1879 major
                                                                                                      16% 1619/system_server: 7.2% user + 9.1% kernel / faults: 7949 minor 1550 major
                                                                                                      2.5% 21975/com.google.android.inputmethod.latin: 1.4% user + 1.1% kernel / faults: 2624 minor 561 major
                                                                                                      2.2% 2261/com.android.systemui: 1.4% user + 0.8% kernel / faults: 2319 minor 351 major
                                                                                                      1.6% 11332/com.lemon.lvoverseas: 0.6% user + 0.9% kernel / faults: 538 minor 115 major
                                                                                                      1.3% 546/surfaceflinger: 0.8% user + 0.5% kernel / faults: 10 minor
                                                                                                      1.2% 2476/com.android.hbmsvmanager: 1% user + 0.2% kernel / faults: 1471 minor 110 major
                                                                                                      1.1% 365/ueventd: 1.1% user + 0% kernel
                                                                                                      1.1% 148/sched_pmu_wq: 0% user + 1.1% kernel
                                                                                                      1% 1484/adbd: 0.3% user + 0.6% kernel / faults: 6 minor
                                                                                                    22% TOTAL: 18% user + 3.2% kernel + 0.2% iowait + 0.4% irq + 0.2% softirq
                                                                                                    CPU usage from 50ms to 596ms later (2026-04-23 09:11:03.287 to 2026-04-23 09:11:03.833):
                                                                                                      224% 12114/com.scmessenger.android: 217% user + 6.4% kernel / faults: 536 minor 10 major
                                                                                                        99% 12114/ssenger.android: 99% user + 0% kernel
                                                                                                        96% 12139/DefaultDispatch: 96% user + 0% kernel
                                                                                                        19% 12116/Signal Catcher: 16% user + 3.2% kernel
                                                                                                      112% 1619/system_server: 50% user + 61% kernel / faults: 3872 minor 1349 major
                                                                                                        55% 13950/AnrAuxiliaryTas: 21% user + 33% kernel
                                                                                                        27% 1630/Signal Catcher: 14% user + 12% kernel
                                                                                                        8.4% 13949/AnrConsumer: 0% user + 8.4% kernel
                                                                                                        4.2% 1728/PowerManagerSer: 4.2% user + 0% kernel
                                                                                                        2.1% 2161/NetworkPolicy: 0% user + 2.1% kernel
                                                                                                        2.1% 2230/ClipboardServic: 0% user + 2.1% kernel
                                                                                                        2.1% 2233/PhotonicModulat: 0% user + 2.1% kernel
                                                                                                        2.1% 2453/MobileDataStats: 2.1% user + 0% kernel
                                                                                                        2.1% 26345/SurfaceSyncGrou: 0% user + 2.1% kernel
                                                                                                      9.5% 365/ueventd: 9.5% user + 0% kernel
                                                                                                      9.7% 546/surfaceflinger: 7.7% user + 1.9% kernel
                                                                                                        3.8% 546/surfaceflinger: 3.8% user + 0% kernel
                                                                                                        1.9% 596/RenderEngine: 1.9% user + 0% kernel
                                                                                                        1.9% 648/app: 1.9% user + 0% kernel
                                                                                                        1.9% 1732/binder:546_3: 1.9% user + 0% kernel
                                                                                                      8.1% 1223/android.hardware.usb-service.gs101: 8.1% user + 0% kernel
                                                                                                        4% 1233/android.hardwar: 4% user + 0% kernel
                                                                                                        2% 2200/android.hardwar: 2% user + 0% kernel
                                                                                                      5.5% 92/kswapd0: 0% user + 5.5% kernel
                                                                                                      5.8% 548/android.hardware.graphics.composer@2.4-service: 1.9% user + 3.8% kernel
                                                                                                        5.8% 597/HwBinder:548_1: 1.9% user + 3.8% kernel
                                                                                                      4.6% 3010/pixelstats-vendor: 2.3% user + 2.3% kernel
                                                                                                        2.3% 3027/pixelstats-vend: 0% user + 2.3% kernel
                                                                                                      1.8% 51/rcuog/4: 0% user + 1.8% kernel
                                                                                                      1.8% 59/rcuop/5: 0% user + 1.8% kernel
                                                                                                      1.8% 148/sched_pmu_wq: 0% user + 1.8% kernel
                                                                                                      1.8% 157/eh_comp_thread: 0% user + 1.8% kernel
                                                                                                      1.8% 181/thermal_BIG: 0% user + 1.8% kernel
                                                                                                      1.8% 218/thermal_LITTLE: 0% user + 1.8% kernel
                                                                                                      1.8% 275/decon0_kthread: 0% user + 1.8% kernel
                                                                                                      1.9% 481/logd: 0% user + 1.9% kernel
                                                                                                        1.9% 496/logd.writer: 0% user + 1.9% kernel
                                                                                                        1.9% 9180/logd.reader.per: 0% user + 1.9% kernel
2026-04-23 09:11:18.617 12114-12114 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:11:18.626 12114-12114 Choreographer           com.scmessenger.android              I  Skipped 1312 frames!  The application may be doing too much work on its main thread.
2026-04-23 09:11:18.697 12114-13924 HWUI                    com.scmessenger.android              I  Davey! duration=21976ms; Flags=0, FrameTimelineVsyncId=75261130, IntendedVsync=283088593170353, Vsync=283110511736241, InputEventId=1062720610, HandleInputStart=283110516668548, AnimationStart=283110516669728, PerformTraversalsStart=283110543933441, DrawStart=283110543971934, FrameDeadline=283100022317859, FrameStartTime=283110514955698, FrameInterval=16666667, WorkloadTarget=16600000, SyncQueued=283110566546356, SyncStart=283110567052297, IssueDrawCommandsStart=283110567199066, SwapBuffers=283110570027842, FrameCompleted=283110570417653, DequeueBufferDuration=10173, QueueBufferDuration=192098, GpuCompleted=283110570417653, SwapBuffersCompleted=283110570260386, DisplayPresentTime=283093520628338, CommandSubmissionCompleted=283110570027842, 
2026-04-23 09:11:20.224 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:11:20.249 12114-12139 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=412))
2026-04-23 09:11:25.145 12114-12114 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:11:25.146 12114-12114 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:11:25.147 12114-12114 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:11:25.255 12114-12139 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:11:25.264 12114-12139 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=417))
2026-04-23 09:11:30.278 12114-12139 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:11:30.294 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=422))
2026-04-23 09:11:34.892 12114-12114 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:11:34.892 12114-12114 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:438b5fa8: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:11:35.299 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:11:35.308 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=427))
2026-04-23 09:11:35.820 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:11:40.316 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:11:40.325 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=432))
2026-04-23 09:11:45.339 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:11:45.353 12114-12210 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=437))
2026-04-23 09:11:50.370 12114-12210 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:11:50.393 12114-12210 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=442))
2026-04-23 09:11:51.293 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:11:54.380 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:11:55.160 12114-12114 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:11:55.160 12114-12114 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:11:55.162 12114-12114 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:11:55.402 12114-12210 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:11:55.414 12114-12139 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=447))
2026-04-23 09:11:57.462 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:11:59.243 12114-12139 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:11:59.270 12114-12139 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:11:59.286 12114-12139 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:11:59.305 12114-12139 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:11:59.316 12114-12139 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:11:59.324 12114-12139 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:11:59.332 12114-12139 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:11:59.338 12114-12139 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:11:59.343 12114-12139 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:11:59.348 12114-12139 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:11:59.352 12114-12139 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:11:59.356 12114-12139 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:11:59.360 12114-12139 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:11:59.363 12114-12139 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=10), next attempt in 60000ms
2026-04-23 09:12:00.420 12114-12139 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:12:00.428 12114-12139 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=452))
2026-04-23 09:12:00.555 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:12:05.434 12114-12139 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:12:05.441 12114-12139 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=457))
2026-04-23 09:12:06.786 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:12:09.897 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:12:10.452 12114-12139 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:12:10.462 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=462))
2026-04-23 09:12:12.996 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:12:15.480 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:12:15.506 12114-12139 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=467))
2026-04-23 09:12:20.519 12114-12139 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:12:20.534 12114-12210 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=472))
2026-04-23 09:12:24.966 12114-12114 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:12:24.966 12114-12114 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:bd65e96f: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:12:25.178 12114-12114 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:12:25.179 12114-12114 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:12:25.180 12114-12114 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:12:25.383 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:12:25.553 12114-12210 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:12:25.578 12114-12209 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=477))
2026-04-23 09:12:30.598 12114-12209 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:12:30.624 12114-12139 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=482))
2026-04-23 09:12:34.189 12114-12114 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:12:34.189 12114-12114 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:cc2f82ff: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:12:35.409 12114-12114 WindowOnBackDispatcher  com.scmessenger.android              W  OnBackInvokedCallback is not enabled for the application.
                                                                                                    Set 'android:enableOnBackInvokedCallback="true"' in the application manifest.
2026-04-23 09:12:35.627 12114-12139 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:12:35.634 12114-12210 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=487))
2026-04-23 09:12:37.752 12114-12196 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:12:37.807 12114-12114 MainActivity            com.scmessenger.android              D  MainActivity paused
2026-04-23 09:12:38.025 12114-12114 AnrWatchdog             com.scmessenger.android              I  ANR watchdog stopped (total ANR events=0)
2026-04-23 09:12:38.031 12114-12114 MainActivity            com.scmessenger.android              D  ANR watchdog stopped
2026-04-23 09:12:38.036 12114-12114 WindowOnBackDispatcher  com.scmessenger.android              D  setTopOnBackInvokedCallback (unwrapped): null
2026-04-23 09:12:38.049 12114-12114 ViewRootImpl            com.scmessenger.android              D  Skipping stats log for color mode
---------------------------- PROCESS ENDED (12114) for package com.scmessenger.android ----------------------------
---------------------------- PROCESS STARTED (14251) for package com.scmessenger.android ----------------------------
2026-04-23 09:12:45.300 14251-14251 ssenger.android         com.scmessenger.android              I  Using CollectorTypeCMC GC.
2026-04-23 09:12:45.306 14251-14251 nativeloader            com.scmessenger.android              D  Load libframework-connectivity-tiramisu-jni.so using APEX ns com_android_tethering for caller /apex/com.android.tethering/javalib/framework-connectivity-t.jar: ok
2026-04-23 09:12:45.649 14251-14251 nativeloader            com.scmessenger.android              D  Configuring clns-9 for other apk /data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk. target_sdk_version=35, uses_libraries=, library_path=/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/lib/arm64:/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk!/lib/arm64-v8a, permitted_path=/data:/mnt/expand:/data/user/0/com.scmessenger.android
2026-04-23 09:12:45.650 14251-14251 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 202956589; UID 10499; state: ENABLED
2026-04-23 09:12:45.655 14251-14251 GraphicsEnvironment     com.scmessenger.android              V  Currently set values for:
2026-04-23 09:12:45.655 14251-14251 GraphicsEnvironment     com.scmessenger.android              V    angle_gl_driver_selection_pkgs=[com.android.angle]
2026-04-23 09:12:45.655 14251-14251 GraphicsEnvironment     com.scmessenger.android              V    angle_gl_driver_selection_values=[angle]
2026-04-23 09:12:45.655 14251-14251 GraphicsEnvironment     com.scmessenger.android              V  com.scmessenger.android is not listed in per-application setting
2026-04-23 09:12:45.655 14251-14251 GraphicsEnvironment     com.scmessenger.android              V  No special selections for ANGLE, returning default driver choice
2026-04-23 09:12:45.655 14251-14251 GraphicsEnvironment     com.scmessenger.android              V  Neither updatable production driver nor prerelease driver is supported.
2026-04-23 09:12:45.672 14251-14251 WM-WrkMgrInitializer    com.scmessenger.android              D  Initializing WorkManager with default configuration.
2026-04-23 09:12:45.691 14251-14278 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 312399441; UID 10499; state: ENABLED
2026-04-23 09:12:45.700 14251-14251 MeshApplication         com.scmessenger.android              I  SCMessenger application started
2026-04-23 09:12:45.704 14251-14280 StorageManager          com.scmessenger.android              D  Performing startup storage maintenance...
2026-04-23 09:12:45.706 14251-14282 DisplayManager          com.scmessenger.android              I  Choreographer implicitly registered for the refresh rate.
2026-04-23 09:12:45.707 14251-14280 StorageManager          com.scmessenger.android              D  Logs rotated successfully.
2026-04-23 09:12:45.708 14251-14282 vulkan                  com.scmessenger.android              D  searching for layers in '/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/lib/arm64'
2026-04-23 09:12:45.708 14251-14282 vulkan                  com.scmessenger.android              D  searching for layers in '/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk!/lib/arm64-v8a'
2026-04-23 09:12:45.712 14251-14280 StorageManager          com.scmessenger.android              D  Startup maintenance complete. Available storage: 17857 MB
2026-04-23 09:12:45.712 14251-14251 DesktopExperienceFlags  com.scmessenger.android              D  Toggle override initialized to: false
2026-04-23 09:12:45.713 14251-14280 MeshApplic...n$onCreate com.scmessenger.android              I  Startup storage maintenance completed
2026-04-23 09:12:45.733 14251-14251 MeshRepository          com.scmessenger.android              D  MeshRepository initialized with storage: /data/user/0/com.scmessenger.android/files
2026-04-23 09:12:45.736 14251-14251 MeshRepository          com.scmessenger.android              D  AND-CONTACTS-WIPE-001: Reinstall check - hasIdentityBackup=true, contactsOnDisk=true, historyOnDisk=true
2026-04-23 09:12:45.736 14251-14251 MeshRepository          com.scmessenger.android              D  AND-CONTACTS-WIPE-001: Normal startup - all data present
2026-04-23 09:12:45.740 14251-14251 MeshRepository          com.scmessenger.android              D  Contacts migration already completed, skipping
2026-04-23 09:12:45.744 14251-14251 nativeloader            com.scmessenger.android              D  Load /data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/lib/arm64/libjnidispatch.so using class loader ns clns-9 (caller=/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk!classes17.dex): ok
2026-04-23 09:12:45.863 14251-14251 MeshRepository          com.scmessenger.android              I  all_managers_init_success
2026-04-23 09:12:45.864 14251-14251 MeshRepository          com.scmessenger.android              I  All managers initialized successfully
2026-04-23 09:12:45.865 14251-14251 MeshRepository          com.scmessenger.android              D  AND-CONTACTS-WIPE-001: Contact data verification - Found 0 contacts
2026-04-23 09:12:45.865 14251-14251 MeshRepository          com.scmessenger.android              W  AND-CONTACTS-WIPE-001: WARNING - No contacts found. If this is unexpected, data recovery may be needed.
2026-04-23 09:12:45.867 14251-14251 MainActivity            com.scmessenger.android              D  MainActivity created
2026-04-23 09:12:45.868 14251-14251 AnrWatchdog             com.scmessenger.android              I  ANR watchdog started (check=5000ms, threshold=10000ms)
2026-04-23 09:12:45.868 14251-14251 MainActivity            com.scmessenger.android              I  ANR watchdog started for UI thread monitoring
2026-04-23 09:12:45.868 14251-14280 MeshRepository          com.scmessenger.android              D  Corruption check: contacts=0, messages=0
2026-04-23 09:12:45.869 14251-14280 MeshRepository          com.scmessenger.android              D  Database integrity check passed
2026-04-23 09:12:45.870 14251-14251 MainActivity            com.scmessenger.android              D  All permissions already granted
2026-04-23 09:12:45.875 14251-14251 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 309578419; UID 10499; state: ENABLED
2026-04-23 09:12:45.875 14251-14251 DesktopModeFlags        com.scmessenger.android              D  Toggle override initialized to: OVERRIDE_UNSET
2026-04-23 09:12:45.886 14251-14280 MeshRepository          com.scmessenger.android              I  Repository background initialization completed
2026-04-23 09:12:45.887 14251-14279 MeshReposi...aintenance com.scmessenger.android              D  Storage maintenance check: free=17857MB / total=112912MB
2026-04-23 09:12:45.887 14251-14280 MainActivity            com.scmessenger.android              D  UI components initialization completed
2026-04-23 09:12:45.887 14251-14251 ContentCaptureHelper    com.scmessenger.android              I  Setting logging level to OFF
2026-04-23 09:12:45.892 14251-14251 MainActivity            com.scmessenger.android              D  MainActivity resumed
2026-04-23 09:12:45.893 14251-14251 MainActivity            com.scmessenger.android              D  All permissions already granted
2026-04-23 09:12:45.893 14251-14251 MeshRepository          com.scmessenger.android              D  Permission refresh skipped: mesh service is not running
2026-04-23 09:12:45.898 14251-14251 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 349153669; UID 10499; state: ENABLED
2026-04-23 09:12:45.988 14251-14251 MainViewModel           com.scmessenger.android              D  MainViewModel init
2026-04-23 09:12:45.989 14251-14279 MainViewMo...rageStatus com.scmessenger.android              D  Storage refreshed: 17857 MB available (Low=false)
2026-04-23 09:12:45.990 14251-14251 MainViewModel           com.scmessenger.android              D  MeshRepository service state: STOPPED
2026-04-23 09:12:45.990 14251-14332 MainViewMo...ntityState com.scmessenger.android              D  refreshIdentityState() called
2026-04-23 09:12:45.991 14251-14332 MainViewMo...ntityState com.scmessenger.android              D  Identity initialized state: true
2026-04-23 09:12:45.993 14251-14332 MainViewMo...ntityState com.scmessenger.android              D  Identity is initialized but install choice not completed, fixing preference...
2026-04-23 09:12:46.008 14251-14251 VRI[MainActivity]       com.scmessenger.android              D  WindowInsets changed: 1080x2400 statusBars:[0,132,0,0] navigationBars:[0,0,0,63] mandatorySystemGestures:[0,164,0,84] 
2026-04-23 09:12:46.009 14251-14251 WindowOnBackDispatcher  com.scmessenger.android              D  setTopOnBackInvokedCallback (unwrapped): android.view.ViewRootImpl$$ExternalSyntheticLambda25@1f45d63
2026-04-23 09:12:46.010 14251-14332 PreferencesRepository   com.scmessenger.android              I  Install choice completed: true
2026-04-23 09:12:46.011 14251-14332 MainViewMo...ntityState com.scmessenger.android              D  Identity is initialized but onboarding not completed, fixing preference...
2026-04-23 09:12:46.012 14251-14281 PreferencesRepository   com.scmessenger.android              I  Onboarding completed: true
2026-04-23 09:12:46.039 14251-14261 ssenger.android         com.scmessenger.android              I  Compiler allocated 5049KB to compile void android.view.ViewRootImpl.performTraversals()
2026-04-23 09:12:46.048 14251-14251 HWUI                    com.scmessenger.android              I  Using FreeType backend (prop=Auto)
2026-04-23 09:12:46.079 14251-14251 MeshServiceViewModel    com.scmessenger.android              D  MeshServiceViewModel initialized
2026-04-23 09:12:46.085 14251-14281 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 09:12:46.086 14251-14281 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=null, initialized=null, nickname=null
2026-04-23 09:12:46.086 14251-14334 MeshReposi...edDeferred com.scmessenger.android              D  Lazy starting MeshService for Identity access...
2026-04-23 09:12:46.086 14251-14281 DashboardViewModel      com.scmessenger.android              D  Topology built: 0 nodes, 0 edges
2026-04-23 09:12:46.087 14251-14281 DashboardV...efreshData com.scmessenger.android              D  Dashboard data refreshed
2026-04-23 09:12:46.087 14251-14332 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 09:12:46.088 14251-14281 MeshReposi...edDeferred com.scmessenger.android              D  MeshService is already starting, skipping redundant init
2026-04-23 09:12:46.089 14251-14332 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=null, initialized=null, nickname=null
2026-04-23 09:12:46.089 14251-14332 DashboardViewModel      com.scmessenger.android              D  Topology built: 0 nodes, 0 edges
2026-04-23 09:12:46.090 14251-14332 DashboardV...efreshData com.scmessenger.android              D  Dashboard data refreshed
2026-04-23 09:12:46.091 14251-14334 MeshRepository          com.scmessenger.android              I  service_start_requested
2026-04-23 09:12:46.091 14251-14334 MeshRepository          com.scmessenger.android              D  Starting MeshService...
2026-04-23 09:12:46.091 14251-14251 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:196068): avc:  denied  { search } for  name="/" dev="cgroup2" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup_v2:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:12:46.091 14251-14251 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:196069): avc:  denied  { search } for  name="/" dev="cgroup2" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup_v2:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:12:46.091 14251-14251 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:196070): avc:  denied  { search } for  name="/" dev="cgroup" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:12:46.091 14251-14251 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:196071): avc:  denied  { search } for  name="/" dev="cgroup" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:12:46.129 14251-14334 MeshRepository          com.scmessenger.android              I  SmartTransportRouter initialized for intelligent transport selection
2026-04-23 09:12:46.133 14251-14334 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:12:46.133 14251-14334 NetworkDetector         com.scmessenger.android              I  NetworkDetector monitoring started
2026-04-23 09:12:46.133 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:12:46.134 14251-14334 MeshRepository          com.scmessenger.android              I  NetworkDetector started — cellular-aware transport fallback active
2026-04-23 09:12:46.134 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:12:46.136 14251-14344 NetworkDetector         com.scmessenger.android              W  Cellular network detected (CELLULAR) — blocking ports: [9001, 9010, 4001, 5001]
2026-04-23 09:12:46.136 14251-14332 MeshReposi...hangeWatch com.scmessenger.android              I  Network type changed: WIFI → CELLULAR, resetting circuit breakers and re-bootstrapping
2026-04-23 09:12:46.139 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: CELLULAR, blocked ports: [9001, 9010, 4001, 5001]
2026-04-23 09:12:46.140 14251-14332 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:12:46.142 14251-14344 NetworkDetector         com.scmessenger.android              W  Cellular network detected (CELLULAR) — blocking ports: [9001, 9010, 4001, 5001]
2026-04-23 09:12:46.144 14251-14332 MeshRepository          com.scmessenger.android              I  Racing bootstrap: network=CELLULAR, transports=wss→tcp→quic→tcp→ws
2026-04-23 09:12:46.144 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: CELLULAR, blocked ports: [9001, 9010, 4001, 5001]
2026-04-23 09:12:46.145 14251-14332 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:12:46.158 14251-14332 MeshReposi...$Companion com.scmessenger.android              I  Using bootstrap source: StaticFallback with 4 nodes
2026-04-23 09:12:46.222 14251-14353 MainViewMo...ntityState com.scmessenger.android              D  refreshIdentityState() called
2026-04-23 09:12:46.225 14251-14353 MainViewMo...ntityState com.scmessenger.android              D  Identity initialized state: true
2026-04-23 09:12:46.226 14251-14353 MainViewMo...ntityState com.scmessenger.android              D  Identity is initialized but install choice not completed, fixing preference...
2026-04-23 09:12:46.229 14251-14332 PreferencesRepository   com.scmessenger.android              I  Install choice completed: true
2026-04-23 09:12:46.231 14251-14332 MainViewMo...ntityState com.scmessenger.android              D  Identity is initialized but onboarding not completed, fixing preference...
2026-04-23 09:12:46.233 14251-14353 PreferencesRepository   com.scmessenger.android              I  Onboarding completed: true
2026-04-23 09:12:46.275 14251-14251 MainViewModel           com.scmessenger.android              D  Preference onboardingCompleted: true
2026-04-23 09:12:46.276 14251-14251 MainViewModel           com.scmessenger.android              D  Preference installChoiceCompleted: true
2026-04-23 09:12:46.334 14251-14251 Conversati...adMessages com.scmessenger.android              D  Loaded 0 messages
2026-04-23 09:12:46.336 14251-14251 Conversati...$loadStats com.scmessenger.android              D  Loaded stats: HistoryStats(totalMessages=0, sentCount=0, receivedCount=0, undeliveredCount=0)
2026-04-23 09:12:46.339 14251-14251 Conversati...ockedPeers com.scmessenger.android              D  Loaded 0 blocked peers
2026-04-23 09:12:46.986 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:12:46.995 14251-14251 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:12:46.995 14251-14251 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:9ba99732: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:12:47.665 14251-14280 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:443 = blocked
2026-04-23 09:12:47.665 14251-14333 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:9010 = blocked
2026-04-23 09:12:47.666 14251-14281 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:9001 = blocked
2026-04-23 09:12:47.666 14251-14279 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:443 = blocked
2026-04-23 09:12:47.694 14251-14279 MeshRepository          com.scmessenger.android              W  All bootstrap addresses failed or timed out, falling back to mDNS
2026-04-23 09:12:47.698 14251-14279 MeshRepository          com.scmessenger.android              I  All relay bootstrap failed, attempting mDNS local discovery
2026-04-23 09:12:50.059 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:12:51.202 14251-14408 ProfileInstaller        com.scmessenger.android              D  Installing profile for com.scmessenger.android
2026-04-23 09:12:52.706 14251-14279 MeshRepository          com.scmessenger.android              E  mDNS fallback: no LAN peers discovered within timeout
2026-04-23 09:12:52.709 14251-14279 MeshReposi...hangeWatch com.scmessenger.android              I  Network type changed: CELLULAR → WIFI, resetting circuit breakers and re-bootstrapping
2026-04-23 09:12:52.710 14251-14279 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:12:52.714 14251-14279 MeshRepository          com.scmessenger.android              I  Racing bootstrap: network=WIFI, transports=quic→tcp→wss→ws
2026-04-23 09:12:52.715 14251-14279 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:12:52.726 14251-14279 MeshReposi...$Companion com.scmessenger.android              I  Using bootstrap source: StaticFallback with 4 nodes
2026-04-23 09:12:54.230 14251-14279 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:443 = blocked
2026-04-23 09:12:54.230 14251-14333 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:9010 = blocked
2026-04-23 09:12:54.230 14251-14281 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:9001 = blocked
2026-04-23 09:12:54.231 14251-14280 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:443 = blocked
2026-04-23 09:12:54.246 14251-14280 MeshRepository          com.scmessenger.android              W  All bootstrap addresses failed or timed out, falling back to mDNS
2026-04-23 09:12:54.250 14251-14280 MeshRepository          com.scmessenger.android              I  All relay bootstrap failed, attempting mDNS local discovery
2026-04-23 09:12:56.705 14251-14334 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:12:56.706 14251-14251 MainViewModel           com.scmessenger.android              D  MeshRepository service state: RUNNING
2026-04-23 09:12:56.707 14251-14334 MeshRepository          com.scmessenger.android              I  SC_IDENTITY_OWN p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9 pk=374d73ebc21d677f889b806d6065a7177a2c5748f368036bd073e949bba5a2d4
2026-04-23 09:12:56.708 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=0))
2026-04-23 09:12:56.711 14251-14334 MeshRepository          com.scmessenger.android              I  Mesh service started successfully
2026-04-23 09:12:56.711 14251-14333 MainViewMo...ntityState com.scmessenger.android              D  refreshIdentityState() called
2026-04-23 09:12:56.716 14251-14334 MeshReposi...edDeferred com.scmessenger.android              D  MeshService started lazily
2026-04-23 09:12:56.728 14251-14333 MainViewMo...ntityState com.scmessenger.android              D  Identity initialized state: true
2026-04-23 09:12:56.738 14251-14333 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:12:56.738 14251-14333 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:12:56.738 14251-14333 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:12:56.738 14251-14280 AdvertiseSettings       com.scmessenger.android              D  setTxPowerLevel: 2
2026-04-23 09:12:56.739 14251-14280 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:12:56.739 14251-14280 BluetoothLeAdvertiser   com.scmessenger.android              D  TxPower == ADVERTISE_TX_POWER_MEDIUM
2026-04-23 09:12:56.739 14251-14280 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:12:56.742 14251-14333 BackoffStrategy         com.scmessenger.android              D  Backoff strategy reset
2026-04-23 09:12:56.743 14251-14262 ssenger.android         com.scmessenger.android              W  userfaultfd: MOVE ioctl seems unsupported: Connection timed out
2026-04-23 09:12:56.748 14251-14268 BluetoothLeScanner      com.scmessenger.android              D  onScannerRegistered(status=0, scannerId=3): mScannerId=0
2026-04-23 09:12:56.753 14251-14280 BluetoothGattServer     com.scmessenger.android              D  registerCallback()
2026-04-23 09:12:56.754 14251-14333 BleScanner              com.scmessenger.android              I  BLE Scanning started (background=false, fallback=false)
2026-04-23 09:12:56.754 14251-14280 BluetoothGattServer     com.scmessenger.android              D  registerCallback() - UUID=55c7c553-2718-436f-9d6c-d45615f6c784
2026-04-23 09:12:56.757 14251-14268 BluetoothGattServer     com.scmessenger.android              D  onServerRegistered(0)
2026-04-23 09:12:56.757 14251-14280 BluetoothGattServer     com.scmessenger.android              D  addService() - service: 0000df01-0000-1000-8000-00805f9b34fb
2026-04-23 09:12:56.759 14251-14280 BleGattServer           com.scmessenger.android              I  GATT server started with SCMessenger service
2026-04-23 09:12:56.759 14251-14268 BluetoothGattServer     com.scmessenger.android              D  onServiceAdded() - handle=147 uuid=0000df01-0000-1000-8000-00805f9b34fb status=0
2026-04-23 09:12:56.760 14251-14333 BleScanner              com.scmessenger.android              D  Duty cycle started: 10000ms scan / 30000ms interval
2026-04-23 09:12:56.761 14251-14251 BleAdverti...seCallback com.scmessenger.android              I  BLE Advertising started successfully
2026-04-23 09:12:56.763 14251-14280 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (2 bytes)
2026-04-23 09:12:56.778 14251-14280 BleAdvertiser           com.scmessenger.android              D  Identity data set: 313 bytes
2026-04-23 09:12:56.781 14251-14280 BleAdvertiser           com.scmessenger.android              I  BLE Advertising stopped
2026-04-23 09:12:56.782 14251-14280 AdvertiseSettings       com.scmessenger.android              D  setTxPowerLevel: 2
2026-04-23 09:12:56.783 14251-14280 BleAdvertiser           com.scmessenger.android              W  Identity data too large for advertising (313 bytes), using GATT
2026-04-23 09:12:56.784 14251-14280 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:12:56.784 14251-14280 BluetoothLeAdvertiser   com.scmessenger.android              D  TxPower == ADVERTISE_TX_POWER_MEDIUM
2026-04-23 09:12:56.784 14251-14280 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:12:56.785 14251-14280 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:12:56.786 14251-14280 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:12:56.792 14251-14251 BleAdverti...seCallback com.scmessenger.android              I  BLE Advertising started successfully
2026-04-23 09:12:56.795 14251-14280 WifiTransportManager    com.scmessenger.android              D  WifiTransportManager initialized
2026-04-23 09:12:56.802 14251-14251 WifiDirect...2pReceiver com.scmessenger.android              D  WiFi P2P state changed: enabled=true
2026-04-23 09:12:56.804 14251-14280 WifiDirectTransport     com.scmessenger.android              I  WiFi Direct started
2026-04-23 09:12:56.975 14251-14251 WifiDirect...2pReceiver com.scmessenger.android              D  Disconnected from WiFi P2P group
2026-04-23 09:12:56.980 14251-14251 WifiDirect...2pReceiver com.scmessenger.android              D  This device changed
2026-04-23 09:12:56.986 14251-14251 WifiP2pManager          com.scmessenger.android              D  Ignored { when=0 what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:12:56.986 14251-14251 WifiP2pManager          com.scmessenger.android              D  Ignored { when=0 what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:12:56.987 14251-14251 WifiDirect...eDiscovery com.scmessenger.android              D  Service discovery request added
2026-04-23 09:12:56.992 14251-14251 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-1ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:12:56.992 14251-14251 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-2ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:12:56.993 14251-14251 WifiDirect...terService com.scmessenger.android              D  WiFi Direct service registered: scmessenger
2026-04-23 09:12:56.998 14251-14251 WifiTransp...tDiscovery com.scmessenger.android              I  WiFi P2P Discovery started
2026-04-23 09:12:57.038 14251-14251 WifiDirect...rDiscovery com.scmessenger.android              D  Peer discovery started
2026-04-23 09:12:59.261 14251-14281 MeshRepository          com.scmessenger.android              E  mDNS fallback: no LAN peers discovered within timeout
2026-04-23 09:13:01.717 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:13:01.726 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=5))
2026-04-23 09:13:01.811 14251-14281 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:13:01.815 14251-14281 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:13:02.431 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:13:06.735 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:13:06.745 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=10))
2026-04-23 09:13:06.760 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:13:06.765 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:13:08.050 14251-14280 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:13:08.051 14251-14280 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:13:08.052 14251-14280 MeshRepository          com.scmessenger.android              I  ✓ Internet transport (Swarm) initiated and bridge wired
2026-04-23 09:13:08.053 14251-14280 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:13:08.055 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:13:08.056 14251-14280 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:13:08.057 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:13:08.059 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:13:08.060 14251-14280 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:13:08.060 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:13:08.062 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:13:08.063 14251-14280 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:13:08.064 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:13:08.065 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:13:08.066 14251-14280 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:13:08.067 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:13:08.068 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:13:08.069 14251-14280 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:13:08.070 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:13:08.071 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:13:08.072 14251-14280 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:13:08.073 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:13:08.074 14251-14280 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=1), next attempt in 10000ms
2026-04-23 09:13:11.751 14251-14334 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:13:11.757 14251-14334 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=15))
2026-04-23 09:13:13.133 14251-14334 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:13:13.141 14251-14334 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:13:16.763 14251-14251 BleScanner...d$Runnable com.scmessenger.android              W  BLE scan fallback enabled after 20022 ms without mesh advertisements; switching to unfiltered scan
2026-04-23 09:13:16.765 14251-14334 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:13:16.776 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=20))
2026-04-23 09:13:16.783 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:13:16.783 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:13:16.784 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:13:16.792 14251-14251 BleScanner              com.scmessenger.android              I  BLE scan restarted (background=false, fallback=true)
2026-04-23 09:13:16.794 14251-14430 BluetoothLeScanner      com.scmessenger.android              D  onScannerRegistered(status=0, scannerId=3): mScannerId=0
2026-04-23 09:13:21.646 14251-14251 MainActivity            com.scmessenger.android              D  MainActivity paused
2026-04-23 09:13:21.648 14251-14251 VRI[MainActivity]       com.scmessenger.android              D  visibilityChanged oldVisibility=true newVisibility=false
2026-04-23 09:13:21.782 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:13:21.791 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=25))
2026-04-23 09:13:24.086 14251-14333 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:13:24.093 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:13:24.148 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:13:24.153 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:13:24.163 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:13:24.176 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:13:24.182 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:13:24.190 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:13:24.211 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:13:24.217 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:13:24.225 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:13:24.242 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:13:24.249 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:13:24.258 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:13:24.280 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:13:24.287 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:13:24.297 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:13:24.313 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:13:24.319 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:13:24.350 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:13:24.357 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=2), next attempt in 30000ms
2026-04-23 09:13:26.794 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:13:26.800 14251-14334 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=30))
2026-04-23 09:13:27.196 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:13:30.259 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:13:31.808 14251-14334 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:13:31.818 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=35))
2026-04-23 09:13:33.361 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:13:36.767 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:13:36.776 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:13:36.826 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:13:36.842 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=40))
2026-04-23 09:13:39.491 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:13:41.858 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:13:41.868 14251-14334 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=45))
2026-04-23 09:13:46.887 14251-14334 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:13:46.911 14251-14334 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=50))
2026-04-23 09:13:51.804 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:13:51.922 14251-14334 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:13:51.935 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=55))
2026-04-23 09:13:54.882 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:13:56.383 14251-14333 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:13:56.396 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:13:56.401 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:13:56.405 14251-14333 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:13:56.411 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:13:56.425 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:13:56.431 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:13:56.435 14251-14333 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:13:56.446 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:13:56.456 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:13:56.461 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:13:56.465 14251-14333 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:13:56.473 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:13:56.485 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:13:56.490 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:13:56.493 14251-14333 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:13:56.500 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:13:56.509 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:13:56.516 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:13:56.526 14251-14333 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:13:56.534 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:13:56.544 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:13:56.549 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:13:56.554 14251-14333 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:13:56.563 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:13:56.568 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=3), next attempt in 30000ms
2026-04-23 09:13:56.943 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:13:56.955 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=60))
2026-04-23 09:13:57.979 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:14:01.967 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:14:01.978 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=65))
2026-04-23 09:14:06.791 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:14:06.791 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:14:06.792 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:14:06.986 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:14:06.997 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=70))
2026-04-23 09:14:07.227 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:14:10.294 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:14:12.012 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:14:12.026 14251-14334 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=76))
2026-04-23 09:14:16.461 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:14:17.045 14251-14334 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:14:17.068 14251-14334 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=81))
2026-04-23 09:14:22.076 14251-14334 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:14:22.090 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=86))
2026-04-23 09:14:27.150 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:14:27.163 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=91))
2026-04-23 09:14:28.602 14251-14333 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:14:28.614 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:14:28.623 14251-14333 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:14:28.630 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:14:28.636 14251-14333 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:14:28.642 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:14:28.649 14251-14333 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:14:28.656 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:14:28.662 14251-14333 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:14:28.670 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:14:28.678 14251-14333 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:14:28.685 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:14:28.691 14251-14333 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:14:28.696 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=4), next attempt in 16000ms
2026-04-23 09:14:28.776 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:17:34.729 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:17:34.729 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:17:34.730 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:17:34.732 14251-14281 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:17:34.734 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:17:34.750 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:17:34.758 14251-14251 WifiDirect...2pReceiver com.scmessenger.android              D  This device changed
2026-04-23 09:17:34.759 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:17:34.761 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=278))
2026-04-23 09:17:34.762 14251-14251 ViewRootImpl            com.scmessenger.android              D  Skipping stats log for color mode
2026-04-23 09:17:34.762 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:17:34.766 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:17:34.766 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:17:34.767 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:17:34.768 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:17:34.768 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:17:34.771 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:17:34.773 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:17:34.774 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:17:34.775 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:17:34.775 14251-14251 MainActivity            com.scmessenger.android              D  MainActivity resumed
2026-04-23 09:17:34.777 14251-14251 MainActivity            com.scmessenger.android              D  All permissions already granted
2026-04-23 09:17:34.777 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:17:34.778 14251-14281 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=5), next attempt in 32000ms
2026-04-23 09:17:34.779 14251-14333 BleGattServer           com.scmessenger.android              W  GATT server already running
2026-04-23 09:17:34.780 14251-14353 BleScanner              com.scmessenger.android              D  BLE scan already in progress, reusing existing session
2026-04-23 09:17:34.781 14251-14333 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:17:34.783 14251-14352 WifiTransportManager    com.scmessenger.android              D  WifiTransportManager initialized
2026-04-23 09:17:34.785 14251-14352 WifiTransportManager    com.scmessenger.android              D  WiFi P2P discovery already active; skipping duplicate start
2026-04-23 09:17:34.790 14251-14333 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:17:34.792 14251-14333 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:17:34.862 14251-14251 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:17:34.862 14251-14251 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:b7febd2e: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:17:35.743 14251-14251 ContactsVi...adContacts com.scmessenger.android              D  Loaded 0 contacts, filtered nearby peers to 0
2026-04-23 09:17:35.753 14251-14251 WindowOnBackDispatcher  com.scmessenger.android              W  OnBackInvokedCallback is not enabled for the application.
                                                                                                    Set 'android:enableOnBackInvokedCallback="true"' in the application manifest.
2026-04-23 09:17:36.916 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:17:39.771 14251-14353 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:17:39.780 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=283))
2026-04-23 09:17:39.854 14251-14352 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:17:39.858 14251-14352 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:17:40.012 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:17:40.524 14251-14251 MeshServiceViewModel    com.scmessenger.android              D  MeshServiceViewModel initialized
2026-04-23 09:17:40.532 14251-14353 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 09:17:40.532 14251-14333 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 09:17:43.012 14251-14251 WindowOnBackDispatcher  com.scmessenger.android              W  OnBackInvokedCallback is not enabled for the application.
                                                                                                    Set 'android:enableOnBackInvokedCallback="true"' in the application manifest.
2026-04-23 09:17:43.093 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:17:44.786 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:17:44.792 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=288))
2026-04-23 09:17:46.055 14251-14334 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:17:46.056 14251-14334 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:17:46.058 14251-14334 MeshRepository          com.scmessenger.android              I  ✓ Internet transport (Swarm) initiated and bridge wired
2026-04-23 09:17:46.170 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:17:49.260 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:17:49.801 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:17:49.810 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=293))
2026-04-23 09:17:51.136 14251-14281 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:17:51.143 14251-14281 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:17:52.679 14251-14333 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:17:52.680 14251-14333 DashboardViewModel      com.scmessenger.android              D  Topology built: 1 nodes, 0 edges
2026-04-23 09:17:52.681 14251-14333 DashboardV...efreshData com.scmessenger.android              D  Dashboard data refreshed
2026-04-23 09:17:53.316 14251-14353 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:17:53.318 14251-14353 DashboardViewModel      com.scmessenger.android              D  Topology built: 1 nodes, 0 edges
2026-04-23 09:17:53.319 14251-14353 DashboardV...efreshData com.scmessenger.android              D  Dashboard data refreshed
2026-04-23 09:17:54.816 14251-14353 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:17:54.819 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=298))
2026-04-23 09:17:59.822 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:17:59.824 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=303))
2026-04-23 09:18:01.614 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:18:04.758 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:18:04.759 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:18:04.760 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:18:04.839 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:18:04.852 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=308))
2026-04-23 09:18:06.792 14251-14281 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:18:06.798 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:18:06.802 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:18:06.806 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:18:06.809 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:18:06.813 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:18:06.817 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:18:06.820 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:18:06.823 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:18:06.826 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:18:06.829 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:18:06.832 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:18:06.836 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:18:06.839 14251-14281 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=6), next attempt in 60000ms
2026-04-23 09:18:09.859 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:18:09.867 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=313))
2026-04-23 09:18:10.893 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:18:13.983 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:18:14.875 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:18:14.885 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=318))
2026-04-23 09:18:19.899 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:18:19.910 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=323))
2026-04-23 09:18:20.214 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:18:23.320 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:18:24.926 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:18:24.943 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=328))
2026-04-23 09:18:26.421 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:18:29.958 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:18:29.974 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=333))
2026-04-23 09:18:32.630 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:18:34.774 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:18:34.774 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:18:34.776 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:18:34.981 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:18:34.991 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=338))
2026-04-23 09:18:40.006 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:18:40.021 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=344))
2026-04-23 09:18:41.960 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:18:45.036 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:18:45.049 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=349))
2026-04-23 09:18:45.064 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:18:50.063 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:18:50.081 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=354))
2026-04-23 09:18:51.291 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:18:54.418 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:18:55.085 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:18:55.092 14251-14353 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=359))
2026-04-23 09:19:00.097 14251-14353 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:19:00.104 14251-14353 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=364))
2026-04-23 09:19:04.798 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:19:04.798 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:19:04.801 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:19:05.119 14251-14353 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:19:05.140 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=369))
2026-04-23 09:19:10.162 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:19:10.187 14251-14353 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=374))
2026-04-23 09:19:10.899 14251-14353 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:19:10.925 14251-14353 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:19:10.957 14251-14353 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=7
2026-04-23 09:19:10.965 14251-14353 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:19:10.973 14251-14353 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:19:10.982 14251-14353 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=8
2026-04-23 09:19:10.988 14251-14353 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:19:10.998 14251-14353 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=7
2026-04-23 09:19:11.001 14251-14353 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:19:11.005 14251-14353 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:19:11.009 14251-14353 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=8
2026-04-23 09:19:11.011 14251-14353 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9
2026-04-23 09:19:11.015 14251-14353 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=7
2026-04-23 09:19:11.018 14251-14353 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:19:11.019 14251-14353 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:19:11.022 14251-14353 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=8
2026-04-23 09:19:11.024 14251-14353 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9
2026-04-23 09:19:11.028 14251-14353 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=7
2026-04-23 09:19:11.030 14251-14353 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:19:11.031 14251-14353 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:19:11.034 14251-14353 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=8
2026-04-23 09:19:11.035 14251-14353 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:19:11.039 14251-14353 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=7
2026-04-23 09:19:11.040 14251-14353 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:19:11.041 14251-14353 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:19:11.043 14251-14353 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=8
2026-04-23 09:19:11.055 14251-14353 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:19:11.057 14251-14353 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=7
2026-04-23 09:19:11.058 14251-14353 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:19:11.059 14251-14353 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:19:11.061 14251-14353 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=8
2026-04-23 09:19:11.062 14251-14353 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=7), next attempt in 60000ms
2026-04-23 09:19:15.189 14251-14353 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:19:15.193 14251-14353 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=379))
2026-04-23 09:19:16.289 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:19:20.200 14251-14353 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:19:20.209 14251-14353 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=384))
2026-04-23 09:19:22.527 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:19:25.218 14251-14353 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:19:25.224 14251-14353 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=389))
2026-04-23 09:19:28.689 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:19:30.231 14251-14353 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:19:30.241 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=394))
2026-04-23 09:19:31.776 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:19:34.822 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:19:34.823 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:19:34.824 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:19:34.862 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:19:35.262 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:19:35.284 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=399))
2026-04-23 09:19:36.897 14251-14251 WindowOnBackDispatcher  com.scmessenger.android              W  OnBackInvokedCallback is not enabled for the application.
                                                                                                    Set 'android:enableOnBackInvokedCallback="true"' in the application manifest.
2026-04-23 09:19:37.931 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:19:38.657 14251-14251 SettingsViewModel       com.scmessenger.android              D  Loaded mesh settings: MeshSettings(relayEnabled=true, maxRelayBudget=200, batteryFloor=20, bleEnabled=true, wifiAwareEnabled=true, wifiDirectEnabled=true, internetEnabled=true, discoveryMode=NORMAL, onionRouting=false, coverTrafficEnabled=false, messagePaddingEnabled=false, timingObfuscationEnabled=false, notificationsEnabled=true, notifyDmEnabled=true, notifyDmRequestEnabled=true, notifyDmInForeground=false, notifyDmRequestInForeground=true, soundEnabled=true, badgeEnabled=true)
2026-04-23 09:19:40.288 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:19:40.297 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=404))
2026-04-23 09:19:44.090 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:19:44.771 14251-14325 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5002ms (threshold: 10000ms)
2026-04-23 09:19:45.302 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:19:47.180 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:19:48.856 14251-14251 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:19:48.861 14251-14251 MeshServiceViewModel    com.scmessenger.android              D  MeshServiceViewModel initialized
2026-04-23 09:19:49.025 14251-14268 HWUI                    com.scmessenger.android              I  Davey! duration=10377ms; Flags=0, FrameTimelineVsyncId=75326436, IntendedVsync=283605181762020, Vsync=283605181762020, InputEventId=568763548, HandleInputStart=283605182742439, AnimationStart=283605182764818, PerformTraversalsStart=283615472857027, DrawStart=283615472898002, FrameDeadline=283605198362020, FrameStartTime=283605182738126, FrameInterval=16701803, WorkloadTarget=16600000, SyncQueued=283615557036918, SyncStart=283615557196423, IssueDrawCommandsStart=283615557298352, SwapBuffers=283615558368339, FrameCompleted=283615559195894, DequeueBufferDuration=9644, QueueBufferDuration=124552, GpuCompleted=283615559195894, SwapBuffersCompleted=283615558526827, DisplayPresentTime=283610144194264, CommandSubmissionCompleted=283615558368339, 
2026-04-23 09:19:49.782 14251-14325 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5001ms (threshold: 10000ms)
2026-04-23 09:19:50.269 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:19:50.315 14251-14353 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:19:53.355 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:19:54.796 14251-14325 AnrWatchdog             com.scmessenger.android              W  Slow main thread: 5001ms (threshold: 10000ms)
2026-04-23 09:19:55.327 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:19:56.444 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:19:59.528 14251-14251 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:19:59.535 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=414))
2026-04-23 09:19:59.535 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=409))
2026-04-23 09:19:59.535 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=419))
2026-04-23 09:19:59.549 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:19:59.565 14251-14251 Choreographer           com.scmessenger.android              I  Skipped 632 frames!  The application may be doing too much work on its main thread.
2026-04-23 09:19:59.600 14251-14269 HWUI                    com.scmessenger.android              I  Davey! duration=10573ms; Flags=0, FrameTimelineVsyncId=75329403, IntendedVsync=283615572573677, Vsync=283626105907221, InputEventId=0, HandleInputStart=283626116656267, AnimationStart=283626116657529, PerformTraversalsStart=283626134964902, DrawStart=283626134997739, FrameDeadline=283615589173677, FrameStartTime=283626116413144, FrameInterval=16666667, WorkloadTarget=16600000, SyncQueued=283626143056333, SyncStart=283626143223569, IssueDrawCommandsStart=283626143342709, SwapBuffers=283626145999570, FrameCompleted=283626146618385, DequeueBufferDuration=9521, QueueBufferDuration=115926, GpuCompleted=283626146618385, SwapBuffersCompleted=283626146149350, DisplayPresentTime=283610177788014, CommandSubmissionCompleted=283626145999570, 
2026-04-23 09:20:00.342 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:20:00.349 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=424))
2026-04-23 09:20:02.648 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:20:04.843 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:20:04.843 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:20:04.844 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:20:05.371 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:20:05.389 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=429))
2026-04-23 09:20:05.747 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:20:10.400 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:20:10.423 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=434))
2026-04-23 09:20:11.982 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:20:15.095 14251-14333 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:20:15.101 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:20:15.104 14251-14333 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:20:15.110 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:20:15.116 14251-14333 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:20:15.122 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:20:15.130 14251-14333 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:20:15.136 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:20:15.140 14251-14333 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:20:15.144 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:20:15.148 14251-14333 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:20:15.151 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:20:15.154 14251-14333 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:20:15.157 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=8), next attempt in 60000ms
2026-04-23 09:20:15.428 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:20:15.434 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=439))
2026-04-23 09:20:20.443 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:20:20.450 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=444))
2026-04-23 09:20:25.465 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:20:25.476 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=449))
2026-04-23 09:20:27.516 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:20:30.494 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:20:30.515 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=454))
2026-04-23 09:20:34.852 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:20:34.852 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:20:34.853 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:20:35.529 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:20:35.551 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=459))
2026-04-23 09:20:39.973 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:20:40.571 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:20:40.596 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=464))
2026-04-23 09:20:43.109 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:20:45.616 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:20:45.633 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=469))
2026-04-23 09:20:46.205 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:20:49.330 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:20:50.654 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:20:50.675 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=474))
2026-04-23 09:20:52.481 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:20:55.563 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:20:55.678 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:20:55.685 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=479))
2026-04-23 09:20:57.079 14251-14251 WindowOnBackDispatcher  com.scmessenger.android              W  OnBackInvokedCallback is not enabled for the application.
                                                                                                    Set 'android:enableOnBackInvokedCallback="true"' in the application manifest.
2026-04-23 09:21:00.688 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:21:00.695 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=484))
2026-04-23 09:21:04.811 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:21:05.705 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:21:07.881 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:21:08.637 14251-14251 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:21:08.670 14251-14251 Choreographer           com.scmessenger.android              I  Skipped 621 frames!  The application may be doing too much work on its main thread.
2026-04-23 09:21:08.685 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:21:08.685 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:21:08.685 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:21:08.685 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=489))
2026-04-23 09:21:08.704 14251-14269 HWUI                    com.scmessenger.android              I  Davey! duration=10409ms; Flags=0, FrameTimelineVsyncId=75332607, IntendedVsync=283684830043884, Vsync=283695206186328, InputEventId=0, HandleInputStart=283695221095714, AnimationStart=283695221097016, PerformTraversalsStart=283695232924571, DrawStart=283695232970673, FrameDeadline=283684846643884, FrameStartTime=283695220714814, FrameInterval=16705780, WorkloadTarget=16600000, SyncQueued=283695234613780, SyncStart=283695234746999, IssueDrawCommandsStart=283695235052460, SwapBuffers=283695238561290, FrameCompleted=283695239328298, DequeueBufferDuration=22135, QueueBufferDuration=412760, GpuCompleted=283695239328298, SwapBuffersCompleted=283695239082652, DisplayPresentTime=283689724012866, CommandSubmissionCompleted=283695238561290, 
2026-04-23 09:21:10.723 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:21:10.733 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=494))
2026-04-23 09:21:10.989 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:21:15.748 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:21:15.763 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=499))
2026-04-23 09:21:19.207 14251-14280 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:21:19.216 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:21:19.221 14251-14280 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:21:19.227 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:21:19.232 14251-14280 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:21:19.239 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:21:19.243 14251-14280 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:21:19.247 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:21:19.251 14251-14280 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:21:19.255 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:21:19.258 14251-14280 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:21:19.262 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:21:19.264 14251-14280 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:21:19.268 14251-14280 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=9), next attempt in 60000ms
2026-04-23 09:21:20.769 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:21:20.777 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=504))
2026-04-23 09:21:25.786 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:21:25.794 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=509))
2026-04-23 09:21:30.801 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:21:30.810 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=514))
2026-04-23 09:21:32.667 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:21:35.783 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:21:35.812 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:21:35.817 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=519))
2026-04-23 09:21:38.691 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:21:38.691 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:21:38.693 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:21:40.833 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:21:40.856 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=524))
2026-04-23 09:21:41.973 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:21:45.870 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:21:45.885 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=529))
2026-04-23 09:21:48.207 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:21:50.905 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:21:50.929 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=534))
2026-04-23 09:21:54.414 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:21:55.947 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:21:55.970 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=539))
2026-04-23 09:22:00.984 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:22:00.999 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=544))
2026-04-23 09:22:06.005 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:22:06.013 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=550))
2026-04-23 09:22:06.860 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:22:08.700 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:22:08.700 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:22:08.701 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:22:09.006 14251-14251 WindowOnBackDispatcher  com.scmessenger.android              W  OnBackInvokedCallback is not enabled for the application.
                                                                                                    Set 'android:enableOnBackInvokedCallback="true"' in the application manifest.
2026-04-23 09:22:11.032 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:22:11.054 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=555))
2026-04-23 09:22:13.049 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:22:16.059 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:22:16.068 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=560))
2026-04-23 09:22:19.320 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:22:21.084 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:22:21.109 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=565))
2026-04-23 09:22:22.406 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:22:23.329 14251-14281 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:22:23.333 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:22:23.335 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:22:23.337 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:22:23.339 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:22:23.341 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:22:23.343 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:22:23.345 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:22:23.346 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:22:23.348 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:22:23.349 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:22:23.350 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:22:23.352 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:22:23.353 14251-14281 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=10), next attempt in 60000ms
2026-04-23 09:22:25.457 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:22:26.107 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:22:26.110 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=570))
2026-04-23 09:22:28.559 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:22:28.765 14251-14251 MainActivity            com.scmessenger.android              D  MainActivity paused
2026-04-23 09:22:29.340 14251-14251 VRI[MainActivity]       com.scmessenger.android              D  visibilityChanged oldVisibility=true newVisibility=false
2026-04-23 09:22:31.132 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:22:31.152 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=575))
2026-04-23 09:22:31.648 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:22:34.720 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:22:36.175 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:22:36.204 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=580))
2026-04-23 09:22:37.809 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:22:38.706 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:22:38.706 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:22:38.708 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:22:41.211 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:22:41.223 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=585))
2026-04-23 09:22:46.228 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:22:46.237 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=590))
2026-04-23 09:22:51.256 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:22:51.284 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=595))
2026-04-23 09:22:56.285 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:22:56.295 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=600))
2026-04-23 09:23:01.305 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:23:01.319 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=605))
2026-04-23 09:23:06.329 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:23:06.346 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=610))
2026-04-23 09:23:08.723 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:23:08.724 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:23:08.727 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:23:11.360 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:23:11.375 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=615))
2026-04-23 09:23:16.397 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:23:16.424 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=620))
2026-04-23 09:23:21.434 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:23:21.449 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=625))
2026-04-23 09:23:24.033 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:23:26.463 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:23:26.479 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=630))
2026-04-23 09:23:27.409 14251-14281 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:23:27.434 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:23:27.443 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:23:27.452 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:23:27.460 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:23:27.468 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:23:27.476 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:23:27.484 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:23:27.490 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:23:27.497 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:23:27.503 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:23:27.510 14251-14281 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:23:27.517 14251-14281 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:23:27.521 14251-14281 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=11), next attempt in 60000ms
2026-04-23 09:23:31.493 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:23:31.504 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=635))
2026-04-23 09:23:36.526 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:23:36.554 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=640))
2026-04-23 09:23:38.755 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:23:38.756 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:23:38.758 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:32:22.682 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:32:22.682 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:32:22.683 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:32:22.685 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:32:22.686 14251-14280 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:32:22.702 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:32:22.717 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:32:22.719 14251-14251 ViewRootImpl            com.scmessenger.android              D  Skipping stats log for color mode
2026-04-23 09:32:22.722 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1166))
2026-04-23 09:32:22.724 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=9
2026-04-23 09:32:22.725 14251-14280 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:32:22.726 14251-14280 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:32:22.727 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=10
2026-04-23 09:32:22.728 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:32:22.731 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=9
2026-04-23 09:32:22.732 14251-14334 MeshReposi...aintenance com.scmessenger.android              D  Storage maintenance check: free=17765MB / total=112912MB
2026-04-23 09:32:22.732 14251-14280 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:32:22.733 14251-14280 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:32:22.734 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=10
2026-04-23 09:32:22.735 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9
2026-04-23 09:32:22.737 14251-14251 MainActivity            com.scmessenger.android              D  MainActivity resumed
2026-04-23 09:32:22.738 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=9
2026-04-23 09:32:22.738 14251-14251 MainActivity            com.scmessenger.android              D  All permissions already granted
2026-04-23 09:32:22.739 14251-14280 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:32:22.740 14251-14280 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:32:22.741 14251-14334 BleGattServer           com.scmessenger.android              W  GATT server already running
2026-04-23 09:32:22.742 14251-14352 BleScanner              com.scmessenger.android              D  BLE scan already in progress, reusing existing session
2026-04-23 09:32:22.742 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=10
2026-04-23 09:32:22.742 14251-14334 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:32:22.743 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9
2026-04-23 09:32:22.749 14251-14334 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:32:22.751 14251-14333 WifiTransportManager    com.scmessenger.android              D  WifiTransportManager initialized
2026-04-23 09:32:22.755 14251-14334 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:32:22.755 14251-14333 WifiTransportManager    com.scmessenger.android              D  WiFi P2P discovery already active; skipping duplicate start
2026-04-23 09:32:22.756 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=9
2026-04-23 09:32:22.759 14251-14280 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:32:22.760 14251-14280 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:32:22.762 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=10
2026-04-23 09:32:22.763 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:32:22.766 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=9
2026-04-23 09:32:22.767 14251-14280 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:32:22.768 14251-14280 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:32:22.769 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=10
2026-04-23 09:32:22.771 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:32:22.774 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=9
2026-04-23 09:32:22.775 14251-14280 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:32:22.776 14251-14280 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:32:22.778 14251-14280 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=10
2026-04-23 09:32:22.779 14251-14280 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=12), next attempt in 60000ms
2026-04-23 09:32:22.829 14251-14251 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:32:22.829 14251-14251 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:aa79ca7c: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:32:23.798 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:32:26.873 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:32:27.723 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:32:27.725 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1171))
2026-04-23 09:32:27.779 14251-14280 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:32:27.783 14251-14280 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:32:29.951 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:32:32.728 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:32:32.733 14251-14334 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1176))
2026-04-23 09:32:33.034 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:32:33.892 14251-14281 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:32:33.893 14251-14281 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:32:33.894 14251-14281 MeshRepository          com.scmessenger.android              I  ✓ Internet transport (Swarm) initiated and bridge wired
2026-04-23 09:32:36.112 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:32:37.736 14251-14334 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:32:37.741 14251-14334 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1181))
2026-04-23 09:32:38.932 14251-14334 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:32:38.937 14251-14334 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:32:39.204 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:32:42.275 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:32:42.744 14251-14334 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:32:42.747 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1186))
2026-04-23 09:32:47.763 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:32:47.786 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1191))
2026-04-23 09:32:52.715 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:32:52.716 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:32:52.717 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:32:52.792 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:32:52.802 14251-14334 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1196))
2026-04-23 09:32:54.668 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:32:57.769 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:32:57.806 14251-14334 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:32:57.814 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1201))
2026-04-23 09:33:00.852 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:33:02.819 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:33:02.823 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1206))
2026-04-23 09:33:04.018 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:33:07.838 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:33:07.851 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1211))
2026-04-23 09:33:10.209 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:33:12.868 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:33:12.888 14251-14334 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1216))
2026-04-23 09:33:17.899 14251-14334 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:33:17.921 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1221))
2026-04-23 09:33:22.550 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:33:22.729 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:33:22.729 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:33:22.729 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:33:22.927 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:33:22.938 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1226))
2026-04-23 09:33:25.628 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:33:26.816 14251-14280 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:33:26.821 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:33:26.826 14251-14280 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:33:26.830 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:33:26.834 14251-14280 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:33:26.838 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:33:26.842 14251-14280 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:33:26.845 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:33:26.848 14251-14280 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:33:26.851 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:33:26.854 14251-14280 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:33:26.858 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:33:26.860 14251-14280 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:33:26.863 14251-14280 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=13), next attempt in 60000ms
2026-04-23 09:33:27.944 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:33:27.951 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1231))
2026-04-23 09:33:28.725 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:33:31.822 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:33:32.955 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:33:32.961 14251-14334 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1236))
2026-04-23 09:33:37.970 14251-14334 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:33:37.991 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:33:37.991 14251-14334 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1241))
2026-04-23 09:33:41.114 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:33:43.007 14251-14334 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:33:43.029 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1247))
2026-04-23 09:33:46.788 14251-14251 MainActivity            com.scmessenger.android              D  MainActivity paused
2026-04-23 09:33:46.789 14251-14251 VRI[MainActivity]       com.scmessenger.android              D  visibilityChanged oldVisibility=true newVisibility=false
2026-04-23 09:33:47.267 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:33:48.042 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:33:48.055 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1252))
2026-04-23 09:33:50.379 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:33:52.734 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:33:52.734 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:33:52.736 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:33:53.060 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:33:53.073 14251-14334 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1257))
2026-04-23 09:33:56.543 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:33:58.084 14251-14334 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:33:58.101 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1262))
2026-04-23 09:34:03.116 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:34:03.136 14251-14334 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1267))
2026-04-23 09:36:56.146 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:36:56.146 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:36:56.149 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:36:56.149 14251-14280 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:36:56.153 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:36:56.164 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:36:56.175 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:36:56.177 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1440))
2026-04-23 09:36:56.177 14251-14280 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:36:56.178 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:36:56.179 14251-14280 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:36:56.180 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:36:56.180 14251-14280 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:36:56.181 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:36:56.181 14251-14280 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:36:56.182 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:36:56.183 14251-14280 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:36:56.183 14251-14280 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:36:56.184 14251-14280 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:36:56.185 14251-14280 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=14), next attempt in 60000ms
2026-04-23 09:36:56.186 14251-14251 MainActivity            com.scmessenger.android              D  MainActivity resumed
2026-04-23 09:36:56.188 14251-14251 MainActivity            com.scmessenger.android              D  All permissions already granted
2026-04-23 09:36:56.190 14251-14280 BleGattServer           com.scmessenger.android              W  GATT server already running
2026-04-23 09:36:56.190 14251-14281 BleScanner              com.scmessenger.android              D  BLE scan already in progress, reusing existing session
2026-04-23 09:36:56.191 14251-14280 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:36:56.197 14251-14352 WifiTransportManager    com.scmessenger.android              D  WifiTransportManager initialized
2026-04-23 09:36:56.198 14251-14352 WifiTransportManager    com.scmessenger.android              D  WiFi P2P discovery already active; skipping duplicate start
2026-04-23 09:36:56.201 14251-14280 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:36:56.204 14251-14280 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:36:56.270 14251-14251 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:36:56.270 14251-14251 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:8dee77c: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:36:59.011 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:37:01.183 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:37:01.190 14251-14281 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1445))
2026-04-23 09:37:01.268 14251-14281 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:37:01.271 14251-14281 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:37:02.123 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:37:06.195 14251-14281 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:37:06.201 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1450))
2026-04-23 09:37:06.420 14251-14333 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:37:06.420 14251-14333 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:37:06.421 14251-14333 MeshRepository          com.scmessenger.android              I  ✓ Internet transport (Swarm) initiated and bridge wired
2026-04-23 09:37:08.313 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:37:11.214 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:37:11.235 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1455))
2026-04-23 09:37:11.398 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:37:11.528 14251-14333 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:37:11.533 14251-14333 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:37:14.502 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:37:16.240 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:37:16.249 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1460))
2026-04-23 09:37:20.704 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:37:21.252 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:37:21.259 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1465))
2026-04-23 09:37:26.177 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:37:26.177 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:37:26.178 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:37:26.264 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:37:26.272 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1470))
2026-04-23 09:37:31.284 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:37:31.298 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1475))
2026-04-23 09:37:33.084 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:37:36.303 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:37:36.317 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1480))
2026-04-23 09:37:41.333 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:37:41.357 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1485))
2026-04-23 09:37:41.371 14251-14333 FileLoggingTree         com.scmessenger.android              D  Truncating log file: /data/user/0/com.scmessenger.android/files/mesh_diagnostics.log
2026-04-23 09:37:46.372 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:37:46.398 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1490))
2026-04-23 09:37:48.552 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:37:51.412 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:37:51.434 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1495))
2026-04-23 09:37:51.678 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:37:54.786 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:37:56.184 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:37:56.184 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:37:56.186 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:37:56.445 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:37:56.465 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1500))
2026-04-23 09:37:57.872 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:38:00.238 14251-14333 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:38:00.248 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:38:00.260 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=11
2026-04-23 09:38:00.266 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:38:00.270 14251-14333 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:38:00.276 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=12
2026-04-23 09:38:00.282 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:38:00.290 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=11
2026-04-23 09:38:00.293 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:38:00.296 14251-14333 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:38:00.300 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=12
2026-04-23 09:38:00.302 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9
2026-04-23 09:38:00.307 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=11
2026-04-23 09:38:00.310 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:38:00.311 14251-14333 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:38:00.313 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=12
2026-04-23 09:38:00.315 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9
2026-04-23 09:38:00.318 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=11
2026-04-23 09:38:00.320 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:38:00.321 14251-14333 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:38:00.323 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=12
2026-04-23 09:38:00.324 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:38:00.327 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=11
2026-04-23 09:38:00.328 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:38:00.329 14251-14333 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:38:00.331 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=12
2026-04-23 09:38:00.332 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:38:00.335 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=11
2026-04-23 09:38:00.336 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:38:00.337 14251-14333 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:38:00.338 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=12
2026-04-23 09:38:00.339 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=15), next attempt in 60000ms
2026-04-23 09:38:00.979 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:38:01.469 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:38:01.474 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1505))
2026-04-23 09:38:06.481 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:38:06.487 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1510))
2026-04-23 09:38:11.496 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:38:11.502 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1515))
2026-04-23 09:38:16.508 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:38:16.515 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1520))
2026-04-23 09:38:19.599 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:38:21.516 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:38:21.521 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1525))
2026-04-23 09:38:25.788 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:38:26.206 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:38:26.206 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:38:26.208 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:38:26.539 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:38:26.564 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1530))
2026-04-23 09:38:28.903 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:38:31.577 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:38:31.599 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1535))
2026-04-23 09:38:36.610 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:38:36.630 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1540))
2026-04-23 09:38:41.644 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:38:41.667 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1545))
2026-04-23 09:38:46.680 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:38:46.694 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1550))
2026-04-23 09:38:51.700 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:38:51.709 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1555))
2026-04-23 09:38:56.227 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:38:56.228 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:38:56.230 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:38:56.713 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:38:56.718 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1560))
2026-04-23 09:38:56.879 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:39:01.726 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:39:01.746 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1565))
2026-04-23 09:39:04.396 14251-14352 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:39:04.416 14251-14352 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:39:04.430 14251-14352 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:39:04.444 14251-14352 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:39:04.454 14251-14352 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:39:04.460 14251-14352 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:39:04.467 14251-14352 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:39:04.473 14251-14352 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:39:04.478 14251-14352 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:39:04.483 14251-14352 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:39:04.488 14251-14352 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:39:04.492 14251-14352 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:39:04.495 14251-14352 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:39:04.499 14251-14352 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=16), next attempt in 60000ms
2026-04-23 09:39:06.755 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:39:06.764 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1570))
2026-04-23 09:39:11.781 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:39:11.807 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1575))
2026-04-23 09:39:12.484 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:39:16.823 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:39:16.846 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1580))
2026-04-23 09:39:21.859 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:39:21.870 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1585))
2026-04-23 09:39:26.247 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:39:26.247 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:39:26.248 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:39:26.883 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:39:26.899 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1590))
2026-04-23 09:39:31.064 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:39:31.906 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:39:31.915 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1595))
2026-04-23 09:39:34.184 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:39:36.923 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:39:36.935 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1600))
2026-04-23 09:39:41.953 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:39:41.977 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1605))
2026-04-23 09:39:46.992 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:39:47.015 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1610))
2026-04-23 09:39:49.742 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:39:52.030 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:39:52.055 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1616))
2026-04-23 09:39:56.256 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:39:56.256 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:39:56.258 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:39:57.067 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:39:57.083 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1621))
2026-04-23 09:39:59.062 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:40:02.099 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:40:02.122 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1626))
2026-04-23 09:40:07.120 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:40:07.122 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1631))
2026-04-23 09:40:07.463 14251-14251 MainActivity            com.scmessenger.android              D  MainActivity paused
2026-04-23 09:40:07.466 14251-14251 VRI[MainActivity]       com.scmessenger.android              D  visibilityChanged oldVisibility=true newVisibility=false
2026-04-23 09:40:07.791 14251-14262 ssenger.android         com.scmessenger.android              W  Reducing the number of considered missed Gc histogram windows from 106 to 100
2026-04-23 09:40:08.310 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:40:08.557 14251-14352 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:40:08.573 14251-14352 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:40:08.576 14251-14352 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:40:08.578 14251-14352 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:40:08.580 14251-14352 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:40:08.583 14251-14352 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:40:08.585 14251-14352 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:40:08.589 14251-14352 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:40:08.592 14251-14352 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:40:08.595 14251-14352 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:40:08.599 14251-14352 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:40:08.602 14251-14352 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:40:08.607 14251-14352 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:40:08.611 14251-14352 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=17), next attempt in 60000ms
2026-04-23 09:40:11.410 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:40:12.127 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:40:12.133 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1636))
2026-04-23 09:40:17.140 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:40:17.147 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1641))
2026-04-23 09:40:17.567 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:40:22.157 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:40:22.169 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1646))
2026-04-23 09:40:23.773 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:40:26.275 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:40:26.276 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:40:26.276 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:40:26.857 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:40:27.182 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:40:27.201 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1651))
2026-04-23 09:40:29.930 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:40:32.203 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:40:32.207 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1656))
2026-04-23 09:40:36.082 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:40:37.222 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:40:37.236 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1661))
2026-04-23 09:40:42.227 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:40:42.236 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:40:42.243 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1666))
2026-04-23 09:40:45.343 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:40:47.257 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:40:47.272 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1671))
2026-04-23 09:40:52.287 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:40:52.308 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1676))
2026-04-23 09:40:54.661 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:40:56.284 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:40:56.284 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:40:56.286 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:40:57.323 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:40:57.347 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1681))
2026-04-23 09:41:02.362 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:41:02.387 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1686))
2026-04-23 09:41:07.401 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:41:07.426 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1691))
2026-04-23 09:41:12.442 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:41:12.464 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1696))
2026-04-23 09:41:12.659 14251-14352 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:41:12.673 14251-14352 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:41:12.680 14251-14352 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:41:12.688 14251-14352 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:41:12.695 14251-14352 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:41:12.702 14251-14352 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:41:12.707 14251-14352 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:41:12.711 14251-14352 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:41:12.715 14251-14352 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9, skipping
2026-04-23 09:41:12.719 14251-14352 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:41:12.724 14251-14352 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:41:12.730 14251-14352 CircuitBreaker          com.scmessenger.android              D  Circuit breaker OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:41:12.734 14251-14352 MeshRepository          com.scmessenger.android              D  Circuit breaker blocked /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw, skipping
2026-04-23 09:41:12.739 14251-14352 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=18), next attempt in 60000ms
2026-04-23 09:41:17.473 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:46:28.260 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:46:28.260 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:46:28.264 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:46:28.265 14251-14333 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:46:28.265 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:46:28.267 14251-14279 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=1701))
2026-04-23 09:46:28.291 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:46:28.300 14251-14251 MainActivity            com.scmessenger.android              D  MainActivity resumed
2026-04-23 09:46:28.301 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=13
2026-04-23 09:46:28.302 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:46:28.302 14251-14251 MainActivity            com.scmessenger.android              D  All permissions already granted
2026-04-23 09:46:28.302 14251-14333 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:46:28.303 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=14
2026-04-23 09:46:28.304 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:46:28.304 14251-14279 BleGattServer           com.scmessenger.android              W  GATT server already running
2026-04-23 09:46:28.304 14251-14353 BleScanner              com.scmessenger.android              D  BLE scan already in progress, reusing existing session
2026-04-23 09:46:28.305 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=13
2026-04-23 09:46:28.305 14251-14279 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:46:28.306 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:46:28.307 14251-14333 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:46:28.308 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=14
2026-04-23 09:46:28.309 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9
2026-04-23 09:46:28.310 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=13
2026-04-23 09:46:28.311 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:46:28.311 14251-14333 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:46:28.312 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=14
2026-04-23 09:46:28.312 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9
2026-04-23 09:46:28.313 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=13
2026-04-23 09:46:28.313 14251-14280 WifiTransportManager    com.scmessenger.android              D  WifiTransportManager initialized
2026-04-23 09:46:28.313 14251-14279 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:46:28.314 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:46:28.315 14251-14279 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:46:28.317 14251-14333 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:46:28.318 14251-14280 WifiTransportManager    com.scmessenger.android              D  WiFi P2P discovery already active; skipping duplicate start
2026-04-23 09:46:28.319 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=14
2026-04-23 09:46:28.321 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:46:28.322 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=13
2026-04-23 09:46:28.323 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:46:28.323 14251-14333 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:46:28.324 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=14
2026-04-23 09:46:28.324 14251-14333 CircuitBreaker          com.scmessenger.android              D  Circuit breaker HALF-OPEN for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw
2026-04-23 09:46:28.326 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=13
2026-04-23 09:46:28.327 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:46:28.329 14251-14333 CircuitBreaker          com.scmessenger.android              W  Circuit breaker RE-OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after failed probe: Unknown network error: NetworkException: Network error
2026-04-23 09:46:28.330 14251-14333 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=14
2026-04-23 09:46:28.331 14251-14333 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=19), next attempt in 60000ms
2026-04-23 09:46:28.472 14251-14251 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:46:28.472 14251-14251 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:5906c4e6: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:46:29.442 14251-14251 WindowOnBackDispatcher  com.scmessenger.android              W  OnBackInvokedCallback is not enabled for the application.
                                                                                                    Set 'android:enableOnBackInvokedCallback="true"' in the application manifest.
2026-04-23 09:46:33.270 14251-14333 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:46:33.277 14251-14279 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=2017))
2026-04-23 09:46:33.359 14251-14279 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:46:33.363 14251-14279 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:46:34.493 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:46:36.374 14251-14251 MainActivity            com.scmessenger.android              D  MainActivity paused
2026-04-23 09:46:36.378 14251-14251 VRI[MainActivity]       com.scmessenger.android              D  visibilityChanged oldVisibility=true newVisibility=false
2026-04-23 09:46:37.599 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:46:38.288 14251-14279 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:46:38.297 14251-14280 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=2022))
2026-04-23 09:46:43.310 14251-14280 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:46:43.321 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=2027))
2026-04-23 09:46:48.326 14251-14279 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:46:48.338 14251-14279 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=2032))
2026-04-23 09:46:51.595 14251-14352 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:46:51.603 14251-14352 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:46:51.609 14251-14352 MeshRepository          com.scmessenger.android              I  ✓ Internet transport (Swarm) initiated and bridge wired
2026-04-23 09:46:53.346 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:46:53.356 14251-14352 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=2037))
2026-04-23 09:46:56.726 14251-14352 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:46:56.735 14251-14352 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:46:58.290 14251-14251 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:46:58.291 14251-14251 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:46:58.292 14251-14251 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:46:58.366 14251-14352 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:46:58.376 14251-14279 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=2042))
2026-04-23 09:47:02.324 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:47:03.383 14251-14279 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:47:03.389 14251-14333 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=2047))
2026-04-23 09:47:05.406 14251-14344 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
---------------------------- PROCESS ENDED (14251) for package com.scmessenger.android ----------------------------
---------------------------- PROCESS STARTED (25552) for package com.scmessenger.android ----------------------------
2026-04-23 09:50:41.534 25552-25552 nativeloader            com.scmessenger.android              D  Load libframework-connectivity-tiramisu-jni.so using APEX ns com_android_tethering for caller /apex/com.android.tethering/javalib/framework-connectivity-t.jar: ok
2026-04-23 09:50:41.551 25552-25552 nativeloader            com.scmessenger.android              D  Load /data/user/0/com.scmessenger.android/code_cache/startup_agents/be2db1e1-agent.so using system ns (caller=<unknown>): ok
2026-04-23 09:50:41.558 25552-25552 ssenger.android         com.scmessenger.android              W  hiddenapi: DexFile /data/data/com.scmessenger.android/code_cache/.studio/instruments-0c0ed4d1.jar is in boot class path but is not in a known location
2026-04-23 09:50:41.614 25552-25552 ssenger.android         com.scmessenger.android              W  Redefining intrinsic method java.lang.Thread java.lang.Thread.currentThread(). This may cause the unexpected use of the original definition of java.lang.Thread java.lang.Thread.currentThread()in methods that have already been compiled.
2026-04-23 09:50:41.614 25552-25552 ssenger.android         com.scmessenger.android              W  Redefining intrinsic method boolean java.lang.Thread.interrupted(). This may cause the unexpected use of the original definition of boolean java.lang.Thread.interrupted()in methods that have already been compiled.
2026-04-23 09:50:41.968 25552-25552 nativeloader            com.scmessenger.android              D  Configuring clns-9 for other apk /data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk. target_sdk_version=35, uses_libraries=, library_path=/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/lib/arm64:/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk!/lib/arm64-v8a, permitted_path=/data:/mnt/expand:/data/user/0/com.scmessenger.android
2026-04-23 09:50:41.969 25552-25552 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 202956589; UID 10499; state: ENABLED
2026-04-23 09:50:41.977 25552-25552 GraphicsEnvironment     com.scmessenger.android              V  Currently set values for:
2026-04-23 09:50:41.977 25552-25552 GraphicsEnvironment     com.scmessenger.android              V    angle_gl_driver_selection_pkgs=[com.android.angle]
2026-04-23 09:50:41.977 25552-25552 GraphicsEnvironment     com.scmessenger.android              V    angle_gl_driver_selection_values=[angle]
2026-04-23 09:50:41.977 25552-25552 GraphicsEnvironment     com.scmessenger.android              V  com.scmessenger.android is not listed in per-application setting
2026-04-23 09:50:41.977 25552-25552 GraphicsEnvironment     com.scmessenger.android              V  No special selections for ANGLE, returning default driver choice
2026-04-23 09:50:41.977 25552-25552 GraphicsEnvironment     com.scmessenger.android              V  Neither updatable production driver nor prerelease driver is supported.
2026-04-23 09:50:41.999 25552-25552 WM-WrkMgrInitializer    com.scmessenger.android              D  Initializing WorkManager with default configuration.
2026-04-23 09:50:42.025 25552-25552 MeshApplication         com.scmessenger.android              I  SCMessenger application started
2026-04-23 09:50:42.027 25552-25569 StorageManager          com.scmessenger.android              D  Performing startup storage maintenance...
2026-04-23 09:50:42.031 25552-25569 StorageManager          com.scmessenger.android              D  Logs rotated successfully.
2026-04-23 09:50:42.033 25552-25569 StorageManager          com.scmessenger.android              D  Startup maintenance complete. Available storage: 17710 MB
2026-04-23 09:50:42.035 25552-25569 MeshApplic...n$onCreate com.scmessenger.android              I  Startup storage maintenance completed
2026-04-23 09:50:42.047 25552-25552 BootReceiver            com.scmessenger.android              I  Boot completed, checking auto-start preference
2026-04-23 09:50:42.049 25552-25573 DisplayManager          com.scmessenger.android              I  Choreographer implicitly registered for the refresh rate.
2026-04-23 09:50:42.050 25552-25573 vulkan                  com.scmessenger.android              D  searching for layers in '/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/lib/arm64'
2026-04-23 09:50:42.050 25552-25573 vulkan                  com.scmessenger.android              D  searching for layers in '/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk!/lib/arm64-v8a'
2026-04-23 09:50:42.057 25552-25552 DesktopExperienceFlags  com.scmessenger.android              D  Toggle override initialized to: false
2026-04-23 09:50:42.077 25552-25552 MeshRepository          com.scmessenger.android              D  MeshRepository initialized with storage: /data/user/0/com.scmessenger.android/files
2026-04-23 09:50:42.082 25552-25552 MeshRepository          com.scmessenger.android              D  AND-CONTACTS-WIPE-001: Reinstall check - hasIdentityBackup=true, contactsOnDisk=true, historyOnDisk=true
2026-04-23 09:50:42.083 25552-25552 MeshRepository          com.scmessenger.android              D  AND-CONTACTS-WIPE-001: Normal startup - all data present
2026-04-23 09:50:42.086 25552-25552 MeshRepository          com.scmessenger.android              D  Contacts migration already completed, skipping
2026-04-23 09:50:42.091 25552-25552 nativeloader            com.scmessenger.android              D  Load /data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/lib/arm64/libjnidispatch.so using class loader ns clns-9 (caller=/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk!classes17.dex): ok
2026-04-23 09:50:42.254 25552-25552 MeshRepository          com.scmessenger.android              I  all_managers_init_success
2026-04-23 09:50:42.255 25552-25552 MeshRepository          com.scmessenger.android              I  All managers initialized successfully
2026-04-23 09:50:42.257 25552-25552 MeshRepository          com.scmessenger.android              D  AND-CONTACTS-WIPE-001: Contact data verification - Found 0 contacts
2026-04-23 09:50:42.258 25552-25552 MeshRepository          com.scmessenger.android              W  AND-CONTACTS-WIPE-001: WARNING - No contacts found. If this is unexpected, data recovery may be needed.
2026-04-23 09:50:42.259 25552-25552 MainActivity            com.scmessenger.android              D  MainActivity created
2026-04-23 09:50:42.260 25552-25552 AnrWatchdog             com.scmessenger.android              I  ANR watchdog started (check=5000ms, threshold=10000ms)
2026-04-23 09:50:42.261 25552-25552 MainActivity            com.scmessenger.android              I  ANR watchdog started for UI thread monitoring
2026-04-23 09:50:42.263 25552-25569 MeshRepository          com.scmessenger.android              D  Corruption check: contacts=0, messages=0
2026-04-23 09:50:42.263 25552-25569 MeshRepository          com.scmessenger.android              D  Database integrity check passed
2026-04-23 09:50:42.263 25552-25552 MainActivity            com.scmessenger.android              D  All permissions already granted
2026-04-23 09:50:42.269 25552-25552 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 309578419; UID 10499; state: ENABLED
2026-04-23 09:50:42.269 25552-25552 DesktopModeFlags        com.scmessenger.android              D  Toggle override initialized to: OVERRIDE_UNSET
2026-04-23 09:50:42.281 25552-25569 MeshRepository          com.scmessenger.android              I  Repository background initialization completed
2026-04-23 09:50:42.281 25552-25569 MainActivity            com.scmessenger.android              D  UI components initialization completed
2026-04-23 09:50:42.282 25552-25571 MeshReposi...aintenance com.scmessenger.android              D  Storage maintenance check: free=17710MB / total=112912MB
2026-04-23 09:50:42.282 25552-25552 ContentCaptureHelper    com.scmessenger.android              I  Setting logging level to OFF
2026-04-23 09:50:42.287 25552-25552 MainActivity            com.scmessenger.android              D  MainActivity resumed
2026-04-23 09:50:42.288 25552-25552 MainActivity            com.scmessenger.android              D  All permissions already granted
2026-04-23 09:50:42.288 25552-25552 MeshRepository          com.scmessenger.android              D  Permission refresh skipped: mesh service is not running
2026-04-23 09:50:42.292 25552-25552 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 349153669; UID 10499; state: ENABLED
2026-04-23 09:50:42.394 25552-25552 MainViewModel           com.scmessenger.android              D  MainViewModel init
2026-04-23 09:50:42.395 25552-25571 MainViewMo...rageStatus com.scmessenger.android              D  Storage refreshed: 17710 MB available (Low=false)
2026-04-23 09:50:42.395 25552-25552 MainViewModel           com.scmessenger.android              D  Preference onboardingCompleted: true
2026-04-23 09:50:42.396 25552-25552 MainViewModel           com.scmessenger.android              D  Preference installChoiceCompleted: true
2026-04-23 09:50:42.396 25552-25552 MainViewModel           com.scmessenger.android              D  MeshRepository service state: STOPPED
2026-04-23 09:50:42.397 25552-25571 MainViewMo...ntityState com.scmessenger.android              D  refreshIdentityState() called
2026-04-23 09:50:42.397 25552-25571 MainViewMo...ntityState com.scmessenger.android              D  Identity initialized state: true
2026-04-23 09:50:42.414 25552-25558 ssenger.android         com.scmessenger.android              I  Compiler allocated 5049KB to compile void android.view.ViewRootImpl.performTraversals()
2026-04-23 09:50:42.416 25552-25552 VRI[MainActivity]       com.scmessenger.android              D  WindowInsets changed: 1080x2400 statusBars:[0,132,0,0] navigationBars:[0,0,0,63] mandatorySystemGestures:[0,164,0,84] 
2026-04-23 09:50:42.418 25552-25552 WindowOnBackDispatcher  com.scmessenger.android              D  setTopOnBackInvokedCallback (unwrapped): android.view.ViewRootImpl$$ExternalSyntheticLambda25@66e15db
2026-04-23 09:50:42.460 25552-25552 HWUI                    com.scmessenger.android              I  Using FreeType backend (prop=Auto)
2026-04-23 09:50:42.495 25552-25552 MeshServiceViewModel    com.scmessenger.android              D  MeshServiceViewModel initialized
2026-04-23 09:50:42.502 25552-25571 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 09:50:42.502 25552-25571 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=null, initialized=null, nickname=null
2026-04-23 09:50:42.503 25552-25594 MeshReposi...edDeferred com.scmessenger.android              D  Lazy starting MeshService (async settings reload)...
2026-04-23 09:50:42.503 25552-25569 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 09:50:42.503 25552-25594 MeshRepository          com.scmessenger.android              I  service_start_requested
2026-04-23 09:50:42.504 25552-25569 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=null, initialized=null, nickname=null
2026-04-23 09:50:42.504 25552-25594 MeshRepository          com.scmessenger.android              D  Starting MeshService...
2026-04-23 09:50:42.504 25552-25569 DashboardViewModel      com.scmessenger.android              D  Topology built: 0 nodes, 0 edges
2026-04-23 09:50:42.504 25552-25571 DashboardViewModel      com.scmessenger.android              D  Topology built: 0 nodes, 0 edges
2026-04-23 09:50:42.504 25552-25570 MeshReposi...edDeferred com.scmessenger.android              D  MeshService is already starting, skipping redundant init
2026-04-23 09:50:42.505 25552-25571 DashboardV...efreshData com.scmessenger.android              D  Dashboard data refreshed
2026-04-23 09:50:42.505 25552-25569 DashboardV...efreshData com.scmessenger.android              D  Dashboard data refreshed
2026-04-23 09:50:42.507 25552-25552 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:197405): avc:  denied  { search } for  name="/" dev="cgroup2" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup_v2:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:50:42.507 25552-25552 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:197406): avc:  denied  { search } for  name="/" dev="cgroup2" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup_v2:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:50:42.507 25552-25552 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:197407): avc:  denied  { search } for  name="/" dev="cgroup" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:50:42.507 25552-25552 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:197408): avc:  denied  { search } for  name="/" dev="cgroup" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:50:42.556 25552-25594 MeshRepository          com.scmessenger.android              I  SmartTransportRouter initialized for intelligent transport selection
2026-04-23 09:50:42.563 25552-25594 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:50:42.563 25552-25594 NetworkDetector         com.scmessenger.android              I  NetworkDetector monitoring started
2026-04-23 09:50:42.564 25552-25594 MeshRepository          com.scmessenger.android              I  NetworkDetector started — cellular-aware transport fallback active
2026-04-23 09:50:42.564 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:50:42.569 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:50:42.571 25552-25606 NetworkDetector         com.scmessenger.android              W  Cellular network detected (CELLULAR) — blocking ports: [9001, 9010, 4001, 5001]
2026-04-23 09:50:42.571 25552-25571 MeshReposi...hangeWatch com.scmessenger.android              I  Network type changed: WIFI → CELLULAR, resetting circuit breakers and re-bootstrapping
2026-04-23 09:50:42.573 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: CELLULAR, blocked ports: [9001, 9010, 4001, 5001]
2026-04-23 09:50:42.574 25552-25571 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:50:42.576 25552-25606 NetworkDetector         com.scmessenger.android              W  Cellular network detected (CELLULAR) — blocking ports: [9001, 9010, 4001, 5001]
2026-04-23 09:50:42.578 25552-25571 MeshRepository          com.scmessenger.android              I  Racing bootstrap: network=CELLULAR, transports=wss→tcp→quic→tcp→ws
2026-04-23 09:50:42.579 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: CELLULAR, blocked ports: [9001, 9010, 4001, 5001]
2026-04-23 09:50:42.580 25552-25571 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:50:42.584 25552-25571 MeshReposi...urcesAsync com.scmessenger.android              I  Using bootstrap source: StaticFallback with 4 nodes
2026-04-23 09:50:42.725 25552-25552 BootReceiver$onReceive  com.scmessenger.android              D  Auto-start disabled, not starting service
2026-04-23 09:50:42.730 25552-25609 MainViewMo...ntityState com.scmessenger.android              D  refreshIdentityState() called
2026-04-23 09:50:42.744 25552-25609 MainViewMo...ntityState com.scmessenger.android              D  Identity initialized state: true
2026-04-23 09:50:42.868 25552-25552 Conversati...adMessages com.scmessenger.android              D  Loaded 0 messages
2026-04-23 09:50:42.870 25552-25552 Conversati...$loadStats com.scmessenger.android              D  Loaded stats: HistoryStats(totalMessages=0, sentCount=0, receivedCount=0, undeliveredCount=0)
2026-04-23 09:50:42.872 25552-25552 Conversati...ockedPeers com.scmessenger.android              D  Loaded 0 blocked peers
2026-04-23 09:50:44.090 25552-25569 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:9001 = blocked
2026-04-23 09:50:44.090 25552-25596 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:9010 = blocked
2026-04-23 09:50:44.091 25552-25595 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:443 = blocked
2026-04-23 09:50:44.091 25552-25570 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:443 = blocked
2026-04-23 09:50:44.117 25552-25570 MeshRepository          com.scmessenger.android              W  All bootstrap addresses failed or timed out, falling back to mDNS
2026-04-23 09:50:44.123 25552-25570 MeshRepository          com.scmessenger.android              I  All relay bootstrap failed, attempting mDNS local discovery
2026-04-23 09:50:47.115 25552-25620 ProfileInstaller        com.scmessenger.android              D  Installing profile for com.scmessenger.android
2026-04-23 09:50:47.641 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:50:49.134 25552-25570 MeshRepository          com.scmessenger.android              E  mDNS fallback: no LAN peers discovered within timeout
2026-04-23 09:50:49.144 25552-25570 MeshReposi...hangeWatch com.scmessenger.android              I  Network type changed: CELLULAR → WIFI, resetting circuit breakers and re-bootstrapping
2026-04-23 09:50:49.150 25552-25570 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:50:49.157 25552-25570 MeshRepository          com.scmessenger.android              I  Racing bootstrap: network=WIFI, transports=quic→tcp→wss→ws
2026-04-23 09:50:49.162 25552-25570 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:50:49.168 25552-25570 MeshReposi...urcesAsync com.scmessenger.android              I  Using bootstrap source: StaticFallback with 4 nodes
2026-04-23 09:50:50.680 25552-25609 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:443 = blocked
2026-04-23 09:50:50.680 25552-25569 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:9001 = blocked
2026-04-23 09:50:50.680 25552-25596 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:443 = blocked
2026-04-23 09:50:50.680 25552-25608 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:9010 = blocked
2026-04-23 09:50:50.730 25552-25569 MeshRepository          com.scmessenger.android              W  All bootstrap addresses failed or timed out, falling back to mDNS
2026-04-23 09:50:50.735 25552-25569 MeshRepository          com.scmessenger.android              I  All relay bootstrap failed, attempting mDNS local discovery
2026-04-23 09:50:53.078 25552-25552 MainViewModel           com.scmessenger.android              D  MeshRepository service state: RUNNING
2026-04-23 09:50:53.078 25552-25594 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:50:53.081 25552-25596 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=0))
2026-04-23 09:50:53.082 25552-25594 MeshRepository          com.scmessenger.android              I  SC_IDENTITY_OWN p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9 pk=374d73ebc21d677f889b806d6065a7177a2c5748f368036bd073e949bba5a2d4
2026-04-23 09:50:53.086 25552-25595 MainViewMo...ntityState com.scmessenger.android              D  refreshIdentityState() called
2026-04-23 09:50:53.088 25552-25594 MeshRepository          com.scmessenger.android              I  Mesh service started successfully
2026-04-23 09:50:53.092 25552-25594 MeshReposi...edDeferred com.scmessenger.android              D  MeshService started lazily (async settings reload)
2026-04-23 09:50:53.094 25552-25595 MainViewMo...ntityState com.scmessenger.android              D  Identity initialized state: true
2026-04-23 09:50:53.095 25552-25559 ssenger.android         com.scmessenger.android              W  userfaultfd: MOVE ioctl seems unsupported: Connection timed out
2026-04-23 09:50:53.098 25552-25596 MeshReposi...edDeferred com.scmessenger.android              D  Settings reloaded asynchronously after service startup
2026-04-23 09:50:53.119 25552-25595 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:50:53.119 25552-25595 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:50:53.120 25552-25595 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:50:53.120 25552-25569 AdvertiseSettings       com.scmessenger.android              D  setTxPowerLevel: 2
2026-04-23 09:50:53.120 25552-25569 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:50:53.120 25552-25569 BluetoothLeAdvertiser   com.scmessenger.android              D  TxPower == ADVERTISE_TX_POWER_MEDIUM
2026-04-23 09:50:53.121 25552-25569 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:50:53.126 25552-25595 BackoffStrategy         com.scmessenger.android              D  Backoff strategy reset
2026-04-23 09:50:53.126 25552-25569 BluetoothGattServer     com.scmessenger.android              D  registerCallback()
2026-04-23 09:50:53.126 25552-25569 BluetoothGattServer     com.scmessenger.android              D  registerCallback() - UUID=ccdb29e3-9276-4a4e-8005-9180273c7140
2026-04-23 09:50:53.132 25552-25595 BleScanner              com.scmessenger.android              I  BLE Scanning started (background=false, fallback=false)
2026-04-23 09:50:53.136 25552-25615 BluetoothGattServer     com.scmessenger.android              D  onServerRegistered(0)
2026-04-23 09:50:53.136 25552-25615 BluetoothLeScanner      com.scmessenger.android              D  onScannerRegistered(status=0, scannerId=3): mScannerId=0
2026-04-23 09:50:53.136 25552-25569 BluetoothGattServer     com.scmessenger.android              D  addService() - service: 0000df01-0000-1000-8000-00805f9b34fb
2026-04-23 09:50:53.137 25552-25569 BleGattServer           com.scmessenger.android              I  GATT server started with SCMessenger service
2026-04-23 09:50:53.138 25552-25595 BleScanner              com.scmessenger.android              D  Duty cycle started: 10000ms scan / 30000ms interval
2026-04-23 09:50:53.139 25552-25569 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (2 bytes)
2026-04-23 09:50:53.142 25552-25615 BluetoothGattServer     com.scmessenger.android              D  onServiceAdded() - handle=147 uuid=0000df01-0000-1000-8000-00805f9b34fb status=0
2026-04-23 09:50:53.144 25552-25569 BleAdvertiser           com.scmessenger.android              D  Identity data set: 313 bytes
2026-04-23 09:50:53.146 25552-25569 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:50:53.148 25552-25569 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:50:53.159 25552-25552 BleAdverti...seCallback com.scmessenger.android              I  BLE Advertising started successfully
2026-04-23 09:50:53.161 25552-25569 WifiTransportManager    com.scmessenger.android              D  WifiTransportManager initialized
2026-04-23 09:50:53.169 25552-25552 WifiDirect...2pReceiver com.scmessenger.android              D  WiFi P2P state changed: enabled=true
2026-04-23 09:50:53.171 25552-25569 WifiDirectTransport     com.scmessenger.android              I  WiFi Direct started
2026-04-23 09:50:53.365 25552-25552 WifiDirect...2pReceiver com.scmessenger.android              D  Disconnected from WiFi P2P group
2026-04-23 09:50:53.374 25552-25552 WifiDirect...2pReceiver com.scmessenger.android              D  This device changed
2026-04-23 09:50:53.380 25552-25552 WifiP2pManager          com.scmessenger.android              D  Ignored { when=0 what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:50:53.380 25552-25552 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-1ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:50:53.380 25552-25552 WifiDirect...eDiscovery com.scmessenger.android              D  Service discovery request added
2026-04-23 09:50:53.382 25552-25552 WifiP2pManager          com.scmessenger.android              D  Ignored { when=0 what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:50:53.382 25552-25552 WifiP2pManager          com.scmessenger.android              D  Ignored { when=0 what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:50:53.383 25552-25552 WifiDirect...terService com.scmessenger.android              D  WiFi Direct service registered: scmessenger
2026-04-23 09:50:53.390 25552-25552 WifiTransp...tDiscovery com.scmessenger.android              I  WiFi P2P Discovery started
2026-04-23 09:50:53.427 25552-25552 WifiDirect...rDiscovery com.scmessenger.android              D  Peer discovery started
2026-04-23 09:50:53.806 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:50:55.745 25552-25596 MeshRepository          com.scmessenger.android              E  mDNS fallback: no LAN peers discovered within timeout
2026-04-23 09:50:58.091 25552-25596 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:50:58.099 25552-25596 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=5))
2026-04-23 09:50:58.175 25552-25596 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:50:58.181 25552-25596 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:51:00.022 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:51:03.109 25552-25596 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:51:03.120 25552-25609 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=10))
2026-04-23 09:51:03.141 25552-25552 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:51:03.148 25552-25552 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:51:04.318 25552-25569 MeshReposi...strapNodes com.scmessenger.android              D  Pre-populating static bootstrap nodes (no network I/O)
2026-04-23 09:51:04.329 25552-25569 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:51:04.330 25552-25569 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:51:04.331 25552-25569 MeshRepository          com.scmessenger.android              I  ✓ Internet transport (Swarm) initiated and bridge wired
2026-04-23 09:51:04.333 25552-25569 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:51:04.336 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:51:04.337 25552-25569 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:51:04.339 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:51:04.341 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:51:04.341 25552-25569 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:51:04.343 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:51:04.345 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:51:04.345 25552-25569 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:51:04.347 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:51:04.348 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:51:04.349 25552-25569 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:51:04.350 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:51:04.352 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:51:04.353 25552-25569 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:51:04.354 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:51:04.356 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=1
2026-04-23 09:51:04.357 25552-25569 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:51:04.358 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=2
2026-04-23 09:51:04.359 25552-25569 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=1), next attempt in 10000ms
2026-04-23 09:51:06.220 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:51:08.128 25552-25594 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:51:08.135 25552-25594 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=15))
2026-04-23 09:51:09.345 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:51:09.414 25552-25594 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:51:09.420 25552-25594 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:51:12.434 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:51:13.140 25552-25594 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:51:13.140 25552-25552 BleScanner...d$Runnable com.scmessenger.android              W  BLE scan fallback enabled after 20020 ms without mesh advertisements; switching to unfiltered scan
2026-04-23 09:51:13.147 25552-25609 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=20))
2026-04-23 09:51:13.150 25552-25552 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:51:13.150 25552-25552 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:51:13.150 25552-25552 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:51:13.154 25552-25552 BleScanner              com.scmessenger.android              I  BLE scan restarted (background=false, fallback=true)
2026-04-23 09:51:13.155 25552-25565 BluetoothLeScanner      com.scmessenger.android              D  onScannerRegistered(status=0, scannerId=3): mScannerId=0
2026-04-23 09:51:18.166 25552-25609 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:51:18.177 25552-25595 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=25))
2026-04-23 09:51:20.375 25552-25595 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:51:20.432 25552-25595 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:51:20.443 25552-25595 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:51:20.458 25552-25595 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:51:20.474 25552-25595 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:51:20.478 25552-25595 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:51:20.484 25552-25595 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:51:20.493 25552-25595 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:51:20.496 25552-25595 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:51:20.500 25552-25595 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:51:20.506 25552-25595 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:51:20.509 25552-25595 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:51:20.512 25552-25595 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:51:20.516 25552-25595 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:51:20.518 25552-25595 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:51:20.522 25552-25595 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:51:20.527 25552-25595 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=3
2026-04-23 09:51:20.529 25552-25595 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:51:20.531 25552-25595 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=4
2026-04-23 09:51:20.533 25552-25595 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=2), next attempt in 30000ms
2026-04-23 09:51:21.746 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:51:23.181 25552-25595 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:51:23.185 25552-25595 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=30))
2026-04-23 09:51:24.829 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:51:28.193 25552-25595 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:51:28.197 25552-25595 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=35))
2026-04-23 09:51:33.155 25552-25552 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:51:33.161 25552-25552 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:51:33.203 25552-25595 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:51:33.210 25552-25595 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=40))
2026-04-23 09:51:34.136 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:51:37.236 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:51:38.222 25552-25595 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:51:38.236 25552-25609 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=45))
2026-04-23 09:51:40.342 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:51:43.258 25552-25609 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:51:43.290 25552-25594 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=50))
2026-04-23 09:51:43.774 25552-25552 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:51:43.774 25552-25552 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:282d2044: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:51:44.822 25552-25552 ContactsVi...adContacts com.scmessenger.android              D  Loaded 0 contacts, filtered nearby peers to 0
2026-04-23 09:51:44.838 25552-25552 WindowOnBackDispatcher  com.scmessenger.android              W  OnBackInvokedCallback is not enabled for the application.
                                                                                                    Set 'android:enableOnBackInvokedCallback="true"' in the application manifest.
2026-04-23 09:51:45.825 25552-25552 MeshServiceViewModel    com.scmessenger.android              D  MeshServiceViewModel initialized
2026-04-23 09:51:45.836 25552-25609 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 09:51:45.836 25552-25595 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 09:51:46.549 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:51:46.796 25552-25552 MeshServiceViewModel    com.scmessenger.android              D  MeshServiceViewModel initialized
2026-04-23 09:51:46.797 25552-25570 SettingsViewModel       com.scmessenger.android              D  Loaded mesh settings: MeshSettings(relayEnabled=true, maxRelayBudget=200, batteryFloor=20, bleEnabled=true, wifiAwareEnabled=true, wifiDirectEnabled=true, internetEnabled=true, discoveryMode=NORMAL, onionRouting=false, coverTrafficEnabled=false, messagePaddingEnabled=false, timingObfuscationEnabled=false, notificationsEnabled=true, notifyDmEnabled=true, notifyDmRequestEnabled=true, notifyDmInForeground=false, notifyDmRequestInForeground=true, soundEnabled=true, badgeEnabled=true)
2026-04-23 09:51:48.290 25552-25596 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:51:48.297 25552-25594 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=55))
2026-04-23 09:51:49.627 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:51:52.560 25552-25594 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:51:52.571 25552-25594 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:51:52.576 25552-25594 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:51:52.581 25552-25594 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:51:52.587 25552-25594 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:51:52.596 25552-25594 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:51:52.600 25552-25594 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:51:52.603 25552-25594 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:51:52.609 25552-25594 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:51:52.617 25552-25594 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:51:52.621 25552-25594 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:51:52.624 25552-25594 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:51:52.629 25552-25594 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:51:52.638 25552-25594 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:51:52.642 25552-25594 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:51:52.654 25552-25594 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:51:52.660 25552-25594 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:51:52.668 25552-25594 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:51:52.672 25552-25594 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:51:52.675 25552-25594 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:51:52.680 25552-25594 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:51:52.689 25552-25594 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=5
2026-04-23 09:51:52.693 25552-25594 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:51:52.696 25552-25594 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:51:52.701 25552-25594 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=6
2026-04-23 09:51:52.705 25552-25594 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=3), next attempt in 30000ms
2026-04-23 09:51:53.304 25552-25594 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:51:53.313 25552-25608 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=60))
2026-04-23 09:51:55.779 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:51:55.789 25552-25559 ssenger.android         com.scmessenger.android              I  Background young concurrent mark compact GC freed 21MB AllocSpace bytes, 6(256KB) LOS objects, 74% free, 7956KB/30MB, paused 832us,4.564ms total 122.817ms
2026-04-23 09:51:56.174 25552-25552 MeshServic...topService com.scmessenger.android              I  Mesh service stop requested
2026-04-23 09:51:56.183 25552-25552 MeshForegroundService   com.scmessenger.android              D  MeshForegroundService created
2026-04-23 09:51:56.199 25552-25608 MeshForegr...eshService com.scmessenger.android              I  Stopping mesh service
2026-04-23 09:51:56.205 25552-25608 NetworkDetector         com.scmessenger.android              I  NetworkDetector monitoring stopped
2026-04-23 09:51:56.212 25552-25608 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:51:56.212 25552-25608 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:51:56.212 25552-25608 BleScanner              com.scmessenger.android              I  BLE Scanning stopped
2026-04-23 09:51:56.218 25552-25608 BleAdvertiser           com.scmessenger.android              I  BLE Advertising stopped
2026-04-23 09:51:56.221 25552-25608 BluetoothGattServer     com.scmessenger.android              D  close()
2026-04-23 09:51:56.221 25552-25608 BluetoothGattServer     com.scmessenger.android              D  unregisterCallback()
2026-04-23 09:51:56.222 25552-25608 BleGattServer           com.scmessenger.android              I  GATT server stopped
2026-04-23 09:51:56.228 25552-25608 WifiDirectTransport     com.scmessenger.android              I  WiFi Direct stopped
2026-04-23 09:51:56.239 25552-25608 MeshRepository          com.scmessenger.android              I  Cleared all discovered peers on mesh service stop
2026-04-23 09:51:56.242 25552-25552 WifiTransp...pDiscovery com.scmessenger.android              I  WiFi P2P Discovery stopped
2026-04-23 09:51:56.248 25552-25608 MeshRepository          com.scmessenger.android              I  Mesh service stopped
2026-04-23 09:51:56.248 25552-25552 MainViewModel           com.scmessenger.android              D  MeshRepository service state: STOPPED
2026-04-23 09:51:56.249 25552-25552 ContactsVi...rviceState com.scmessenger.android              D  Cleared all nearby peers on service stop
2026-04-23 09:51:56.253 25552-25608 PerformanceMonitor      com.scmessenger.android              I  Service stopped - Total uptime: 0h 0m 0s, ANR events: 0
2026-04-23 09:51:56.259 25552-25608 AndroidPlatformBridge   com.scmessenger.android              D  AndroidPlatformBridge cleaning up
2026-04-23 09:51:56.290 25552-25552 MeshForegroundService   com.scmessenger.android              D  MeshForegroundService destroyed
2026-04-23 09:51:58.119 25552-25552 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=null, initialized=null, nickname=null
2026-04-23 09:51:58.119 25552-25608 MeshReposi...edDeferred com.scmessenger.android              D  Lazy starting MeshService (async settings reload)...
2026-04-23 09:51:58.124 25552-25608 MeshRepository          com.scmessenger.android              I  service_start_requested
2026-04-23 09:51:58.125 25552-25552 MeshServic...artService com.scmessenger.android              I  Mesh service start requested
2026-04-23 09:51:58.128 25552-25608 MeshRepository          com.scmessenger.android              D  Starting MeshService...
2026-04-23 09:51:58.133 25552-25552 MeshForegroundService   com.scmessenger.android              D  MeshForegroundService created
2026-04-23 09:51:58.144 25552-25698 MeshForegr...eshService com.scmessenger.android              I  Starting mesh service
2026-04-23 09:51:58.168 25552-25698 MeshForegroundService   com.scmessenger.android              D  WakeLock acquired for BLE scan windows
2026-04-23 09:51:58.169 25552-25698 AndroidPlatformBridge   com.scmessenger.android              D  AndroidPlatformBridge initializing
2026-04-23 09:51:58.172 25552-25552 AndroidPlatformBridge   com.scmessenger.android              D  Battery changed: 60%, charging=false
2026-04-23 09:51:58.175 25552-25698 AndroidPlatformBridge   com.scmessenger.android              D  Motion detection initialized (screen state proxy)
2026-04-23 09:51:58.177 25552-25552 AndroidPlatformBridge   com.scmessenger.android              D  Applying BLE adjustments: scan=1000ms, advertise=200ms, txPower=0dBm
2026-04-23 09:51:58.178 25552-25606 AndroidPlatformBridge   com.scmessenger.android              D  Network changed: wifi=true, cellular=false
2026-04-23 09:51:58.180 25552-25552 AndroidPlatformBridge   com.scmessenger.android              D  Applying relay adjustments: maxPerHour=500, priority=50, maxPayload=32768
2026-04-23 09:51:58.184 25552-25552 AndroidPlatformBridge   com.scmessenger.android              D  Adjustment profile: HIGH for battery 60%, charging=false
2026-04-23 09:51:58.193 25552-25608 MeshRepository          com.scmessenger.android              I  SmartTransportRouter initialized for intelligent transport selection
2026-04-23 09:51:58.194 25552-25606 AndroidPlatformBridge   com.scmessenger.android              D  Applying BLE adjustments: scan=1000ms, advertise=200ms, txPower=0dBm
2026-04-23 09:51:58.202 25552-25608 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:51:58.202 25552-25606 AndroidPlatformBridge   com.scmessenger.android              D  Applying relay adjustments: maxPerHour=500, priority=50, maxPayload=32768
2026-04-23 09:51:58.205 25552-25608 NetworkDetector         com.scmessenger.android              I  NetworkDetector monitoring started
2026-04-23 09:51:58.210 25552-25608 MeshRepository          com.scmessenger.android              I  NetworkDetector started — cellular-aware transport fallback active
2026-04-23 09:51:58.213 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:51:58.218 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:51:58.222 25552-25606 NetworkDetector         com.scmessenger.android              W  Cellular network detected (CELLULAR) — blocking ports: [9001, 9010, 4001, 5001]
2026-04-23 09:51:58.222 25552-25594 MeshReposi...hangeWatch com.scmessenger.android              I  Network type changed: WIFI → CELLULAR, resetting circuit breakers and re-bootstrapping
2026-04-23 09:51:58.225 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: CELLULAR, blocked ports: [9001, 9010, 4001, 5001]
2026-04-23 09:51:58.230 25552-25594 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:51:58.232 25552-25606 NetworkDetector         com.scmessenger.android              W  Cellular network detected (CELLULAR) — blocking ports: [9001, 9010, 4001, 5001]
2026-04-23 09:51:58.235 25552-25594 MeshRepository          com.scmessenger.android              I  Racing bootstrap: network=CELLULAR, transports=wss→tcp→quic→tcp→ws
2026-04-23 09:51:58.236 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: CELLULAR, blocked ports: [9001, 9010, 4001, 5001]
2026-04-23 09:51:58.238 25552-25594 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:51:58.243 25552-25594 MeshReposi...urcesAsync com.scmessenger.android              I  Using bootstrap source: StaticFallback with 4 nodes
2026-04-23 09:51:58.316 25552-25709 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:51:58.322 25552-25707 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=0))
2026-04-23 09:51:59.156 25552-25609 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:51:59.160 25552-25609 DashboardViewModel      com.scmessenger.android              D  Topology built: 1 nodes, 0 edges
2026-04-23 09:51:59.163 25552-25609 DashboardV...efreshData com.scmessenger.android              D  Dashboard data refreshed
2026-04-23 09:51:59.558 25552-25595 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:51:59.559 25552-25595 DashboardViewModel      com.scmessenger.android              D  Topology built: 1 nodes, 0 edges
2026-04-23 09:51:59.560 25552-25595 DashboardV...efreshData com.scmessenger.android              D  Dashboard data refreshed
2026-04-23 09:51:59.751 25552-25607 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:443 = blocked
2026-04-23 09:51:59.751 25552-25596 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:9001 = blocked
2026-04-23 09:51:59.752 25552-25704 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:9010 = blocked
2026-04-23 09:51:59.759 25552-25705 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:443 = blocked
2026-04-23 09:51:59.761 25552-25705 MeshRepository          com.scmessenger.android              W  No candidate addresses available (all circuit-breaker-blocked or throttled)
2026-04-23 09:51:59.762 25552-25705 MeshRepository          com.scmessenger.android              I  All relay bootstrap failed, attempting mDNS local discovery
2026-04-23 09:52:01.132 25552-25570 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:52:01.596 25552-25569 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:52:01.966 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:52:03.323 25552-25569 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:03.326 25552-25569 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=0))
2026-04-23 09:52:04.261 25552-25552 MeshRepository          com.scmessenger.android              I  Settings saved
2026-04-23 09:52:04.262 25552-25552 SettingsVi...teSettings com.scmessenger.android              I  Mesh settings saved (debounced)
2026-04-23 09:52:04.765 25552-25569 MeshRepository          com.scmessenger.android              E  mDNS fallback: no LAN peers discovered within timeout
2026-04-23 09:52:04.768 25552-25569 MeshReposi...hangeWatch com.scmessenger.android              I  Network type changed: CELLULAR → WIFI, resetting circuit breakers and re-bootstrapping
2026-04-23 09:52:04.770 25552-25569 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:52:04.776 25552-25569 MeshRepository          com.scmessenger.android              I  Racing bootstrap: network=WIFI, transports=quic→tcp→wss→ws
2026-04-23 09:52:04.779 25552-25569 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:52:04.781 25552-25569 MeshReposi...urcesAsync com.scmessenger.android              I  Using bootstrap source: StaticFallback with 4 nodes
2026-04-23 09:52:05.167 25552-25552 MeshRepository          com.scmessenger.android              I  Settings saved
2026-04-23 09:52:05.170 25552-25552 SettingsVi...teSettings com.scmessenger.android              I  Mesh settings saved (debounced)
2026-04-23 09:52:06.285 25552-25570 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:443 = blocked
2026-04-23 09:52:06.285 25552-25607 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:9010 = blocked
2026-04-23 09:52:06.286 25552-25705 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:9001 = blocked
2026-04-23 09:52:06.286 25552-25595 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:443 = blocked
2026-04-23 09:52:06.297 25552-25595 MeshRepository          com.scmessenger.android              W  All bootstrap addresses failed or timed out, falling back to mDNS
2026-04-23 09:52:06.298 25552-25595 MeshRepository          com.scmessenger.android              I  All relay bootstrap failed, attempting mDNS local discovery
2026-04-23 09:52:08.097 25552-25595 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 312399441; UID 10499; state: ENABLED
2026-04-23 09:52:08.105 25552-25552 MainViewModel           com.scmessenger.android              D  Preference onboardingCompleted: true
2026-04-23 09:52:08.107 25552-25552 MainViewModel           com.scmessenger.android              D  Preference installChoiceCompleted: true
2026-04-23 09:52:08.108 25552-25552 PreferencesRepository   com.scmessenger.android              D  Service auto-start: true
2026-04-23 09:52:08.113 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:52:08.328 25552-25595 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:08.331 25552-25705 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=0))
2026-04-23 09:52:10.434 25552-25608 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:10.434 25552-25552 MainViewModel           com.scmessenger.android              D  MeshRepository service state: RUNNING
2026-04-23 09:52:10.436 25552-25609 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=0))
2026-04-23 09:52:10.436 25552-25608 MeshRepository          com.scmessenger.android              I  SC_IDENTITY_OWN p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9 pk=374d73ebc21d677f889b806d6065a7177a2c5748f368036bd073e949bba5a2d4
2026-04-23 09:52:10.438 25552-25705 AdvertiseSettings       com.scmessenger.android              D  setTxPowerLevel: 2
2026-04-23 09:52:10.438 25552-25705 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:52:10.438 25552-25705 BluetoothLeAdvertiser   com.scmessenger.android              D  TxPower == ADVERTISE_TX_POWER_MEDIUM
2026-04-23 09:52:10.438 25552-25705 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:52:10.438 25552-25607 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:52:10.438 25552-25607 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:52:10.439 25552-25607 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:52:10.440 25552-25569 MainViewMo...ntityState com.scmessenger.android              D  refreshIdentityState() called
2026-04-23 09:52:10.440 25552-25705 BluetoothGattServer     com.scmessenger.android              D  registerCallback()
2026-04-23 09:52:10.441 25552-25705 BluetoothGattServer     com.scmessenger.android              D  registerCallback() - UUID=bdd667bc-c671-4d3b-b264-81ac73f595f3
2026-04-23 09:52:10.441 25552-25608 MeshRepository          com.scmessenger.android              I  Mesh service started successfully
2026-04-23 09:52:10.442 25552-25607 BackoffStrategy         com.scmessenger.android              D  Backoff strategy reset
2026-04-23 09:52:10.443 25552-25563 BluetoothGattServer     com.scmessenger.android              D  onServerRegistered(0)
2026-04-23 09:52:10.443 25552-25705 BluetoothGattServer     com.scmessenger.android              D  addService() - service: 0000df01-0000-1000-8000-00805f9b34fb
2026-04-23 09:52:10.444 25552-25705 BleGattServer           com.scmessenger.android              I  GATT server started with SCMessenger service
2026-04-23 09:52:10.445 25552-25563 BluetoothGattServer     com.scmessenger.android              D  onServiceAdded() - handle=147 uuid=0000df01-0000-1000-8000-00805f9b34fb status=0
2026-04-23 09:52:10.446 25552-25608 MeshReposi...edDeferred com.scmessenger.android              D  MeshService started lazily (async settings reload)
2026-04-23 09:52:10.446 25552-25698 MeshRepository          com.scmessenger.android              I  service_start_requested
2026-04-23 09:52:10.450 25552-25607 BleScanner              com.scmessenger.android              I  BLE Scanning started (background=false, fallback=false)
2026-04-23 09:52:10.453 25552-25569 MainViewMo...ntityState com.scmessenger.android              D  Identity initialized state: true
2026-04-23 09:52:10.453 25552-25705 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:52:10.454 25552-25563 BluetoothLeScanner      com.scmessenger.android              D  onScannerRegistered(status=0, scannerId=3): mScannerId=0
2026-04-23 09:52:10.458 25552-25609 MeshReposi...edDeferred com.scmessenger.android              D  Settings reloaded asynchronously after service startup
2026-04-23 09:52:10.461 25552-25698 MeshRepository          com.scmessenger.android              D  MeshService is already running
2026-04-23 09:52:10.461 25552-25607 BleScanner              com.scmessenger.android              D  Duty cycle started: 10000ms scan / 30000ms interval
2026-04-23 09:52:10.472 25552-25705 BleAdvertiser           com.scmessenger.android              D  Identity data set: 313 bytes
2026-04-23 09:52:10.474 25552-25698 MeshForegroundService   com.scmessenger.android              D  CoreDelegate wired to MeshEventBus
2026-04-23 09:52:10.476 25552-25705 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:52:10.488 25552-25552 BleAdverti...seCallback com.scmessenger.android              I  BLE Advertising started successfully
2026-04-23 09:52:10.489 25552-25705 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:52:10.490 25552-25698 MeshForegr...eshService com.scmessenger.android              I  Mesh service started successfully
2026-04-23 09:52:10.494 25552-25705 WifiTransportManager    com.scmessenger.android              D  WifiTransportManager initialized
2026-04-23 09:52:10.496 25552-25698 AnrWatchdog             com.scmessenger.android              I  ANR watchdog started (check=5000ms, threshold=10000ms)
2026-04-23 09:52:10.498 25552-25552 WifiDirect...2pReceiver com.scmessenger.android              D  WiFi P2P state changed: enabled=true
2026-04-23 09:52:10.498 25552-25705 WifiDirectTransport     com.scmessenger.android              I  WiFi Direct started
2026-04-23 09:52:10.501 25552-25552 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-2ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:52:10.501 25552-25552 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-2ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:52:10.501 25552-25552 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-2ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:52:10.501 25552-25552 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-2ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:52:10.501 25552-25552 WifiDirect...eDiscovery com.scmessenger.android              D  Service discovery request added
2026-04-23 09:52:10.502 25552-25552 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-2ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:52:10.503 25552-25552 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-2ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:52:10.503 25552-25552 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-2ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:52:10.503 25552-25552 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-1ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:52:10.503 25552-25552 WifiDirect...terService com.scmessenger.android              D  WiFi Direct service registered: scmessenger
2026-04-23 09:52:10.504 25552-25698 PerformanceMonitor      com.scmessenger.android              I  Service started - uptime tracking initialized
2026-04-23 09:52:10.505 25552-25552 WifiTransp...tDiscovery com.scmessenger.android              I  WiFi P2P Discovery started
2026-04-23 09:52:10.507 25552-25698 MeshForegr...eshService com.scmessenger.android              I  Mesh service started successfully - ANR watchdog active
2026-04-23 09:52:10.541 25552-25552 WifiDirect...rDiscovery com.scmessenger.android              D  Peer discovery started
2026-04-23 09:52:11.303 25552-25698 MeshRepository          com.scmessenger.android              E  mDNS fallback: no LAN peers discovered within timeout
2026-04-23 09:52:13.336 25552-25698 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:13.343 25552-25609 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=3))
2026-04-23 09:52:15.438 25552-25569 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:15.442 25552-25569 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=5))
2026-04-23 09:52:15.508 25552-25607 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:52:15.510 25552-25607 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:52:17.390 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:52:18.351 25552-25607 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:18.359 25552-25607 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=8))
2026-04-23 09:52:20.452 25552-25607 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:20.462 25552-25552 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:52:20.463 25552-25607 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=10))
2026-04-23 09:52:20.465 25552-25552 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:52:20.479 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:52:22.640 25552-25705 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:52:22.641 25552-25705 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:52:22.642 25552-25705 MeshRepository          com.scmessenger.android              I  ✓ Internet transport (Swarm) initiated and bridge wired
2026-04-23 09:52:22.643 25552-25705 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:52:22.646 25552-25705 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=7
2026-04-23 09:52:22.646 25552-25705 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:52:22.647 25552-25705 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=8
2026-04-23 09:52:22.649 25552-25705 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=7
2026-04-23 09:52:22.650 25552-25705 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:52:22.651 25552-25705 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=8
2026-04-23 09:52:22.653 25552-25705 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=7
2026-04-23 09:52:22.653 25552-25705 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:52:22.654 25552-25705 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=8
2026-04-23 09:52:22.656 25552-25705 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=7
2026-04-23 09:52:22.657 25552-25705 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:52:22.658 25552-25705 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=8
2026-04-23 09:52:22.659 25552-25705 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=7
2026-04-23 09:52:22.660 25552-25705 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:52:22.661 25552-25705 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=8
2026-04-23 09:52:22.663 25552-25705 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=7
2026-04-23 09:52:22.663 25552-25705 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:52:22.664 25552-25705 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=8
2026-04-23 09:52:22.665 25552-25705 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=4), next attempt in 16000ms
2026-04-23 09:52:23.371 25552-25607 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:23.389 25552-25607 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=13))
2026-04-23 09:52:23.597 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:52:25.482 25552-25609 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:25.511 25552-25607 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=15))
2026-04-23 09:52:27.807 25552-25607 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:52:27.828 25552-25607 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:52:28.403 25552-25607 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:28.418 25552-25705 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=18))
2026-04-23 09:52:30.466 25552-25552 BleScanner...d$Runnable com.scmessenger.android              W  BLE scan fallback enabled after 20026 ms without mesh advertisements; switching to unfiltered scan
2026-04-23 09:52:30.474 25552-25552 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:52:30.474 25552-25552 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:52:30.474 25552-25552 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:52:30.481 25552-25552 BleScanner              com.scmessenger.android              I  BLE scan restarted (background=false, fallback=true)
2026-04-23 09:52:30.484 25552-25564 BluetoothLeScanner      com.scmessenger.android              D  onScannerRegistered(status=0, scannerId=3): mScannerId=0
2026-04-23 09:52:30.516 25552-25698 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:30.525 25552-25698 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=20))
2026-04-23 09:52:33.424 25552-25705 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:33.433 25552-25705 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=23))
2026-04-23 09:52:35.545 25552-25698 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:35.570 25552-25698 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=25))
2026-04-23 09:52:38.447 25552-25705 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:38.466 25552-25698 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=28))
2026-04-23 09:52:38.679 25552-25698 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:52:38.715 25552-25698 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=9
2026-04-23 09:52:38.728 25552-25698 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:52:38.741 25552-25698 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=10
2026-04-23 09:52:38.755 25552-25698 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=9
2026-04-23 09:52:38.759 25552-25698 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:52:38.764 25552-25698 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=10
2026-04-23 09:52:38.771 25552-25698 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=9
2026-04-23 09:52:38.774 25552-25698 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:52:38.777 25552-25698 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=10
2026-04-23 09:52:38.782 25552-25698 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=9
2026-04-23 09:52:38.784 25552-25698 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:52:38.786 25552-25698 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=10
2026-04-23 09:52:38.790 25552-25698 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=9
2026-04-23 09:52:38.791 25552-25698 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:52:38.793 25552-25698 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=10
2026-04-23 09:52:38.797 25552-25698 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=9
2026-04-23 09:52:38.798 25552-25698 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:52:38.800 25552-25698 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=10
2026-04-23 09:52:38.801 25552-25698 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=5), next attempt in 32000ms
2026-04-23 09:52:39.118 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:52:40.500 25552-25698 MeshForegr...djustments com.scmessenger.android              D  Periodic AutoAdjust profile computed
2026-04-23 09:52:40.572 25552-25698 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:40.575 25552-25698 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=30))
2026-04-23 09:52:43.480 25552-25607 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:43.506 25552-25698 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=33))
2026-04-23 09:52:45.592 25552-25705 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:45.616 25552-25698 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=35))
2026-04-23 09:52:48.523 25552-25607 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:48.550 25552-25698 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=38))
2026-04-23 09:52:48.792 25552-25552 WindowOnBackDispatcher  com.scmessenger.android              W  OnBackInvokedCallback is not enabled for the application.
                                                                                                    Set 'android:enableOnBackInvokedCallback="true"' in the application manifest.
2026-04-23 09:52:50.173 25552-25705 SettingsViewModel       com.scmessenger.android              D  Loaded mesh settings: MeshSettings(relayEnabled=true, maxRelayBudget=200, batteryFloor=20, bleEnabled=true, wifiAwareEnabled=true, wifiDirectEnabled=true, internetEnabled=true, discoveryMode=NORMAL, onionRouting=false, coverTrafficEnabled=false, messagePaddingEnabled=false, timingObfuscationEnabled=false, notificationsEnabled=true, notifyDmEnabled=true, notifyDmRequestEnabled=true, notifyDmInForeground=false, notifyDmRequestInForeground=true, soundEnabled=true, badgeEnabled=true)
2026-04-23 09:52:50.473 25552-25552 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:52:50.475 25552-25552 BleScanner              com.scmessenger.android              V  BLE scan window ended
2026-04-23 09:52:50.616 25552-25609 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:50.619 25552-25609 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=40))
2026-04-23 09:52:53.558 25552-25609 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:53.567 25552-25569 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=43))
2026-04-23 09:52:54.650 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:52:55.627 25552-25609 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:55.638 25552-25569 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=45))
2026-04-23 09:52:56.153 25552-25552 WindowOnBackDispatcher  com.scmessenger.android              W  OnBackInvokedCallback is not enabled for the application.
                                                                                                    Set 'android:enableOnBackInvokedCallback="true"' in the application manifest.
2026-04-23 09:52:56.174 25552-25552 WindowOnBackDispatcher  com.scmessenger.android              D  setTopOnBackInvokedCallback (unwrapped): android.view.ViewRootImpl$$ExternalSyntheticLambda25@f11ce62
2026-04-23 09:52:56.236 25552-25552 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:52:56.236 25552-25552 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:e7dbf548: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:52:56.254 25552-25552 AndroidPlatformBridge   com.scmessenger.android              D  Battery changed: 58%, charging=false
2026-04-23 09:52:56.257 25552-25552 AndroidPlatformBridge   com.scmessenger.android              D  Applying BLE adjustments: scan=1000ms, advertise=200ms, txPower=0dBm
2026-04-23 09:52:56.258 25552-25552 AndroidPlatformBridge   com.scmessenger.android              D  Applying relay adjustments: maxPerHour=500, priority=50, maxPayload=32768
2026-04-23 09:52:56.259 25552-25552 AndroidPlatformBridge   com.scmessenger.android              D  Adjustment profile: HIGH for battery 58%, charging=false
2026-04-23 09:52:57.222 25552-25608 NetworkDiagnostics      com.scmessenger.android              D  DNS test failed for bootstrap.scmessenger.net: Unable to resolve host "bootstrap.scmessenger.net": No address associated with hostname
2026-04-23 09:52:57.403 25552-25608 NetworkDiagnostics      com.scmessenger.android              D  DNS test failed for relay.scmessenger.net: Unable to resolve host "relay.scmessenger.net": No address associated with hostname
2026-04-23 09:52:57.752 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:52:57.981 25552-25552 WindowOnBackDispatcher  com.scmessenger.android              D  setTopOnBackInvokedCallback (unwrapped): null
2026-04-23 09:52:57.999 25552-25573 HWUI                    com.scmessenger.android              D  endAllActiveAnimators on 0xb40000765792f0e0 (UnprojectedRipple) with handle 0xb4000076179763b0
2026-04-23 09:52:58.003 25552-25552 InputEventReceiver      com.scmessenger.android              W  Failed to send outbound event on channel '834d144 com.scmessenger.android/com.scmessenger.android.ui.MainActivity'.  status=DEAD_OBJECT(-32)
2026-04-23 09:52:58.003 25552-25552 InputEventReceiver      com.scmessenger.android              W  channel '834d144 com.scmessenger.android/com.scmessenger.android.ui.MainActivity' ~ Could not send 1 outbound event(s), status:DEAD_OBJECT
2026-04-23 09:52:58.040 25552-25552 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:52:58.040 25552-25552 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:bc5f79: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:52:58.574 25552-25569 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:52:58.580 25552-25569 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=48))
2026-04-23 09:53:00.101 25552-25698 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:53:00.410 25552-25608 NetworkDiagnostics      com.scmessenger.android              D  Port test failed for 80: failed to connect to /8.8.8.8 (port 80) from /192.168.0.119 (port 54374) after 3000ms
2026-04-23 09:53:00.638 25552-25698 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:53:00.641 25552-25698 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=50))
2026-04-23 09:53:00.808 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:53:01.585 25552-25705 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=caccf8657f8cf09f647503be0c6a51f27ee4ff616fd5b7153fdf2c41b6b55747, initialized=true, nickname=Luke
2026-04-23 09:53:03.581 25552-25705 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:53:03.583 25552-25705 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=53))
2026-04-23 09:53:03.658 25552-25608 NetworkDiagnostics      com.scmessenger.android              D  Port test failed for 9001: failed to connect to /8.8.8.8 (port 9001) from /192.168.0.119 (port 46888) after 3000ms
2026-04-23 09:53:03.893 25552-25606 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:53:05.647 25552-25705 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:53:05.649 25552-25705 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=55))
2026-04-23 09:53:06.671 25552-25608 NetworkDiagnostics      com.scmessenger.android              D  Port test failed for 9010: failed to connect to /8.8.8.8 (port 9010) from /192.168.0.119 (port 52626) after 3000ms
2026-04-23 09:53:08.589 25552-25607 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:53:08.596 25552-25705 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=58))
2026-04-23 09:53:10.514 25552-25698 MeshForegr...djustments com.scmessenger.android              D  Periodic AutoAdjust profile computed
2026-04-23 09:53:10.654 25552-25698 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:53:10.659 25552-25698 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=60))
2026-04-23 09:53:10.824 25552-25569 MeshRepository          com.scmessenger.android              I  Bootstrap: network=WIFI, cellular=false, priority=[QUIC, TCP, WEBSOCKET_WSS, WEBSOCKET_WS]
2026-04-23 09:53:10.871 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=11
2026-04-23 09:53:10.881 25552-25569 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:53:10.890 25552-25569 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:53:10.901 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=12
2026-04-23 09:53:10.913 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=11
2026-04-23 09:53:10.916 25552-25569 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:53:10.919 25552-25569 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:53:10.923 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=12
2026-04-23 09:53:10.928 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=11
2026-04-23 09:53:10.930 25552-25569 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:53:10.933 25552-25569 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:53:10.935 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=12
2026-04-23 09:53:10.939 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=11
2026-04-23 09:53:10.940 25552-25569 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 - Unknown network error: NetworkException: Network error
2026-04-23 09:53:10.942 25552-25569 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:53:10.944 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/ip4/104.28.216.43/tcp/9010/p2p/12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9 reason=Unknown network error: NetworkException: Network error total=12
2026-04-23 09:53:10.947 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=11
2026-04-23 09:53:10.949 25552-25569 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:53:10.950 25552-25569 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:53:10.952 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/443/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=12
2026-04-23 09:53:10.955 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=11
2026-04-23 09:53:10.956 25552-25569 MeshRepository          com.scmessenger.android              W  Bootstrap dial failed for /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw - Unknown network error: NetworkException: Network error
2026-04-23 09:53:10.957 25552-25569 CircuitBreaker          com.scmessenger.android              W  Circuit breaker OPENING for relay /dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw after 3 failures: Unknown network error: NetworkException: Network error
2026-04-23 09:53:10.958 25552-25569 NetworkFailureMetrics   com.scmessenger.android              D  Failure metric recorded: node=/dns4/bootstrap.scmessenger.net/tcp/80/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw reason=Unknown network error: NetworkException: Network error total=12
2026-04-23 09:53:10.960 25552-25569 MeshRepository          com.scmessenger.android              W  Bootstrap all-failed (consecutive=6), next attempt in 60000ms
2026-04-23 09:53:11.693 25552-25608 NetworkDiagnostics      com.scmessenger.android              D  Relay connectivity test failed for 34.135.34.73:9001: failed to connect to /34.135.34.73 (port 9001) from /192.168.0.119 (port 50808) after 5000ms
2026-04-23 09:53:13.529 25552-25552 AndroidRuntime          com.scmessenger.android              E  FATAL EXCEPTION: main (Fix with AI)
                                                                                                    Process: com.scmessenger.android, PID: 25552
                                                                                                    java.lang.IllegalArgumentException: Failed to find configured root that contains /data/data/com.scmessenger.android/cache/scmessenger_diagnostics_bundle.txt
                                                                                                    	at androidx.core.content.FileProvider$SimplePathStrategy.getUriForFile(FileProvider.java:867)
                                                                                                    	at androidx.core.content.FileProvider.getUriForFile(FileProvider.java:467)
                                                                                                    	at com.scmessenger.android.ui.screens.DiagnosticsScreenKt.shareDiagnosticsBundle(DiagnosticsScreen.kt:154)
                                                                                                    	at com.scmessenger.android.ui.screens.DiagnosticsScreenKt.access$shareDiagnosticsBundle(DiagnosticsScreen.kt:1)
                                                                                                    	at com.scmessenger.android.ui.screens.DiagnosticsScreenKt$DiagnosticsScreen$4$2$3.invoke(DiagnosticsScreen.kt:83)
                                                                                                    	at com.scmessenger.android.ui.screens.DiagnosticsScreenKt$DiagnosticsScreen$4$2$3.invoke(DiagnosticsScreen.kt:83)
                                                                                                    	at androidx.compose.foundation.ClickableNode$clickPointerInput$3.invoke-k-4lQ0M(Clickable.kt:639)
                                                                                                    	at androidx.compose.foundation.ClickableNode$clickPointerInput$3.invoke(Clickable.kt:633)
                                                                                                    	at androidx.compose.foundation.gestures.TapGestureDetectorKt$detectTapAndPress$2$1.invokeSuspend(TapGestureDetector.kt:255)
                                                                                                    	at kotlin.coroutines.jvm.internal.BaseContinuationImpl.resumeWith(ContinuationImpl.kt:33)
                                                                                                    	at kotlinx.coroutines.DispatchedTaskKt.resume(DispatchedTask.kt:179)
                                                                                                    	at kotlinx.coroutines.DispatchedTaskKt.dispatch(DispatchedTask.kt:168)
                                                                                                    	at kotlinx.coroutines.CancellableContinuationImpl.dispatchResume(CancellableContinuationImpl.kt:474)
                                                                                                    	at kotlinx.coroutines.CancellableContinuationImpl.resumeImpl(CancellableContinuationImpl.kt:508)
                                                                                                    	at kotlinx.coroutines.CancellableContinuationImpl.resumeImpl$default(CancellableContinuationImpl.kt:497)
                                                                                                    	at kotlinx.coroutines.CancellableContinuationImpl.resumeWith(CancellableContinuationImpl.kt:368)
                                                                                                    	at androidx.compose.ui.input.pointer.SuspendingPointerInputModifierNodeImpl$PointerEventHandlerCoroutine.offerPointerEvent(SuspendingPointerInputFilter.kt:719)
                                                                                                    	at androidx.compose.ui.input.pointer.SuspendingPointerInputModifierNodeImpl.dispatchPointerEvent(SuspendingPointerInputFilter.kt:598)
                                                                                                    	at androidx.compose.ui.input.pointer.SuspendingPointerInputModifierNodeImpl.onPointerEvent-H0pRuoY(SuspendingPointerInputFilter.kt:620)
                                                                                                    	at androidx.compose.foundation.AbstractClickableNode.onPointerEvent-H0pRuoY(Clickable.kt:1044)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:387)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:373)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:373)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:373)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:373)
                                                                                                    	at androidx.compose.ui.input.pointer.NodeParent.dispatchMainEventPass(HitPathTracker.kt:229)
                                                                                                    	at androidx.compose.ui.input.pointer.HitPathTracker.dispatchChanges(HitPathTracker.kt:144)
                                                                                                    	at androidx.compose.ui.input.pointer.PointerInputEventProcessor.process-BIzXfog(PointerInputEventProcessor.kt:120)
                                                                                                    	at androidx.compose.ui.platform.AndroidComposeView.sendMotionEvent-8iAsVTc(AndroidComposeView.android.kt:1994)
                                                                                                    	at androidx.compose.ui.platform.AndroidComposeView.handleMotionEvent-8iAsVTc(AndroidComposeView.android.kt:1945)
                                                                                                    	at androidx.compose.ui.platform.AndroidComposeView.dispatchTouchEvent(AndroidComposeView.android.kt:1829)
                                                                                                    	at android.view.ViewGroup.dispatchTransformedTouchEvent(ViewGroup.java:3143)
                                                                                                    	at android.view.ViewGroup.dispatchTouchEvent(ViewGroup.java:2826)
                                                                                                    	at android.view.ViewGroup.dispatchTransformedTouchEvent(ViewGroup.java:3143)
                                                                                                    	at android.view.ViewGroup.dispatchTouchEvent(ViewGroup.java:2826)
                                                                                                    	at android.view.ViewGroup.dispatchTransformedTouchEvent(ViewGroup.java:3143)
                                                                                                    	at android.view.ViewGroup.dispatchTouchEvent(ViewGroup.java:2826)
                                                                                                    	at android.view.ViewGroup.dispatchTransformedTouchEvent(ViewGroup.java:3143)
                                                                                                    	at android.view.ViewGroup.dispatchTouchEvent(ViewGroup.java:2826)
---------------------------- PROCESS ENDED (25552) for package com.scmessenger.android ----------------------------
2026-04-23 09:53:13.533 25552-25552 AndroidRuntime          com.scmessenger.android              E  	at com.android.internal.policy.DecorView.superDispatchTouchEvent(DecorView.java:503) (Fix with AI)
                                                                                                    	at com.android.internal.policy.PhoneWindow.superDispatchTouchEvent(PhoneWindow.java:2017)
                                                                                                    	at android.app.Activity.dispatchTouchEvent(Activity.java:4666)
                                                                                                    	at com.android.internal.policy.DecorView.dispatchTouchEvent(DecorView.java:441)
                                                                                                    	at android.view.View.dispatchPointerEvent(View.java:17196)
                                                                                                    	at android.view.ViewRootImpl$ViewPostImeInputStage.processPointerEvent(ViewRootImpl.java:8585)
                                                                                                    	at android.view.ViewRootImpl$ViewPostImeInputStage.onProcess(ViewRootImpl.java:8335)
                                                                                                    	at android.view.ViewRootImpl$InputStage.deliver(ViewRootImpl.java:7702)
                                                                                                    	at android.view.ViewRootImpl$InputStage.onDeliverToNext(ViewRootImpl.java:7759)
                                                                                                    	at android.view.ViewRootImpl$InputStage.forward(ViewRootImpl.java:7725)
                                                                                                    	at android.view.ViewRootImpl$AsyncInputStage.forward(ViewRootImpl.java:7896)
                                                                                                    	at android.view.ViewRootImpl$InputStage.apply(ViewRootImpl.java:7733)
                                                                                                    	at android.view.ViewRootImpl$AsyncInputStage.apply(ViewRootImpl.java:7953)
                                                                                                    	at android.view.ViewRootImpl$InputStage.deliver(ViewRootImpl.java:7706)
                                                                                                    	at android.view.ViewRootImpl$InputStage.onDeliverToNext(ViewRootImpl.java:7759)
                                                                                                    	at android.view.ViewRootImpl$InputStage.forward(ViewRootImpl.java:7725)
                                                                                                    	at android.view.ViewRootImpl$InputStage.apply(ViewRootImpl.java:7733)
                                                                                                    	at android.view.ViewRootImpl$InputStage.deliver(ViewRootImpl.java:7706)
                                                                                                    	at android.view.ViewRootImpl.deliverInputEvent(ViewRootImpl.java:11033)
                                                                                                    	at android.view.ViewRootImpl.doProcessInputEvents(ViewRootImpl.java:10973)
                                                                                                    	at android.view.ViewRootImpl.enqueueInputEvent(ViewRootImpl.java:10941)
                                                                                                    	at android.view.ViewRootImpl.processRawInputEvent(ViewRootImpl.java:11377)
                                                                                                    	at android.view.ViewRootImpl$WindowInputEventReceiver.onInputEvent(ViewRootImpl.java:11161)
                                                                                                    	at android.view.InputEventReceiver.dispatchInputEvent(InputEventReceiver.java:284)
                                                                                                    	at android.os.MessageQueue.nativePollOnce(Native Method)
                                                                                                    	at android.os.MessageQueue.nextDeliQueue(MessageQueue.java:780)
                                                                                                    	at android.os.MessageQueue.next(MessageQueue.java:760)
                                                                                                    	at android.os.Looper.loopOnce(Looper.java:196)
                                                                                                    	at android.os.Looper.loop(Looper.java:367)
                                                                                                    	at android.app.ActivityThread.main(ActivityThread.java:9333)
                                                                                                    	at java.lang.reflect.Method.invoke(Native Method)
                                                                                                    	at com.android.internal.os.RuntimeInit$MethodAndArgsCaller.run(RuntimeInit.java:566)
                                                                                                    	at com.android.internal.os.ZygoteInit.main(ZygoteInit.java:929)
                                                                                                    	Suppressed: kotlinx.coroutines.internal.DiagnosticCoroutineContextException: [androidx.compose.ui.platform.MotionDurationScaleImpl@f8bebfa, androidx.compose.runtime.BroadcastFrameClock@dd5d5ab, StandaloneCoroutine{Cancelling}@8684408, AndroidUiDispatcher@f4f97a1]
2026-04-23 09:53:13.551 25552-25552 Process                 com.scmessenger.android              I  Sending signal. PID: 25552 SIG: 9
---------------------------- PROCESS STARTED (25840) for package com.scmessenger.android ----------------------------
2026-04-23 09:53:14.752 25840-25840 ssenger.android         com.scmessenger.android              I  Using CollectorTypeCMC GC.
2026-04-23 09:53:14.774 25840-25840 nativeloader            com.scmessenger.android              D  Load libframework-connectivity-tiramisu-jni.so using APEX ns com_android_tethering for caller /apex/com.android.tethering/javalib/framework-connectivity-t.jar: ok
2026-04-23 09:53:14.799 25840-25840 re-initialized>         com.scmessenger.android              W  type=1400 audit(0.0:197415): avc:  granted  { execute } for  path="/data/data/com.scmessenger.android/code_cache/startup_agents/be2db1e1-agent.so" dev="dm-58" ino=2207268 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:app_data_file:s0:c243,c257,c512,c768 tclass=file app=com.scmessenger.android
2026-04-23 09:53:14.802 25840-25840 nativeloader            com.scmessenger.android              D  Load /data/user/0/com.scmessenger.android/code_cache/startup_agents/be2db1e1-agent.so using system ns (caller=<unknown>): ok
2026-04-23 09:53:14.810 25840-25840 ssenger.android         com.scmessenger.android              W  hiddenapi: DexFile /data/data/com.scmessenger.android/code_cache/.studio/instruments-0c0ed4d1.jar is in boot class path but is not in a known location
2026-04-23 09:53:14.925 25840-25840 ssenger.android         com.scmessenger.android              W  Redefining intrinsic method java.lang.Thread java.lang.Thread.currentThread(). This may cause the unexpected use of the original definition of java.lang.Thread java.lang.Thread.currentThread()in methods that have already been compiled.
2026-04-23 09:53:14.925 25840-25840 ssenger.android         com.scmessenger.android              W  Redefining intrinsic method boolean java.lang.Thread.interrupted(). This may cause the unexpected use of the original definition of boolean java.lang.Thread.interrupted()in methods that have already been compiled.
2026-04-23 09:53:15.465 25840-25840 nativeloader            com.scmessenger.android              D  Configuring clns-9 for other apk /data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk. target_sdk_version=35, uses_libraries=, library_path=/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/lib/arm64:/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk!/lib/arm64-v8a, permitted_path=/data:/mnt/expand:/data/user/0/com.scmessenger.android
2026-04-23 09:53:15.466 25840-25840 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 202956589; UID 10499; state: ENABLED
2026-04-23 09:53:15.480 25840-25840 GraphicsEnvironment     com.scmessenger.android              V  Currently set values for:
2026-04-23 09:53:15.480 25840-25840 GraphicsEnvironment     com.scmessenger.android              V    angle_gl_driver_selection_pkgs=[com.android.angle]
2026-04-23 09:53:15.480 25840-25840 GraphicsEnvironment     com.scmessenger.android              V    angle_gl_driver_selection_values=[angle]
2026-04-23 09:53:15.480 25840-25840 GraphicsEnvironment     com.scmessenger.android              V  com.scmessenger.android is not listed in per-application setting
2026-04-23 09:53:15.480 25840-25840 GraphicsEnvironment     com.scmessenger.android              V  No special selections for ANGLE, returning default driver choice
2026-04-23 09:53:15.480 25840-25840 GraphicsEnvironment     com.scmessenger.android              V  Neither updatable production driver nor prerelease driver is supported.
2026-04-23 09:53:15.524 25840-25840 WM-WrkMgrInitializer    com.scmessenger.android              D  Initializing WorkManager with default configuration.
2026-04-23 09:53:15.567 25840-25840 MeshApplication         com.scmessenger.android              I  SCMessenger application started
2026-04-23 09:53:15.575 25840-25854 StorageManager          com.scmessenger.android              D  Performing startup storage maintenance...
2026-04-23 09:53:15.582 25840-25854 StorageManager          com.scmessenger.android              D  Logs rotated successfully.
2026-04-23 09:53:15.590 25840-25854 StorageManager          com.scmessenger.android              D  Clearing cache (63 KB)...
2026-04-23 09:53:15.601 25840-25840 MeshRepository          com.scmessenger.android              D  MeshRepository initialized with storage: /data/user/0/com.scmessenger.android/files
2026-04-23 09:53:15.602 25840-25854 StorageManager          com.scmessenger.android              D  Startup maintenance complete. Available storage: 17711 MB
2026-04-23 09:53:15.606 25840-25854 MeshApplic...n$onCreate com.scmessenger.android              I  Startup storage maintenance completed
2026-04-23 09:53:15.611 25840-25840 MeshRepository          com.scmessenger.android              D  AND-CONTACTS-WIPE-001: Reinstall check - hasIdentityBackup=true, contactsOnDisk=true, historyOnDisk=true
2026-04-23 09:53:15.612 25840-25840 MeshRepository          com.scmessenger.android              D  AND-CONTACTS-WIPE-001: Normal startup - all data present
2026-04-23 09:53:15.617 25840-25840 MeshRepository          com.scmessenger.android              D  Contacts migration already completed, skipping
2026-04-23 09:53:15.626 25840-25840 nativeloader            com.scmessenger.android              D  Load /data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/lib/arm64/libjnidispatch.so using class loader ns clns-9 (caller=/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk!classes17.dex): ok
2026-04-23 09:53:15.866 25840-25840 MeshRepository          com.scmessenger.android              I  all_managers_init_success
2026-04-23 09:53:15.867 25840-25840 MeshRepository          com.scmessenger.android              I  All managers initialized successfully
2026-04-23 09:53:15.877 25840-25840 MeshRepository          com.scmessenger.android              D  AND-CONTACTS-WIPE-001: Contact data verification - Found 0 contacts
2026-04-23 09:53:15.878 25840-25840 MeshRepository          com.scmessenger.android              W  AND-CONTACTS-WIPE-001: WARNING - No contacts found. If this is unexpected, data recovery may be needed.
2026-04-23 09:53:15.883 25840-25840 MeshForegroundService   com.scmessenger.android              D  MeshForegroundService created
2026-04-23 09:53:15.890 25840-25855 MeshForegr...eshService com.scmessenger.android              I  Starting mesh service
2026-04-23 09:53:15.899 25840-25854 MeshRepository          com.scmessenger.android              D  Corruption check: contacts=0, messages=0
2026-04-23 09:53:15.901 25840-25854 MeshRepository          com.scmessenger.android              D  Database integrity check passed
2026-04-23 09:53:15.904 25840-25855 MeshForegroundService   com.scmessenger.android              D  WakeLock acquired for BLE scan windows
2026-04-23 09:53:15.904 25840-25855 AndroidPlatformBridge   com.scmessenger.android              D  AndroidPlatformBridge initializing
2026-04-23 09:53:15.910 25840-25855 AndroidPlatformBridge   com.scmessenger.android              D  Motion detection initialized (screen state proxy)
2026-04-23 09:53:15.912 25840-25855 AndroidPlatformBridge   com.scmessenger.android              D  Battery changed: 58%, charging=false
2026-04-23 09:53:15.917 25840-25876 AndroidPlatformBridge   com.scmessenger.android              D  Network changed: wifi=true, cellular=false
2026-04-23 09:53:15.927 25840-25855 AndroidPlatformBridge   com.scmessenger.android              D  Applying BLE adjustments: scan=1000ms, advertise=200ms, txPower=0dBm
2026-04-23 09:53:15.928 25840-25855 AndroidPlatformBridge   com.scmessenger.android              D  Applying relay adjustments: maxPerHour=500, priority=50, maxPayload=32768
2026-04-23 09:53:15.928 25840-25855 AndroidPlatformBridge   com.scmessenger.android              D  Adjustment profile: HIGH for battery 58%, charging=false
2026-04-23 09:53:15.929 25840-25855 MeshRepository          com.scmessenger.android              I  service_start_requested
2026-04-23 09:53:15.930 25840-25876 AndroidPlatformBridge   com.scmessenger.android              D  Applying BLE adjustments: scan=1000ms, advertise=200ms, txPower=0dBm
2026-04-23 09:53:15.930 25840-25855 MeshRepository          com.scmessenger.android              D  Starting MeshService...
2026-04-23 09:53:15.931 25840-25876 AndroidPlatformBridge   com.scmessenger.android              D  Applying relay adjustments: maxPerHour=500, priority=50, maxPayload=32768
2026-04-23 09:53:15.935 25840-25840 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:197416): avc:  denied  { search } for  name="/" dev="cgroup2" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup_v2:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:53:15.935 25840-25840 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:197417): avc:  denied  { search } for  name="/" dev="cgroup2" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup_v2:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:53:15.935 25840-25840 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:197418): avc:  denied  { search } for  name="/" dev="cgroup" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:53:15.935 25840-25840 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:197419): avc:  denied  { search } for  name="/" dev="cgroup" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 09:53:16.016 25840-25855 MeshRepository          com.scmessenger.android              I  SmartTransportRouter initialized for intelligent transport selection
2026-04-23 09:53:16.019 25840-25855 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:53:16.019 25840-25855 NetworkDetector         com.scmessenger.android              I  NetworkDetector monitoring started
2026-04-23 09:53:16.020 25840-25855 MeshRepository          com.scmessenger.android              I  NetworkDetector started — cellular-aware transport fallback active
2026-04-23 09:53:16.020 25840-25876 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:53:16.022 25840-25876 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:53:16.029 25840-25889 MeshReposi...hangeWatch com.scmessenger.android              I  Network type changed: WIFI → CELLULAR, resetting circuit breakers and re-bootstrapping
2026-04-23 09:53:16.030 25840-25854 MeshReposi...aintenance com.scmessenger.android              D  Storage maintenance check: free=17708MB / total=112912MB
2026-04-23 09:53:16.031 25840-25876 NetworkDetector         com.scmessenger.android              W  Cellular network detected (CELLULAR) — blocking ports: [9001, 9010, 4001, 5001]
2026-04-23 09:53:16.033 25840-25889 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:53:16.042 25840-25876 NetworkDetector         com.scmessenger.android              D  Network type: CELLULAR, blocked ports: [9001, 9010, 4001, 5001]
2026-04-23 09:53:16.044 25840-25889 MeshRepository          com.scmessenger.android              I  Racing bootstrap: network=CELLULAR, transports=wss→tcp→quic→tcp→ws
2026-04-23 09:53:16.044 25840-25876 NetworkDetector         com.scmessenger.android              W  Cellular network detected (CELLULAR) — blocking ports: [9001, 9010, 4001, 5001]
2026-04-23 09:53:16.046 25840-25889 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:53:16.047 25840-25876 NetworkDetector         com.scmessenger.android              D  Network type: CELLULAR, blocked ports: [9001, 9010, 4001, 5001]
2026-04-23 09:53:16.052 25840-25889 MeshReposi...urcesAsync com.scmessenger.android              I  Using bootstrap source: StaticFallback with 4 nodes
2026-04-23 09:53:16.268 25840-25876 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:53:17.559 25840-25854 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:9001 = blocked
2026-04-23 09:53:17.561 25840-25894 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:443 = blocked
2026-04-23 09:53:17.562 25840-25893 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:9010 = blocked
2026-04-23 09:53:17.564 25840-25856 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:443 = blocked
2026-04-23 09:53:17.579 25840-25893 MeshRepository          com.scmessenger.android              W  All bootstrap addresses failed or timed out, falling back to mDNS
2026-04-23 09:53:17.581 25840-25893 MeshRepository          com.scmessenger.android              I  All relay bootstrap failed, attempting mDNS local discovery
2026-04-23 09:53:19.352 25840-25876 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:53:21.569 25840-25920 ProfileInstaller        com.scmessenger.android              D  Installing profile for com.scmessenger.android
2026-04-23 09:53:22.447 25840-25876 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:53:22.587 25840-25893 MeshRepository          com.scmessenger.android              E  mDNS fallback: no LAN peers discovered within timeout
2026-04-23 09:53:22.593 25840-25893 MeshReposi...hangeWatch com.scmessenger.android              I  Network type changed: CELLULAR → WIFI, resetting circuit breakers and re-bootstrapping
2026-04-23 09:53:22.595 25840-25893 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:53:22.598 25840-25893 MeshRepository          com.scmessenger.android              I  Racing bootstrap: network=WIFI, transports=quic→tcp→wss→ws
2026-04-23 09:53:22.600 25840-25893 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 09:53:22.602 25840-25893 MeshReposi...urcesAsync com.scmessenger.android              I  Using bootstrap source: StaticFallback with 4 nodes
2026-04-23 09:53:24.108 25840-25854 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:443 = blocked
2026-04-23 09:53:24.109 25840-25894 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:9010 = blocked
2026-04-23 09:53:24.110 25840-25856 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:9001 = blocked
2026-04-23 09:53:24.113 25840-25895 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:443 = blocked
2026-04-23 09:53:24.125 25840-25856 MeshRepository          com.scmessenger.android              W  All bootstrap addresses failed or timed out, falling back to mDNS
2026-04-23 09:53:24.127 25840-25856 MeshRepository          com.scmessenger.android              I  All relay bootstrap failed, attempting mDNS local discovery
2026-04-23 09:53:25.530 25840-25876 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:53:25.559 25840-25845 ssenger.android         com.scmessenger.android              W  userfaultfd: MOVE ioctl seems unsupported: Connection timed out
2026-04-23 09:53:26.205 25840-25926 DisplayManager          com.scmessenger.android              I  Choreographer implicitly registered for the refresh rate.
2026-04-23 09:53:26.210 25840-25840 DesktopExperienceFlags  com.scmessenger.android              D  Toggle override initialized to: false
2026-04-23 09:53:26.211 25840-25926 vulkan                  com.scmessenger.android              D  searching for layers in '/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/lib/arm64'
2026-04-23 09:53:26.213 25840-25926 vulkan                  com.scmessenger.android              D  searching for layers in '/data/app/~~Jyv3LdX7F9rx3EGbdGQkag==/com.scmessenger.android-c8nrovuS6FUlALg6bLXB5w==/base.apk!/lib/arm64-v8a'
2026-04-23 09:53:26.220 25840-25840 MainActivity            com.scmessenger.android              D  MainActivity created
2026-04-23 09:53:26.222 25840-25840 AnrWatchdog             com.scmessenger.android              I  ANR watchdog started (check=5000ms, threshold=10000ms)
2026-04-23 09:53:26.223 25840-25840 MainActivity            com.scmessenger.android              I  ANR watchdog started for UI thread monitoring
2026-04-23 09:53:26.227 25840-25840 MainActivity            com.scmessenger.android              D  All permissions already granted
2026-04-23 09:53:26.236 25840-25840 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 309578419; UID 10499; state: ENABLED
2026-04-23 09:53:26.236 25840-25840 DesktopModeFlags        com.scmessenger.android              D  Toggle override initialized to: OVERRIDE_UNSET
2026-04-23 09:53:26.245 25840-25856 MeshRepository          com.scmessenger.android              I  Repository background initialization completed
2026-04-23 09:53:26.246 25840-25840 ContentCaptureHelper    com.scmessenger.android              I  Setting logging level to OFF
2026-04-23 09:53:26.250 25840-25856 MainActivity            com.scmessenger.android              D  UI components initialization completed
2026-04-23 09:53:26.252 25840-25840 MainActivity            com.scmessenger.android              D  MainActivity resumed
2026-04-23 09:53:26.253 25840-25840 MainActivity            com.scmessenger.android              D  All permissions already granted
2026-04-23 09:53:28.610 25840-25876 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:53:29.129 25840-25856 MeshRepository          com.scmessenger.android              E  mDNS fallback: no LAN peers discovered within timeout
2026-04-23 09:53:29.814 25840-25855 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:53:29.819 25840-25854 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=0))
2026-04-23 09:53:29.819 25840-25855 MeshRepository          com.scmessenger.android              I  SC_IDENTITY_OWN p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9 pk=374d73ebc21d677f889b806d6065a7177a2c5748f368036bd073e949bba5a2d4
2026-04-23 09:53:29.821 25840-25855 MeshRepository          com.scmessenger.android              I  Mesh service started successfully
2026-04-23 09:53:29.823 25840-25854 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:53:29.823 25840-25854 BluetoothLeScanner      com.scmessenger.android              D  could not find callback wrapper
2026-04-23 09:53:29.823 25840-25854 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:53:29.823 25840-25840 Activity                com.scmessenger.android              W  Slow operation Fragment dispatchResume since onResume, duration=3575
2026-04-23 09:53:29.824 25840-25840 Activity                com.scmessenger.android              W  Slow operation onPostResume since onResume, duration=3575
2026-04-23 09:53:29.824 25840-25855 MeshForegroundService   com.scmessenger.android              D  CoreDelegate wired to MeshEventBus
2026-04-23 09:53:29.824 25840-25840 Activity                com.scmessenger.android              W  Slow operation dispatchActivityPostResumed since onResume, duration=3576
2026-04-23 09:53:29.825 25840-25856 AdvertiseSettings       com.scmessenger.android              D  setTxPowerLevel: 2
2026-04-23 09:53:29.825 25840-25856 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:53:29.826 25840-25856 BluetoothLeAdvertiser   com.scmessenger.android              D  TxPower == ADVERTISE_TX_POWER_MEDIUM
2026-04-23 09:53:29.826 25840-25856 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:53:29.827 25840-25855 MeshForegr...eshService com.scmessenger.android              I  Mesh service started successfully
2026-04-23 09:53:29.827 25840-25895 AdvertiseSettings       com.scmessenger.android              D  setTxPowerLevel: 2
2026-04-23 09:53:29.828 25840-25896 WifiTransportManager    com.scmessenger.android              D  WifiTransportManager initialized
2026-04-23 09:53:29.829 25840-25854 BackoffStrategy         com.scmessenger.android              D  Backoff strategy reset
2026-04-23 09:53:29.829 25840-25855 AnrWatchdog             com.scmessenger.android              I  ANR watchdog started (check=5000ms, threshold=10000ms)
2026-04-23 09:53:29.829 25840-25895 BluetoothAdapter        com.scmessenger.android              D  isLeEnabled(): ON
2026-04-23 09:53:29.829 25840-25840 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 349153669; UID 10499; state: ENABLED
2026-04-23 09:53:29.829 25840-25850 BluetoothLeScanner      com.scmessenger.android              D  onScannerRegistered(status=0, scannerId=3): mScannerId=0
2026-04-23 09:53:29.830 25840-25895 BluetoothGattServer     com.scmessenger.android              D  registerCallback()
2026-04-23 09:53:29.830 25840-25895 BluetoothGattServer     com.scmessenger.android              D  registerCallback() - UUID=9a278fc9-bf4e-40ae-af42-66e150433cc6
2026-04-23 09:53:29.831 25840-25856 BluetoothGattServer     com.scmessenger.android              D  registerCallback()
2026-04-23 09:53:29.832 25840-25856 BluetoothGattServer     com.scmessenger.android              D  registerCallback() - UUID=a08fd9bf-cba1-429f-888c-c5b5ddfbfb66
2026-04-23 09:53:29.834 25840-25850 BluetoothGattServer     com.scmessenger.android              D  onServerRegistered(0)
2026-04-23 09:53:29.834 25840-25895 BluetoothGattServer     com.scmessenger.android              D  addService() - service: 0000df01-0000-1000-8000-00805f9b34fb
2026-04-23 09:53:29.834 25840-25854 BleScanner              com.scmessenger.android              I  BLE Scanning started (background=false, fallback=false)
2026-04-23 09:53:29.838 25840-25895 BleGattServer           com.scmessenger.android              I  GATT server started with SCMessenger service
2026-04-23 09:53:29.838 25840-25850 BluetoothGattServer     com.scmessenger.android              D  onServerRegistered(0)
2026-04-23 09:53:29.838 25840-25856 BluetoothGattServer     com.scmessenger.android              D  addService() - service: 0000df01-0000-1000-8000-00805f9b34fb
2026-04-23 09:53:29.838 25840-25925 BluetoothGattServer     com.scmessenger.android              D  onServiceAdded() - handle=147 uuid=0000df01-0000-1000-8000-00805f9b34fb status=0
2026-04-23 09:53:29.839 25840-25840 BleAdverti...seCallback com.scmessenger.android              E  BLE Advertising failed with error: 3
2026-04-23 09:53:29.840 25840-25856 BleGattServer           com.scmessenger.android              I  GATT server started with SCMessenger service
2026-04-23 09:53:29.840 25840-25925 BluetoothGattServer     com.scmessenger.android              D  onServiceAdded() - handle=155 uuid=0000df01-0000-1000-8000-00805f9b34fb status=0
2026-04-23 09:53:29.840 25840-25896 WifiDirectTransport     com.scmessenger.android              I  WiFi Direct started
2026-04-23 09:53:29.841 25840-25895 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (2 bytes)
2026-04-23 09:53:29.842 25840-25855 PerformanceMonitor      com.scmessenger.android              I  Service started - uptime tracking initialized
2026-04-23 09:53:29.843 25840-25854 BleScanner              com.scmessenger.android              D  Duty cycle started: 10000ms scan / 30000ms interval
2026-04-23 09:53:29.843 25840-25840 BleAdverti...seCallback com.scmessenger.android              W  BLE advertising failed: Already started, stopping first
2026-04-23 09:53:29.845 25840-25856 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (2 bytes)
2026-04-23 09:53:29.848 25840-25855 MeshForegr...eshService com.scmessenger.android              I  Mesh service started successfully - ANR watchdog active
2026-04-23 09:53:29.851 25840-25895 BleAdvertiser           com.scmessenger.android              D  Identity data set: 313 bytes
2026-04-23 09:53:29.853 25840-25896 BleScanner              com.scmessenger.android              D  BLE scan already in progress, reusing existing session
2026-04-23 09:53:29.857 25840-25895 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:53:29.858 25840-25856 WifiTransportManager    com.scmessenger.android              D  WifiTransportManager initialized
2026-04-23 09:53:29.859 25840-25895 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 09:53:29.860 25840-25856 WifiDirectTransport     com.scmessenger.android              W  WiFi Direct already running
2026-04-23 09:53:29.941 25840-25840 MainViewModel           com.scmessenger.android              D  MainViewModel init
2026-04-23 09:53:29.943 25840-25895 MainViewMo...rageStatus com.scmessenger.android              D  Storage refreshed: 17708 MB available (Low=false)
2026-04-23 09:53:29.944 25840-25840 MainViewModel           com.scmessenger.android              D  MeshRepository service state: RUNNING
2026-04-23 09:53:29.946 25840-25854 MainViewMo...ntityState com.scmessenger.android              D  refreshIdentityState() called
2026-04-23 09:53:29.946 25840-25895 MainViewMo...ntityState com.scmessenger.android              D  refreshIdentityState() called
2026-04-23 09:53:29.950 25840-25854 MainViewMo...ntityState com.scmessenger.android              D  Identity initialized state: true
2026-04-23 09:53:29.952 25840-25854 MainViewMo...ntityState com.scmessenger.android              D  Identity is initialized but install choice not completed, fixing preference...
2026-04-23 09:53:29.960 25840-25895 MainViewMo...ntityState com.scmessenger.android              D  Identity initialized state: true
2026-04-23 09:53:29.965 25840-25895 MainViewMo...ntityState com.scmessenger.android              D  Identity is initialized but install choice not completed, fixing preference...
2026-04-23 09:53:29.967 25840-25840 VRI[MainActivity]       com.scmessenger.android              D  WindowInsets changed: 1080x2400 statusBars:[0,132,0,0] navigationBars:[0,0,0,63] mandatorySystemGestures:[0,164,0,84] 
2026-04-23 09:53:29.969 25840-25840 WindowOnBackDispatcher  com.scmessenger.android              D  setTopOnBackInvokedCallback (unwrapped): android.view.ViewRootImpl$$ExternalSyntheticLambda25@c596c37
2026-04-23 09:53:29.988 25840-25895 PreferencesRepository   com.scmessenger.android              I  Install choice completed: true
2026-04-23 09:53:29.989 25840-25896 PreferencesRepository   com.scmessenger.android              I  Install choice completed: true
2026-04-23 09:53:29.993 25840-25895 MainViewMo...ntityState com.scmessenger.android              D  Identity is initialized but onboarding not completed, fixing preference...
2026-04-23 09:53:29.997 25840-25896 MainViewMo...ntityState com.scmessenger.android              D  Identity is initialized but onboarding not completed, fixing preference...
2026-04-23 09:53:30.003 25840-25895 PreferencesRepository   com.scmessenger.android              I  Onboarding completed: true
2026-04-23 09:53:30.006 25840-25854 PreferencesRepository   com.scmessenger.android              I  Onboarding completed: true
2026-04-23 09:53:30.013 25840-25844 ssenger.android         com.scmessenger.android              I  Compiler allocated 5049KB to compile void android.view.ViewRootImpl.performTraversals()
2026-04-23 09:53:30.016 25840-25840 HWUI                    com.scmessenger.android              I  Using FreeType backend (prop=Auto)
2026-04-23 09:53:30.054 25840-25840 MeshServiceViewModel    com.scmessenger.android              D  MeshServiceViewModel initialized
2026-04-23 09:53:30.062 25840-25896 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 09:53:30.062 25840-25894 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 09:53:30.300 25840-25840 WifiDirect...2pReceiver com.scmessenger.android              D  WiFi P2P state changed: enabled=true
2026-04-23 09:53:30.306 25840-25855 MainViewMo...ntityState com.scmessenger.android              D  refreshIdentityState() called
2026-04-23 09:53:30.318 25840-25855 MainViewMo...ntityState com.scmessenger.android              D  Identity initialized state: true
2026-04-23 09:53:30.324 25840-25855 MainViewMo...ntityState com.scmessenger.android              D  Identity is initialized but install choice not completed, fixing preference...
2026-04-23 09:53:30.327 25840-25855 PreferencesRepository   com.scmessenger.android              I  Install choice completed: true
2026-04-23 09:53:30.328 25840-25855 MainViewMo...ntityState com.scmessenger.android              D  Identity is initialized but onboarding not completed, fixing preference...
2026-04-23 09:53:30.331 25840-25855 PreferencesRepository   com.scmessenger.android              I  Onboarding completed: true
2026-04-23 09:53:30.389 25840-25840 WifiDirect...2pReceiver com.scmessenger.android              D  Disconnected from WiFi P2P group
2026-04-23 09:53:30.391 25840-25840 WifiDirect...2pReceiver com.scmessenger.android              D  This device changed
2026-04-23 09:53:30.392 25840-25840 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-416ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:53:30.392 25840-25840 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-417ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:53:30.392 25840-25840 WifiDirect...eDiscovery com.scmessenger.android              D  Service discovery request added
2026-04-23 09:53:30.393 25840-25840 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-415ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:53:30.393 25840-25840 WifiP2pManager          com.scmessenger.android              D  Ignored { when=-415ms what=139313 target=android.net.wifi.p2p.WifiP2pManager$Channel$P2pHandler async=false heapIndex=-1 }
2026-04-23 09:53:30.393 25840-25840 WifiDirect...terService com.scmessenger.android              D  WiFi Direct service registered: scmessenger
2026-04-23 09:53:30.394 25840-25840 WifiTransp...tDiscovery com.scmessenger.android              I  WiFi P2P Discovery started
2026-04-23 09:53:30.395 25840-25840 MainViewModel           com.scmessenger.android              D  Preference onboardingCompleted: true
2026-04-23 09:53:30.398 25840-25840 MainViewModel           com.scmessenger.android              D  Preference installChoiceCompleted: true
2026-04-23 09:53:30.447 25840-25840 WifiTransp...tDiscovery com.scmessenger.android              I  WiFi P2P Discovery started
2026-04-23 09:53:30.449 25840-25840 BleAdverti...seCallback com.scmessenger.android              I  BLE Advertising started successfully
2026-04-23 09:53:30.479 25840-25840 Conversati...adMessages com.scmessenger.android              D  Loaded 0 messages
2026-04-23 09:53:30.481 25840-25840 Conversati...$loadStats com.scmessenger.android              D  Loaded stats: HistoryStats(totalMessages=0, sentCount=0, receivedCount=0, undeliveredCount=0)
2026-04-23 09:53:30.483 25840-25840 Conversati...ockedPeers com.scmessenger.android              D  Loaded 0 blocked peers
2026-04-23 09:53:30.582 25840-25840 WifiDirect...rDiscovery com.scmessenger.android              D  Peer discovery started
2026-04-23 09:53:30.664 25840-25840 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 09:53:30.664 25840-25840 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:ae82f5e0: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 09:53:31.671 25840-25876 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 09:53:34.828 25840-25855 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 09:53:34.841 25840-25949 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=5))
2026-04-23 09:53:34.882 25840-25949 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 09:53:34.888 25840-25949 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9



2026-04-23 09:04:14.988 10908-10908 AndroidRuntime          pid-10908                            E  FATAL EXCEPTION: main (Fix with AI)
                                                                                                    Process: com.scmessenger.android, PID: 10908
                                                                                                    java.lang.IllegalArgumentException: Failed to find configured root that contains /data/data/com.scmessenger.android/cache/scmessenger_diagnostics_bundle.txt
                                                                                                    	at androidx.core.content.FileProvider$SimplePathStrategy.getUriForFile(FileProvider.java:867)
                                                                                                    	at androidx.core.content.FileProvider.getUriForFile(FileProvider.java:467)
                                                                                                    	at com.scmessenger.android.ui.screens.DiagnosticsScreenKt.shareDiagnosticsBundle(DiagnosticsScreen.kt:154)
                                                                                                    	at com.scmessenger.android.ui.screens.DiagnosticsScreenKt.access$shareDiagnosticsBundle(DiagnosticsScreen.kt:1)
                                                                                                    	at com.scmessenger.android.ui.screens.DiagnosticsScreenKt$DiagnosticsScreen$4$2$3.invoke(DiagnosticsScreen.kt:83)
                                                                                                    	at com.scmessenger.android.ui.screens.DiagnosticsScreenKt$DiagnosticsScreen$4$2$3.invoke(DiagnosticsScreen.kt:83)
                                                                                                    	at androidx.compose.foundation.ClickableNode$clickPointerInput$3.invoke-k-4lQ0M(Clickable.kt:639)
                                                                                                    	at androidx.compose.foundation.ClickableNode$clickPointerInput$3.invoke(Clickable.kt:633)
                                                                                                    	at androidx.compose.foundation.gestures.TapGestureDetectorKt$detectTapAndPress$2$1.invokeSuspend(TapGestureDetector.kt:255)
                                                                                                    	at kotlin.coroutines.jvm.internal.BaseContinuationImpl.resumeWith(ContinuationImpl.kt:33)
                                                                                                    	at kotlinx.coroutines.DispatchedTaskKt.resume(DispatchedTask.kt:179)
                                                                                                    	at kotlinx.coroutines.DispatchedTaskKt.dispatch(DispatchedTask.kt:168)
                                                                                                    	at kotlinx.coroutines.CancellableContinuationImpl.dispatchResume(CancellableContinuationImpl.kt:474)
                                                                                                    	at kotlinx.coroutines.CancellableContinuationImpl.resumeImpl(CancellableContinuationImpl.kt:508)
                                                                                                    	at kotlinx.coroutines.CancellableContinuationImpl.resumeImpl$default(CancellableContinuationImpl.kt:497)
                                                                                                    	at kotlinx.coroutines.CancellableContinuationImpl.resumeWith(CancellableContinuationImpl.kt:368)
                                                                                                    	at androidx.compose.ui.input.pointer.SuspendingPointerInputModifierNodeImpl$PointerEventHandlerCoroutine.offerPointerEvent(SuspendingPointerInputFilter.kt:719)
                                                                                                    	at androidx.compose.ui.input.pointer.SuspendingPointerInputModifierNodeImpl.dispatchPointerEvent(SuspendingPointerInputFilter.kt:598)
                                                                                                    	at androidx.compose.ui.input.pointer.SuspendingPointerInputModifierNodeImpl.onPointerEvent-H0pRuoY(SuspendingPointerInputFilter.kt:620)
                                                                                                    	at androidx.compose.foundation.AbstractClickableNode.onPointerEvent-H0pRuoY(Clickable.kt:1044)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:387)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:373)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:373)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:373)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:373)
                                                                                                    	at androidx.compose.ui.input.pointer.NodeParent.dispatchMainEventPass(HitPathTracker.kt:229)
                                                                                                    	at androidx.compose.ui.input.pointer.HitPathTracker.dispatchChanges(HitPathTracker.kt:144)
                                                                                                    	at androidx.compose.ui.input.pointer.PointerInputEventProcessor.process-BIzXfog(PointerInputEventProcessor.kt:120)
                                                                                                    	at androidx.compose.ui.platform.AndroidComposeView.sendMotionEvent-8iAsVTc(AndroidComposeView.android.kt:1994)
                                                                                                    	at androidx.compose.ui.platform.AndroidComposeView.handleMotionEvent-8iAsVTc(AndroidComposeView.android.kt:1945)
                                                                                                    	at androidx.compose.ui.platform.AndroidComposeView.dispatchTouchEvent(AndroidComposeView.android.kt:1829)
                                                                                                    	at android.view.ViewGroup.dispatchTransformedTouchEvent(ViewGroup.java:3143)
                                                                                                    	at android.view.ViewGroup.dispatchTouchEvent(ViewGroup.java:2826)
                                                                                                    	at android.view.ViewGroup.dispatchTransformedTouchEvent(ViewGroup.java:3143)
                                                                                                    	at android.view.ViewGroup.dispatchTouchEvent(ViewGroup.java:2826)
                                                                                                    	at android.view.ViewGroup.dispatchTransformedTouchEvent(ViewGroup.java:3143)
                                                                                                    	at android.view.ViewGroup.dispatchTouchEvent(ViewGroup.java:2826)
                                                                                                    	at android.view.ViewGroup.dispatchTransformedTouchEvent(ViewGroup.java:3143)
                                                                                                    	at android.view.ViewGroup.dispatchTouchEvent(ViewGroup.java:2826)
2026-04-23 09:53:13.529 25552-25552 AndroidRuntime          pid-25552                            E  FATAL EXCEPTION: main (Fix with AI)
                                                                                                    Process: com.scmessenger.android, PID: 25552
                                                                                                    java.lang.IllegalArgumentException: Failed to find configured root that contains /data/data/com.scmessenger.android/cache/scmessenger_diagnostics_bundle.txt
                                                                                                    	at androidx.core.content.FileProvider$SimplePathStrategy.getUriForFile(FileProvider.java:867)
                                                                                                    	at androidx.core.content.FileProvider.getUriForFile(FileProvider.java:467)
                                                                                                    	at com.scmessenger.android.ui.screens.DiagnosticsScreenKt.shareDiagnosticsBundle(DiagnosticsScreen.kt:154)
                                                                                                    	at com.scmessenger.android.ui.screens.DiagnosticsScreenKt.access$shareDiagnosticsBundle(DiagnosticsScreen.kt:1)
                                                                                                    	at com.scmessenger.android.ui.screens.DiagnosticsScreenKt$DiagnosticsScreen$4$2$3.invoke(DiagnosticsScreen.kt:83)
                                                                                                    	at com.scmessenger.android.ui.screens.DiagnosticsScreenKt$DiagnosticsScreen$4$2$3.invoke(DiagnosticsScreen.kt:83)
                                                                                                    	at androidx.compose.foundation.ClickableNode$clickPointerInput$3.invoke-k-4lQ0M(Clickable.kt:639)
                                                                                                    	at androidx.compose.foundation.ClickableNode$clickPointerInput$3.invoke(Clickable.kt:633)
                                                                                                    	at androidx.compose.foundation.gestures.TapGestureDetectorKt$detectTapAndPress$2$1.invokeSuspend(TapGestureDetector.kt:255)
                                                                                                    	at kotlin.coroutines.jvm.internal.BaseContinuationImpl.resumeWith(ContinuationImpl.kt:33)
                                                                                                    	at kotlinx.coroutines.DispatchedTaskKt.resume(DispatchedTask.kt:179)
                                                                                                    	at kotlinx.coroutines.DispatchedTaskKt.dispatch(DispatchedTask.kt:168)
                                                                                                    	at kotlinx.coroutines.CancellableContinuationImpl.dispatchResume(CancellableContinuationImpl.kt:474)
                                                                                                    	at kotlinx.coroutines.CancellableContinuationImpl.resumeImpl(CancellableContinuationImpl.kt:508)
                                                                                                    	at kotlinx.coroutines.CancellableContinuationImpl.resumeImpl$default(CancellableContinuationImpl.kt:497)
                                                                                                    	at kotlinx.coroutines.CancellableContinuationImpl.resumeWith(CancellableContinuationImpl.kt:368)
                                                                                                    	at androidx.compose.ui.input.pointer.SuspendingPointerInputModifierNodeImpl$PointerEventHandlerCoroutine.offerPointerEvent(SuspendingPointerInputFilter.kt:719)
                                                                                                    	at androidx.compose.ui.input.pointer.SuspendingPointerInputModifierNodeImpl.dispatchPointerEvent(SuspendingPointerInputFilter.kt:598)
                                                                                                    	at androidx.compose.ui.input.pointer.SuspendingPointerInputModifierNodeImpl.onPointerEvent-H0pRuoY(SuspendingPointerInputFilter.kt:620)
                                                                                                    	at androidx.compose.foundation.AbstractClickableNode.onPointerEvent-H0pRuoY(Clickable.kt:1044)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:387)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:373)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:373)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:373)
                                                                                                    	at androidx.compose.ui.input.pointer.Node.dispatchMainEventPass(HitPathTracker.kt:373)
                                                                                                    	at androidx.compose.ui.input.pointer.NodeParent.dispatchMainEventPass(HitPathTracker.kt:229)
                                                                                                    	at androidx.compose.ui.input.pointer.HitPathTracker.dispatchChanges(HitPathTracker.kt:144)
                                                                                                    	at androidx.compose.ui.input.pointer.PointerInputEventProcessor.process-BIzXfog(PointerInputEventProcessor.kt:120)
                                                                                                    	at androidx.compose.ui.platform.AndroidComposeView.sendMotionEvent-8iAsVTc(AndroidComposeView.android.kt:1994)
                                                                                                    	at androidx.compose.ui.platform.AndroidComposeView.handleMotionEvent-8iAsVTc(AndroidComposeView.android.kt:1945)
                                                                                                    	at androidx.compose.ui.platform.AndroidComposeView.dispatchTouchEvent(AndroidComposeView.android.kt:1829)
                                                                                                    	at android.view.ViewGroup.dispatchTransformedTouchEvent(ViewGroup.java:3143)
                                                                                                    	at android.view.ViewGroup.dispatchTouchEvent(ViewGroup.java:2826)
                                                                                                    	at android.view.ViewGroup.dispatchTransformedTouchEvent(ViewGroup.java:3143)
                                                                                                    	at android.view.ViewGroup.dispatchTouchEvent(ViewGroup.java:2826)
                                                                                                    	at android.view.ViewGroup.dispatchTransformedTouchEvent(ViewGroup.java:3143)
                                                                                                    	at android.view.ViewGroup.dispatchTouchEvent(ViewGroup.java:2826)
                                                                                                    	at android.view.ViewGroup.dispatchTransformedTouchEvent(ViewGroup.java:3143)
                                                                                                    	at android.view.ViewGroup.dispatchTouchEvent(ViewGroup.java:2826)
2026-04-23 11:48:35.126 22149-22149 ssenger.android         com.scmessenger.android              I  Using CollectorTypeCMC GC.
---------------------------- PROCESS STARTED (22149) for package com.scmessenger.android ----------------------------
2026-04-23 11:48:35.137 22149-22149 nativeloader            com.scmessenger.android              D  Load libframework-connectivity-tiramisu-jni.so using APEX ns com_android_tethering for caller /apex/com.android.tethering/javalib/framework-connectivity-t.jar: ok
2026-04-23 11:48:35.525 22149-22149 nativeloader            com.scmessenger.android              D  Configuring clns-9 for other apk /data/app/~~fDAhMeocr2ndkSrtlfrtbg==/com.scmessenger.android-YBj-SiJLBE4Sed_j8qVgbA==/base.apk. target_sdk_version=35, uses_libraries=, library_path=/data/app/~~fDAhMeocr2ndkSrtlfrtbg==/com.scmessenger.android-YBj-SiJLBE4Sed_j8qVgbA==/lib/arm64:/data/app/~~fDAhMeocr2ndkSrtlfrtbg==/com.scmessenger.android-YBj-SiJLBE4Sed_j8qVgbA==/base.apk!/lib/arm64-v8a, permitted_path=/data:/mnt/expand:/data/user/0/com.scmessenger.android
2026-04-23 11:48:35.526 22149-22149 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 202956589; UID 10499; state: ENABLED
2026-04-23 11:48:35.532 22149-22149 GraphicsEnvironment     com.scmessenger.android              V  Currently set values for:
2026-04-23 11:48:35.532 22149-22149 GraphicsEnvironment     com.scmessenger.android              V    angle_gl_driver_selection_pkgs=[com.android.angle]
2026-04-23 11:48:35.532 22149-22149 GraphicsEnvironment     com.scmessenger.android              V    angle_gl_driver_selection_values=[angle]
2026-04-23 11:48:35.532 22149-22149 GraphicsEnvironment     com.scmessenger.android              V  com.scmessenger.android is not listed in per-application setting
2026-04-23 11:48:35.532 22149-22149 GraphicsEnvironment     com.scmessenger.android              V  No special selections for ANGLE, returning default driver choice
2026-04-23 11:48:35.532 22149-22149 GraphicsEnvironment     com.scmessenger.android              V  Neither updatable production driver nor prerelease driver is supported.
2026-04-23 11:48:35.557 22149-22149 WM-WrkMgrInitializer    com.scmessenger.android              D  Initializing WorkManager with default configuration.
2026-04-23 11:48:35.579 22149-22226 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 312399441; UID 10499; state: ENABLED
2026-04-23 11:48:35.591 22149-22149 MeshApplication         com.scmessenger.android              I  SCMessenger application started
2026-04-23 11:48:35.594 22149-22228 StorageManager          com.scmessenger.android              D  Performing startup storage maintenance...
2026-04-23 11:48:35.602 22149-22228 StorageManager          com.scmessenger.android              D  Logs rotated successfully.
2026-04-23 11:48:35.606 22149-22228 StorageManager          com.scmessenger.android              D  Startup maintenance complete. Available storage: 17944 MB
2026-04-23 11:48:35.609 22149-22228 MeshApplic...n$onCreate com.scmessenger.android              I  Startup storage maintenance completed
2026-04-23 11:48:35.610 22149-22149 BootReceiver            com.scmessenger.android              I  Boot completed, checking auto-start preference
2026-04-23 11:48:35.613 22149-22230 DisplayManager          com.scmessenger.android              I  Choreographer implicitly registered for the refresh rate.
2026-04-23 11:48:35.617 22149-22230 vulkan                  com.scmessenger.android              D  searching for layers in '/data/app/~~fDAhMeocr2ndkSrtlfrtbg==/com.scmessenger.android-YBj-SiJLBE4Sed_j8qVgbA==/lib/arm64'
2026-04-23 11:48:35.617 22149-22230 vulkan                  com.scmessenger.android              D  searching for layers in '/data/app/~~fDAhMeocr2ndkSrtlfrtbg==/com.scmessenger.android-YBj-SiJLBE4Sed_j8qVgbA==/base.apk!/lib/arm64-v8a'
2026-04-23 11:48:35.621 22149-22149 DesktopExperienceFlags  com.scmessenger.android              D  Toggle override initialized to: false
2026-04-23 11:48:35.643 22149-22149 MeshRepository          com.scmessenger.android              D  MeshRepository initialized with storage: /data/user/0/com.scmessenger.android/files
2026-04-23 11:48:35.647 22149-22149 MeshRepository          com.scmessenger.android              D  AND-CONTACTS-WIPE-001: Reinstall check - hasIdentityBackup=true, contactsOnDisk=true, historyOnDisk=true
2026-04-23 11:48:35.648 22149-22149 MeshRepository          com.scmessenger.android              D  AND-CONTACTS-WIPE-001: Normal startup - all data present
2026-04-23 11:48:35.651 22149-22149 MeshRepository          com.scmessenger.android              D  Contacts migration already completed, skipping
2026-04-23 11:48:35.657 22149-22149 nativeloader            com.scmessenger.android              D  Load /data/app/~~fDAhMeocr2ndkSrtlfrtbg==/com.scmessenger.android-YBj-SiJLBE4Sed_j8qVgbA==/lib/arm64/libjnidispatch.so using class loader ns clns-9 (caller=/data/app/~~fDAhMeocr2ndkSrtlfrtbg==/com.scmessenger.android-YBj-SiJLBE4Sed_j8qVgbA==/base.apk!classes17.dex): ok
2026-04-23 11:48:35.834 22149-22149 MeshRepository          com.scmessenger.android              I  all_managers_init_success
2026-04-23 11:48:35.834 22149-22149 MeshRepository          com.scmessenger.android              I  All managers initialized successfully
2026-04-23 11:48:35.837 22149-22149 MeshRepository          com.scmessenger.android              D  AND-CONTACTS-WIPE-001: Contact data verification - Found 0 contacts
2026-04-23 11:48:35.837 22149-22149 MeshRepository          com.scmessenger.android              W  AND-CONTACTS-WIPE-001: WARNING - No contacts found. If this is unexpected, data recovery may be needed.
2026-04-23 11:48:35.840 22149-22149 MainActivity            com.scmessenger.android              D  MainActivity created
2026-04-23 11:48:35.841 22149-22149 AnrWatchdog             com.scmessenger.android              I  ANR watchdog started (check=5000ms, threshold=10000ms)
2026-04-23 11:48:35.842 22149-22149 MainActivity            com.scmessenger.android              I  ANR watchdog started for UI thread monitoring
2026-04-23 11:48:35.843 22149-22149 MainActivity            com.scmessenger.android              D  All permissions already granted
2026-04-23 11:48:35.844 22149-22228 MeshRepository          com.scmessenger.android              D  Corruption check: contacts=0, messages=0
2026-04-23 11:48:35.848 22149-22228 MeshRepository          com.scmessenger.android              D  Database integrity check passed
2026-04-23 11:48:35.852 22149-22149 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 309578419; UID 10499; state: ENABLED
2026-04-23 11:48:35.852 22149-22149 DesktopModeFlags        com.scmessenger.android              D  Toggle override initialized to: OVERRIDE_UNSET
2026-04-23 11:48:35.866 22149-22228 MeshRepository          com.scmessenger.android              I  Repository background initialization completed
2026-04-23 11:48:35.866 22149-22229 MeshReposi...aintenance com.scmessenger.android              D  Storage maintenance check: free=17944MB / total=112912MB
2026-04-23 11:48:35.867 22149-22149 ContentCaptureHelper    com.scmessenger.android              I  Setting logging level to OFF
2026-04-23 11:48:35.867 22149-22228 MainActivity            com.scmessenger.android              D  UI components initialization completed
2026-04-23 11:48:35.873 22149-22149 MainActivity            com.scmessenger.android              D  MainActivity resumed
2026-04-23 11:48:35.873 22149-22149 MainActivity            com.scmessenger.android              D  All permissions already granted
2026-04-23 11:48:35.874 22149-22149 MeshRepository          com.scmessenger.android              D  Permission refresh skipped: mesh service is not running
2026-04-23 11:48:35.883 22149-22149 CompatChangeReporter    com.scmessenger.android              D  Compat change id reported: 349153669; UID 10499; state: ENABLED
2026-04-23 11:48:36.045 22149-22149 MainViewModel           com.scmessenger.android              D  MainViewModel init
2026-04-23 11:48:36.046 22149-22228 MainViewMo...rageStatus com.scmessenger.android              D  Storage refreshed: 17947 MB available (Low=false)
2026-04-23 11:48:36.047 22149-22149 MainViewModel           com.scmessenger.android              D  Preference onboardingCompleted: true
2026-04-23 11:48:36.047 22149-22149 MainViewModel           com.scmessenger.android              D  Preference installChoiceCompleted: true
2026-04-23 11:48:36.048 22149-22149 MainViewModel           com.scmessenger.android              D  MeshRepository service state: STOPPED
2026-04-23 11:48:36.049 22149-22228 MainViewMo...ntityState com.scmessenger.android              D  refreshIdentityState() called
2026-04-23 11:48:36.049 22149-22228 MainViewMo...ntityState com.scmessenger.android              D  Identity initialized state: true
2026-04-23 11:48:36.069 22149-22149 VRI[MainActivity]       com.scmessenger.android              D  WindowInsets changed: 1080x2400 statusBars:[0,132,0,0] navigationBars:[0,0,0,63] mandatorySystemGestures:[0,164,0,84] 
2026-04-23 11:48:36.070 22149-22149 WindowOnBackDispatcher  com.scmessenger.android              D  setTopOnBackInvokedCallback (unwrapped): android.view.ViewRootImpl$$ExternalSyntheticLambda25@6b5e2de
2026-04-23 11:48:36.119 22149-22149 HWUI                    com.scmessenger.android              I  Using FreeType backend (prop=Auto)
2026-04-23 11:48:36.158 22149-22177 ssenger.android         com.scmessenger.android              I  Compiler allocated 5049KB to compile void android.view.ViewRootImpl.performTraversals()
2026-04-23 11:48:36.164 22149-22149 MeshServiceViewModel    com.scmessenger.android              D  MeshServiceViewModel initialized
2026-04-23 11:48:36.172 22149-22228 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 11:48:36.173 22149-22229 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 11:48:36.174 22149-22228 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=null, initialized=null, nickname=null
2026-04-23 11:48:36.174 22149-22229 MeshRepository          com.scmessenger.android              D  getIdentityInfo: result=null, initialized=null, nickname=null
2026-04-23 11:48:36.174 22149-22228 DashboardViewModel      com.scmessenger.android              D  Topology built: 0 nodes, 0 edges
2026-04-23 11:48:36.175 22149-22228 DashboardV...efreshData com.scmessenger.android              D  Dashboard data refreshed
2026-04-23 11:48:36.175 22149-22227 MeshReposi...edDeferred com.scmessenger.android              D  Lazy starting MeshService (async settings reload)...
2026-04-23 11:48:36.175 22149-22229 DashboardViewModel      com.scmessenger.android              D  Topology built: 0 nodes, 0 edges
2026-04-23 11:48:36.175 22149-22288 MeshReposi...edDeferred com.scmessenger.android              D  Lazy starting MeshService (async settings reload)...
2026-04-23 11:48:36.175 22149-22229 DashboardV...efreshData com.scmessenger.android              D  Dashboard data refreshed
2026-04-23 11:48:36.176 22149-22227 MeshRepository          com.scmessenger.android              I  service_start_requested
2026-04-23 11:48:36.176 22149-22227 MeshRepository          com.scmessenger.android              D  Starting MeshService...
2026-04-23 11:48:36.181 22149-22149 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:199624): avc:  denied  { search } for  name="/" dev="cgroup2" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup_v2:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 11:48:36.181 22149-22149 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:199625): avc:  denied  { search } for  name="/" dev="cgroup2" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup_v2:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 11:48:36.181 22149-22149 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:199626): avc:  denied  { search } for  name="/" dev="cgroup" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 11:48:36.181 22149-22149 DefaultDispatch         com.scmessenger.android              W  type=1400 audit(0.0:199627): avc:  denied  { search } for  name="/" dev="cgroup" ino=1 scontext=u:r:untrusted_app:s0:c243,c257,c512,c768 tcontext=u:object_r:cgroup:s0 tclass=dir permissive=0 app=com.scmessenger.android
2026-04-23 11:48:36.296 22149-22227 MeshRepository          com.scmessenger.android              I  SmartTransportRouter initialized for intelligent transport selection
2026-04-23 11:48:36.302 22149-22227 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 11:48:36.303 22149-22302 NetworkDetector         com.scmessenger.android              W  Cellular network detected (CELLULAR) — blocking ports: [9001, 9010, 4001, 5001]
2026-04-23 11:48:36.303 22149-22227 NetworkDetector         com.scmessenger.android              I  NetworkDetector monitoring started
2026-04-23 11:48:36.303 22149-22302 NetworkDetector         com.scmessenger.android              D  Network type: CELLULAR, blocked ports: [9001, 9010, 4001, 5001]
2026-04-23 11:48:36.303 22149-22227 MeshRepository          com.scmessenger.android              I  NetworkDetector started — cellular-aware transport fallback active
2026-04-23 11:48:36.306 22149-22302 NetworkDetector         com.scmessenger.android              W  Cellular network detected (CELLULAR) — blocking ports: [9001, 9010, 4001, 5001]
2026-04-23 11:48:36.313 22149-22302 NetworkDetector         com.scmessenger.android              D  Network type: CELLULAR, blocked ports: [9001, 9010, 4001, 5001]
2026-04-23 11:48:36.318 22149-22302 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 11:48:36.318 22149-22229 MeshReposi...hangeWatch com.scmessenger.android              I  Network type changed: CELLULAR → WIFI, resetting circuit breakers and re-bootstrapping
2026-04-23 11:48:36.320 22149-22229 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 11:48:36.322 22149-22302 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 11:48:36.323 22149-22229 MeshRepository          com.scmessenger.android              I  Racing bootstrap: network=WIFI, transports=quic→tcp→wss→ws
2026-04-23 11:48:36.325 22149-22302 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 11:48:36.325 22149-22229 CircuitBreaker          com.scmessenger.android              I  All circuit breakers reset
2026-04-23 11:48:36.326 22149-22302 NetworkDetector         com.scmessenger.android              D  Network type: WIFI, blocked ports: []
2026-04-23 11:48:36.327 22149-22229 MeshReposi...urcesAsync com.scmessenger.android              I  Using bootstrap source: StaticFallback with 4 nodes
2026-04-23 11:48:36.436 22149-22306 MainViewMo...ntityState com.scmessenger.android              D  refreshIdentityState() called
2026-04-23 11:48:36.440 22149-22306 MainViewMo...ntityState com.scmessenger.android              D  Identity initialized state: true
2026-04-23 11:48:36.444 22149-22149 Choreographer           com.scmessenger.android              I  Skipped 31 frames!  The application may be doing too much work on its main thread.
2026-04-23 11:48:36.501 22149-22149 BootReceiver$onReceive  com.scmessenger.android              I  Auto-start enabled, starting mesh service
2026-04-23 11:48:36.603 22149-22149 Conversati...adMessages com.scmessenger.android              D  Loaded 0 messages
2026-04-23 11:48:36.610 22149-22149 Conversati...$loadStats com.scmessenger.android              D  Loaded stats: HistoryStats(totalMessages=0, sentCount=0, receivedCount=0, undeliveredCount=0)
2026-04-23 11:48:36.613 22149-22149 Conversati...ockedPeers com.scmessenger.android              D  Loaded 0 blocked peers
2026-04-23 11:48:36.704 22149-22149 MeshForegroundService   com.scmessenger.android              D  MeshForegroundService created
2026-04-23 11:48:36.724 22149-22306 MeshForegr...eshService com.scmessenger.android              W  Mesh service already running; foreground promotion refreshed
2026-04-23 11:48:36.767 22149-22149 InsetsController        com.scmessenger.android              D  hide(ime())
2026-04-23 11:48:36.767 22149-22149 ImeTracker              com.scmessenger.android              I  com.scmessenger.android:452b17e8: onCancelled at PHASE_CLIENT_ALREADY_HIDDEN
2026-04-23 11:48:37.836 22149-22290 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:9010 = blocked
2026-04-23 11:48:37.836 22149-22228 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:443 = blocked
2026-04-23 11:48:37.836 22149-22291 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 34.135.34.73:9001 = blocked
2026-04-23 11:48:37.837 22149-22307 NetworkDet...probePorts com.scmessenger.android              D  Port probe: 104.28.216.43:443 = blocked
2026-04-23 11:48:37.872 22149-22307 MeshRepository          com.scmessenger.android              W  All bootstrap addresses failed or timed out, falling back to mDNS
2026-04-23 11:48:37.878 22149-22307 MeshRepository          com.scmessenger.android              I  All relay bootstrap failed, attempting mDNS local discovery
2026-04-23 11:48:40.630 22149-22347 ProfileInstaller        com.scmessenger.android              D  Installing profile for com.scmessenger.android
2026-04-23 11:48:42.887 22149-22307 MeshRepository          com.scmessenger.android              E  mDNS fallback: no LAN peers discovered within timeout
2026-04-23 11:48:43.357 22149-22149 ContactsVi...adContacts com.scmessenger.android              D  Loaded 0 contacts, filtered nearby peers to 0
2026-04-23 11:48:43.359 22149-22149 ContactsVi...rviceState com.scmessenger.android              D  Cleared all nearby peers on service stop
2026-04-23 11:48:43.368 22149-22149 WindowOnBackDispatcher  com.scmessenger.android              W  OnBackInvokedCallback is not enabled for the application.
                                                                                                    Set 'android:enableOnBackInvokedCallback="true"' in the application manifest.
2026-04-23 11:48:44.277 22149-22149 MeshServiceViewModel    com.scmessenger.android              D  MeshServiceViewModel initialized
2026-04-23 11:48:44.295 22149-22306 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 11:48:44.295 22149-22291 DashboardViewModel      com.scmessenger.android              D  Loaded 0 discovered peers (0 full)
2026-04-23 11:48:45.189 22149-22178 ssenger.android         com.scmessenger.android              W  userfaultfd: MOVE ioctl seems unsupported: Connection timed out
2026-04-23 11:48:45.453 22149-22149 MeshServiceViewModel    com.scmessenger.android              D  MeshServiceViewModel initialized
2026-04-23 11:48:45.463 22149-22229 SettingsViewModel       com.scmessenger.android              D  Loaded mesh settings: MeshSettings(relayEnabled=true, maxRelayBudget=200, batteryFloor=20, bleEnabled=true, wifiAwareEnabled=true, wifiDirectEnabled=true, internetEnabled=true, discoveryMode=NORMAL, onionRouting=false, coverTrafficEnabled=false, messagePaddingEnabled=false, timingObfuscationEnabled=false, notificationsEnabled=true, notifyDmEnabled=true, notifyDmRequestEnabled=true, notifyDmInForeground=false, notifyDmRequestInForeground=true, soundEnabled=true, badgeEnabled=true)
2026-04-23 11:48:49.150 22149-22227 MeshRepository          com.scmessenger.android              D  Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
2026-04-23 11:48:49.156 22149-22290 MeshEventBus            com.scmessenger.android              D  StatusEvent emitted: StatsUpdated(stats=ServiceStats(peersDiscovered=0, messagesRelayed=0, bytesTransferred=0, uptimeSecs=0))
2026-04-23 11:48:49.159 22149-22227 MeshRepository          com.scmessenger.android              I  SC_IDENTITY_OWN p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9 pk=374d73ebc21d677f889b806d6065a7177a2c5748f368036bd073e949bba5a2d4
2026-04-23 11:48:49.164 22149-22227 MeshRepository          com.scmessenger.android              I  Mesh service started successfully
2026-04-23 11:48:49.167 22149-22227 MeshReposi...edDeferred com.scmessenger.android              D  MeshService started lazily (async settings reload)
2026-04-23 11:48:49.167 22149-22288 MeshRepository          com.scmessenger.android              I  service_start_requested
2026-04-23 11:48:49.175 22149-22288 MeshRepository          com.scmessenger.android              D  MeshService is already running
2026-04-23 11:48:49.179 22149-22290 MeshReposi...edDeferred com.scmessenger.android              D  Settings reloaded asynchronously after service startup
2026-04-23 11:48:49.179 22149-22288 MeshReposi...edDeferred com.scmessenger.android              D  MeshService started lazily (async settings reload)
2026-04-23 11:48:49.183 22149-22289 BleScanner              com.scmessenger.android              W  Bluetooth Scanner not available
2026-04-23 11:48:49.183 22149-22228 BleAdvertiser           com.scmessenger.android              W  Bluetooth Advertiser not available
2026-04-23 11:48:49.195 22149-22289 MeshReposi...edDeferred com.scmessenger.android              D  Settings reloaded asynchronously after service startup
2026-04-23 11:48:49.197 22149-22228 BluetoothManager        com.scmessenger.android              E  Fail to get GATT Server connection
2026-04-23 11:48:49.197 22149-22228 BleGattServer           com.scmessenger.android              E  Failed to open GATT server
2026-04-23 11:48:49.202 22149-22228 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (2 bytes)
2026-04-23 11:48:49.210 22149-22228 BleAdvertiser           com.scmessenger.android              D  Identity data set: 313 bytes
2026-04-23 11:48:49.213 22149-22228 BleGattServer           com.scmessenger.android              D  BleGattServer: identity beacon set (313 bytes)
2026-04-23 11:48:49.216 22149-22228 MeshRepository          com.scmessenger.android              I  BLE GATT identity beacon updated: 374d73eb... (313 bytes, listeners=0, external=0) p2p_id=12D3KooWDYF6HEnGo8fs4gbAhBth7HdJ2nQz7wKFXXUb6WBxNof9
2026-04-23 11:48:49.246 22149-22228 WifiTransportManager    com.scmessenger.android              D  WifiTransportManager initialized
2026-04-23 11:48:49.254 22149-22228 WifiDirectTransport     com.scmessenger.android              I  WiFi Direct started
---------------------------- PROCESS ENDED (22149) for package com.scmessenger.android ----------------------------
