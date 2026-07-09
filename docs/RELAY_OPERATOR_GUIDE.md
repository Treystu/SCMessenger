
Pass criteria per direction (A->iOS, iOS->A):

- same `msg=<id>` appears with `delivery_attempt` timeline entries,
- recipient shows `msg_rx_processed`,
- sender shows `state=delivered` without duplicate terminal oscillation.

Fail criteria:

- repeated retry loops without `msg_rx_processed`,
- recipient ingest observed but sender never reaches `delivered`,
- conflicting terminal states for the same message ID after retry window.

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Clients can't connect | Check firewall/port forwarding on TCP 9001 and 443 |
| Node shows 0 peers | Verify internet connectivity; check bootstrap config |
| High CPU usage | Check `set_relay_budget` to limit relay throughput |
| PeerId changed | Check if `storage/relay_network_key.pb` was deleted |