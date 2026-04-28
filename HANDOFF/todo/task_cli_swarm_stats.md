TARGET: cli\src\main.rs

SUMMARY:
Add a new CLI command 'scm swarm stats' that invokes the newly wired 'swarm_get_all_connection_stats' API and displays the results in a formatted table.

CONTEXT:
The Core API 'swarm_get_all_connection_stats' has been wired through UniFFI and into IronCore. Now the CLI needs a way to show this data to the developer for debugging transport issues.

ACCEPTANCE CRITERIA:
1. 'scm swarm stats' command is added to the CLI.
2. It fetches stats from the IronCore instance (either via direct storage if standalone, or via API if daemon is running).
3. It prints a table showing: Peer ID, State, Latency, Messages Sent/Failed, Bytes Sent/Received.
4. If no connections exist, it should state so clearly.
