# emqxctl — CLI for the EMQX Management REST API

## Overview

`emqxctl` is a command-line tool that wraps the EMQX v5 REST API (`/api/v5`), providing ergonomic access to all management functions across multiple EMQX instances. It follows the UX conventions of `kubectl` — particularly `--context` for targeting different clusters and a `~/.emqxctl/config.yaml` for storing connection profiles.

There is no existing CLI tool (official or community) that wraps the EMQX management API. This fills that gap.

## Target EMQX Version

EMQX 5.x (API base path `/api/v5`). The latest stable release as of writing is 5.10.3.

## Language & Distribution

- **Language**: Python 3.10+
- **Packaging**: Single installable via `pip install .` or `pipx install .`
- **Entry point**: `emqxctl`
- **Dependencies**: Keep minimal — `httpx` (HTTP client), `click` (CLI framework), `pyyaml` (config), `rich` (table/JSON output formatting)

## Configuration

### Config file: `~/.emqxctl/config.yaml`

```yaml
contexts:
  red:
    url: http://red.lan:18083
    api_key: "key_abc123"
    api_secret: "secret_xyz"
  staging:
    url: https://emqx-staging.example.com:18083
    api_key: "key_staging"
    api_secret: "secret_staging"
  prod:
    url: https://emqx.example.com:18083
    # bearer token auth alternative
    username: admin
    password: changeme

current-context: red
```

### Context resolution order

1. `--context` flag (highest priority)
2. `EMQXCTL_CONTEXT` environment variable
3. `current-context` field in config file

### Auth methods (per context)

The EMQX API supports two authentication methods. The config should support both:

1. **API Key + Secret** — sent as HTTP Basic Auth (`api_key` as username, `api_secret` as password)
2. **Dashboard credentials** — `username` + `password`, used to obtain a bearer token via `POST /api/v5/login`. Token should be cached in `~/.emqxctl/cache/<context>_token` with expiry tracking.

### Context management commands

```
emqxctl config set-context <name> --url <url> --api-key <key> --api-secret <secret>
emqxctl config set-context <name> --url <url> --username <user> --password <pass>
emqxctl config use-context <name>
emqxctl config get-contexts
emqxctl config current-context
emqxctl config delete-context <name>
```

## Global flags

Every command accepts:

| Flag | Short | Env Var | Description |
|---|---|---|---|
| `--context` | `-c` | `EMQXCTL_CONTEXT` | Target EMQX instance |
| `--output` | `-o` | | Output format: `table` (default), `json`, `yaml`, `wide` |
| `--verbose` | `-v` | | Show HTTP request/response details |
| `--no-color` | | `NO_COLOR` | Disable colored output |

## Command Structure

Commands are organized as `emqxctl <resource> <action> [args] [flags]`. Follow kubectl-style conventions: `get`, `list`, `describe`, `create`, `update`, `delete` where applicable.

---

## API Coverage — All Command Groups

### 1. Status & Info

```
emqxctl status                          # GET /status — quick health check
emqxctl broker                          # GET /nodes (cluster-level summary)
```

### 2. Nodes

```
emqxctl node list                       # GET /nodes
emqxctl node get <node>                 # GET /nodes/{node}
emqxctl node metrics <node>             # GET /nodes/{node}/metrics
emqxctl node stats <node>               # GET /nodes/{node}/stats
```

### 3. Cluster

```
emqxctl cluster status                  # GET /cluster
emqxctl cluster metrics                 # GET /monitor_current
emqxctl cluster metrics --latest <n>    # GET /monitor_current?latest={n}
```

### 4. Clients

```
emqxctl client list                     # GET /clients (paginated)
emqxctl client list --limit 50 --page 2
emqxctl client get <clientid>           # GET /clients/{clientid}
emqxctl client kick <clientid>          # DELETE /clients/{clientid}
emqxctl client subscriptions <clientid> # GET /clients/{clientid}/subscriptions
emqxctl client subscribe <clientid> --topic <t> --qos <q>    # POST /clients/{clientid}/subscribe
emqxctl client unsubscribe <clientid> --topic <t>             # POST /clients/{clientid}/unsubscribe
emqxctl client mqueue <clientid>        # GET /clients/{clientid}/mqueue_messages
emqxctl client inflight <clientid>      # GET /clients/{clientid}/inflight_messages
```

Filtering flags for `client list`:
- `--username`, `--ip-address`, `--conn-state` (connected/disconnected/idle)
- `--clean-start`, `--proto-ver`, `--like-clientid`, `--like-username`

### 5. Topics

```
emqxctl topic list                      # GET /topics (paginated)
emqxctl topic list --topic <pattern>    # GET /topics?topic={pattern}
emqxctl topic get <topic>               # GET /topics/{topic}
```

### 6. Subscriptions

```
emqxctl subscription list               # GET /subscriptions (paginated)
emqxctl subscription list --clientid <id>
emqxctl subscription list --topic <t>
```

### 7. Publish

```
emqxctl publish --topic <t> --payload <p> [--qos 0|1|2] [--retain]
# POST /publish

emqxctl publish-batch --file <json-file>
# POST /publish/bulk
```

### 8. Retained Messages

```
emqxctl retainer list                   # GET /retainer/messages (paginated)
emqxctl retainer get <topic>            # GET /retainer/message/{topic}
emqxctl retainer delete <topic>         # DELETE /retainer/message/{topic}
emqxctl retainer config                 # GET /retainer
emqxctl retainer config update --file <yaml/json>  # PUT /retainer
```

### 9. Rules Engine

```
emqxctl rule list                       # GET /rules
emqxctl rule get <id>                   # GET /rules/{id}
emqxctl rule create --file <yaml/json>  # POST /rules
emqxctl rule update <id> --file <yaml/json>  # PUT /rules/{id}
emqxctl rule delete <id>               # DELETE /rules/{id}
emqxctl rule test --file <yaml/json>   # POST /rules/{id}/test
emqxctl rule metrics <id>             # GET /rules/{id}/metrics
emqxctl rule reset-metrics <id>       # PUT /rules/{id}/metrics/reset
```

### 10. Data Integration — Connectors (Bridges v2)

```
emqxctl connector list                          # GET /connectors
emqxctl connector list --type <type>            # GET /connectors?type={type}
emqxctl connector get <type>:<name>             # GET /connectors/{type}:{name}
emqxctl connector create --file <yaml/json>     # POST /connectors
emqxctl connector update <type>:<name> --file <yaml/json>  # PUT /connectors/{type}:{name}
emqxctl connector delete <type>:<name>          # DELETE /connectors/{type}:{name}
emqxctl connector test <type>:<name>            # POST /connectors/{type}:{name}/test
emqxctl connector start <type>:<name>           # POST /connectors/{type}:{name}/{start|stop|restart}
emqxctl connector stop <type>:<name>
emqxctl connector restart <type>:<name>
emqxctl connector metrics <type>:<name>         # GET /connectors/{type}:{name}/metrics
```

### 11. Data Integration — Actions & Sources

```
emqxctl action list                     # GET /actions
emqxctl action get <type>:<name>        # GET /actions/{type}:{name}
emqxctl action create --file <yaml/json>
emqxctl action update <type>:<name> --file <yaml/json>
emqxctl action delete <type>:<name>
emqxctl action metrics <type>:<name>
emqxctl action start <type>:<name>
emqxctl action stop <type>:<name>

emqxctl source list                     # GET /sources
emqxctl source get <type>:<name>
emqxctl source create --file <yaml/json>
emqxctl source update <type>:<name> --file <yaml/json>
emqxctl source delete <type>:<name>
emqxctl source metrics <type>:<name>
```

### 12. Authentication

```
emqxctl authn list                      # GET /authentication
emqxctl authn get <id>                  # GET /authentication/{id}
emqxctl authn create --file <yaml/json> # POST /authentication
emqxctl authn update <id> --file <yaml/json>  # PUT /authentication/{id}
emqxctl authn delete <id>              # DELETE /authentication/{id}
emqxctl authn reorder --file <yaml/json>  # POST /authentication/order
emqxctl authn users <id> list          # GET /authentication/{id}/users
emqxctl authn users <id> create --file <yaml/json>
emqxctl authn users <id> get <userid>
emqxctl authn users <id> update <userid> --file <yaml/json>
emqxctl authn users <id> delete <userid>
emqxctl authn import <id> --file <csv/json>  # POST /authentication/{id}/import_users
```

### 13. Authorization

```
emqxctl authz list                      # GET /authorization/sources
emqxctl authz get <type>               # GET /authorization/sources/{type}
emqxctl authz create --file <yaml/json>
emqxctl authz update <type> --file <yaml/json>
emqxctl authz delete <type>
emqxctl authz reorder --file <yaml/json>
emqxctl authz cache clean              # DELETE /authorization/cache
emqxctl authz cache clean --clientid <id>
```

### 14. Banned Clients

```
emqxctl ban list                        # GET /banned
emqxctl ban create --who <clientid|username|peerhost> --as <type> [--reason <r>] [--until <time>]
# POST /banned
emqxctl ban delete <as>/<who>           # DELETE /banned/{as}/{who}
emqxctl ban clear                       # DELETE /banned
```

### 15. Listeners

```
emqxctl listener list                   # GET /listeners
emqxctl listener get <id>              # GET /listeners/{id}
emqxctl listener create --file <yaml/json>  # POST /listeners
emqxctl listener update <id> --file <yaml/json>
emqxctl listener delete <id>
emqxctl listener start <id>            # POST /listeners/{id}/start
emqxctl listener stop <id>             # POST /listeners/{id}/stop
emqxctl listener restart <id>          # POST /listeners/{id}/restart
```

### 16. Metrics & Stats

```
emqxctl metrics                         # GET /metrics
emqxctl metrics --node <node>           # GET /nodes/{node}/metrics
emqxctl stats                           # GET /stats
emqxctl stats --node <node>             # GET /nodes/{node}/stats
```

### 17. Alarms

```
emqxctl alarm list                      # GET /alarms
emqxctl alarm list --activated          # GET /alarms?activated=true
emqxctl alarm list --deactivated        # GET /alarms?activated=false
emqxctl alarm clear                     # DELETE /alarms
```

### 18. Tracing (Log Trace)

```
emqxctl trace list                      # GET /trace
emqxctl trace get <name>               # GET /trace/{name}
emqxctl trace create --name <n> --type <clientid|topic|ip_address> --target <val> [--start <t>] [--end <t>]
# POST /trace
emqxctl trace delete <name>            # DELETE /trace/{name}
emqxctl trace stop <name>              # PUT /trace/{name}/stop
emqxctl trace log <name>               # GET /trace/{name}/log
emqxctl trace download <name>          # GET /trace/{name}/download
emqxctl trace clear                    # DELETE /trace
```

### 19. Configuration

```
emqxctl config get [<root_key>]         # GET /configs [/configs/{root_key}]
emqxctl config update <root_key> --file <yaml/json>  # PUT /configs/{root_key}
emqxctl config reset <root_key>         # POST /configs/{root_key}/reset

# Hot configuration reload
emqxctl config global                   # GET /configs/global_zone
```

### 20. Plugins

```
emqxctl plugin list                     # GET /plugins
emqxctl plugin get <name>              # GET /plugins/{name}
emqxctl plugin install --file <path>   # POST /plugins/install
emqxctl plugin uninstall <name>        # DELETE /plugins/{name}
emqxctl plugin start <name>            # PUT /plugins/{name}/start
emqxctl plugin stop <name>             # PUT /plugins/{name}/stop
emqxctl plugin config <name>           # GET /plugins/{name}/config
emqxctl plugin config <name> --file <yaml/json>  # PUT /plugins/{name}/config
emqxctl plugin reorder --file <yaml/json>  # POST /plugins/order
```

### 21. API Keys (self-management)

```
emqxctl apikey list                     # GET /api_key
emqxctl apikey get <name>              # GET /api_key/{name}
emqxctl apikey create --name <n> --expired-at <t> [--role <admin|viewer|publisher>]
# POST /api_key
emqxctl apikey update <name> --file <yaml/json>  # PUT /api_key/{name}
emqxctl apikey delete <name>           # DELETE /api_key/{name}
```

### 22. Dashboard Users

```
emqxctl admin list                      # GET /users
emqxctl admin get <username>            # GET /users/{username}
emqxctl admin create --username <u> --password <p> [--role <admin|viewer|publisher>]
emqxctl admin update <username> --role <role>
emqxctl admin delete <username>
emqxctl admin change-password <username> --old <old> --new <new>
# PUT /users/{username}/change_pwd
```

### 23. Gateways

```
emqxctl gateway list                    # GET /gateways
emqxctl gateway get <name>             # GET /gateways/{name}
emqxctl gateway enable <name>          # PUT /gateways/{name} (enable: true)
emqxctl gateway disable <name>         # PUT /gateways/{name} (enable: false)
emqxctl gateway update <name> --file <yaml/json>
emqxctl gateway clients <name> list    # GET /gateways/{name}/clients
emqxctl gateway clients <name> get <clientid>
emqxctl gateway clients <name> kick <clientid>
emqxctl gateway authn <name> list      # GET /gateways/{name}/authentication
emqxctl gateway authn <name> create --file <yaml/json>
emqxctl gateway authn <name> update --file <yaml/json>
emqxctl gateway authn <name> delete
emqxctl gateway authn <name> users list
emqxctl gateway authn <name> users create --file <yaml/json>
emqxctl gateway listeners <name> list  # GET /gateways/{name}/listeners
emqxctl gateway listeners <name> get <id>
emqxctl gateway listeners <name> create --file <yaml/json>
emqxctl gateway listeners <name> update <id> --file <yaml/json>
emqxctl gateway listeners <name> delete <id>
```

### 24. Schema Registry

```
emqxctl schema list                     # GET /schemas
emqxctl schema get <name>              # GET /schemas/{name}
emqxctl schema create --file <yaml/json>
emqxctl schema update <name> --file <yaml/json>
emqxctl schema delete <name>
```

### 25. Slow Subscriptions

```
emqxctl slow-sub config                 # GET /slow_subscriptions/settings
emqxctl slow-sub config update --file <yaml/json>
emqxctl slow-sub list                   # GET /slow_subscriptions
emqxctl slow-sub clear                  # DELETE /slow_subscriptions
```

### 26. Topic Metrics

```
emqxctl topic-metrics list              # GET /topic-metrics
emqxctl topic-metrics get <topic>       # GET /topic-metrics/{topic}
emqxctl topic-metrics register <topic>  # POST /topic-metrics
emqxctl topic-metrics deregister <topic>  # DELETE /topic-metrics/{topic}
```

### 27. Data Backup

```
emqxctl backup list                     # GET /data/export
emqxctl backup create                   # POST /data/export
emqxctl backup import --file <path>     # POST /data/import
emqxctl backup download <name>          # GET /data/export/{name}
emqxctl backup upload --file <path>     # POST /data/files
emqxctl backup delete <name>            # DELETE /data/export/{name}
```

### 28. Certificates

```
emqxctl cert list                       # GET /ssl_certs
emqxctl cert get <id>                  # GET /ssl_certs/{id}
```

---

## Pagination

Commands that return paginated lists must handle pagination automatically:

- Default: fetch first page, show results with pagination info
- `--all` flag: iterate all pages and return complete results
- `--limit <n>` and `--page <n>`: manual page control
- For cursor-based endpoints (mqueue, inflight): `--cursor <pos>` flag

## Error Handling

- Parse EMQX error responses (JSON with `code` and `reason` fields) and display human-readable messages
- Non-2xx HTTP status should print the status code, EMQX error code, and reason
- Connection errors should suggest checking `--context` configuration
- 401 errors should suggest checking credentials

## Output Formatting

### Table (default)
```
$ emqxctl client list
CLIENTID         USERNAME    IP ADDRESS      CONNECTED    PROTO    CLEAN START
sensor-01        device1     192.168.1.10    true         5        true
sensor-02        device2     192.168.1.11    true         5        false
--- Page 1/3 (50 total) ---
```

### JSON
```
$ emqxctl client list -o json
[
  {"clientid": "sensor-01", "username": "device1", ...},
  ...
]
```

### YAML
Same data, YAML formatted.

### Wide
Table with all fields (no column truncation).

## Shell Completion

Provide shell completion for bash, zsh, and fish:

```
emqxctl completion bash > /etc/bash_completion.d/emqxctl
emqxctl completion zsh > ~/.zfunc/_emqxctl
emqxctl completion fish > ~/.config/fish/completions/emqxctl.fish
```

## Raw API Escape Hatch

For endpoints not yet covered or future API additions:

```
emqxctl api GET /topics
emqxctl api POST /publish --data '{"topic":"t/1","payload":"hello"}'
emqxctl api PUT /configs/mqtt --file mqtt-config.json
emqxctl api DELETE /banned/clientid/badclient
```

This uses the configured context for auth and base URL, so it's still easier than raw curl.

## Project Structure

```
emqxctl/
  pyproject.toml
  src/
    emqxctl/
      __init__.py
      cli.py              # click entry point, global flags
      config.py            # config file loading, context resolution
      client.py            # HTTP client (httpx), auth handling, pagination
      output.py            # table/json/yaml formatting (rich)
      commands/
        __init__.py
        status.py
        node.py
        cluster.py
        client_cmd.py      # (client.py would shadow the HTTP client module)
        topic.py
        subscription.py
        publish.py
        retainer.py
        rule.py
        connector.py
        action.py
        source.py
        authn.py
        authz.py
        ban.py
        listener.py
        metrics.py
        alarm.py
        trace.py
        config_cmd.py
        plugin.py
        apikey.py
        admin.py
        gateway.py
        schema.py
        slow_sub.py
        topic_metrics.py
        backup.py
        cert.py
        api.py              # raw API escape hatch
        completion.py
  tests/
    ...
```

## Implementation Notes

- Each command module should be a click group registered in `cli.py`
- The HTTP client should be a shared class (`EmqxClient`) instantiated per-invocation with the resolved context
- `EmqxClient` handles: base URL, auth headers, pagination iteration, error parsing, retries on transient failures
- For bearer token auth, cache tokens per-context and refresh automatically on 401
- Use `click.pass_context` to thread the client and output formatter through commands
- Keep command implementations thin — they should parse args, call the client, and pass results to the formatter
- The `--file` flag should accept both YAML and JSON (detect by extension or try both parsers)
- Stdin (`--file -`) should be supported for piping
