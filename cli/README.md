# SCMessenger CLI

Current CLI binary crate: `scmessenger-cli`.

## Run

From repository root:

```bash
cargo run -p scmessenger-cli -- --help
```

If installed:

```bash
scmessenger-cli --help
```

Optional shell alias:

```bash
alias scm='scmessenger-cli'
```

## Commands

Top-level commands (verified from `--help`):

- `init`
- `identity`
- `contact`
- `config`
- `history`
- `start`
- `send`
- `status`
- `stop`
- `test`

## Common Flows

### Initialize identity

```bash
scmessenger-cli init
scmessenger-cli identity show
```

### Add contact and send message

```bash
scmessenger-cli contact add <peer-id> <public-key> --name <nickname>
scmessenger-cli send <contact-or-peer-id> "hello"
```

### Start live node mode

```bash
scmessenger-cli start
```

`start` launches:

- libp2p swarm node
- local control API on `127.0.0.1:9876`
- web landing/dashboard server (default HTTP port: `9000`)
- interactive terminal commands (`send`, `contacts`, `peers`, `status`, `quit`)

### Stop a running node

```bash
scmessenger-cli stop
```

## Config Keys

Supported keys for `config get|set`:

- `listen_port`
- `enable_mdns`
- `enable_dht`
- `storage_path`
- `max_peers`
- `connection_timeout`
- `enable_nat_traversal`
- `enable_relay`

Bootstrap nodes are managed under:

```bash
scmessenger-cli config bootstrap add <multiaddr>
scmessenger-cli config bootstrap remove <multiaddr>
scmessenger-cli config bootstrap list
```

## Test

```bash
cargo test -p scmessenger-cli
```
