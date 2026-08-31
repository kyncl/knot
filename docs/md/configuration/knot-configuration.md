# Knot Configuration

This page covers the configuration options available for
individual Knots and their unique behaviors.

## Connection Types

Each Knot requires a defined connection type to dictate how
the application handles the directory. Valid types include
`"Local"` and `"SSH"`. For a complete breakdown, see the
[Knots and Their Types](../knots-and-their-types.html)
chapter.

## General Knot Properties

The following base properties apply to all Knots, whether
they are configured as source or remote:

| Property | Default | Type | Description |
|---|---|---|---|
| `type` | `"Local"` | Enum | Defines the connection type. |
| `path` | `"./"` | String | The absolute or relative path to the specific directory. |

## Remote Knots

Remote Knots are defined in the `knots.toml` file. You can
configure multiple remote Knots by adding multiple
`[[knots]]` arrays. Below is an example configuration:

```toml
[[knots]]

[knots.behavior]
uniques = "Archive"
conflicts = "Newer"

[knots.config]
type = "SSH"
path = "path/to/remote/directory"

[knots.config.credentials]
username = "username"
host = "localhost"
port = 22
connection_limit = 1

[knots.config.credentials.authentication]
type = "Password"
```

## Behaviors

Unlike the source Knot, remote Knots accept specific behavior
rules. Behaviors define how Knot handles unique files (those
existing in only one location) and conflict files (those with
the same path but different content).

### Unique Files

This property defines how Knot handles newly discovered files
that exist on only one side of the synchronization process.

| Behavior | Description |
|---|---|
| `Archive` | Syncs new source files to the remote. New remote files are compressed and archived. See the [Archive page](../knot-modes/archiving.html) for details. |
| `MirrorSource` | Adds new source files to the remote. Deletes new remote files. |
| `MirrorRemote` | Adds new remote files to the source. Deletes new source files. |
| `AddOnly` | Adds new files to the missing side. Never deletes files. `OnlyAdd` is also accepted as an alias. |
| `Ask` | Opens the TUI conflict solver to prompt for a manual decision. |
| `Skip` | Ignores the unique files entirely. |

### Conflict Files

This property defines how Knot handles files that exist on
both sides but contain different content.

| Behavior | Description |
|---|---|
| `Newer` | The file with the most recent modification timestamp overwrites the older file. |
| `Older` | The file with the oldest modification timestamp overwrites the newer file. |
| `Source` | The source file overwrites the remote file. |
| `Remote` | The remote file overwrites the source file. |
| `Ask` | Opens the TUI conflict solver to prompt for a manual decision. |
| `Skip` | Ignores the conflicting files entirely. |

## Connection Configuration

Remote Knots using network protocols like SSH require
connection credentials. For security reasons, Knot does not
store passwords in plaintext inside the configuration file.

| Property | Default | Type | Description |
|---|---|---|---|
| `username` | `"username"` | String | The username for the remote connection. |
| `host` | `"localhost"` | String | The hostname or IP address of the target. |
| `port` | `22` | U16 | The port on which the remote service runs. |
| `connection_limit` | `1` | U64 | The maximum concurrent SSH sessions Knot can establish. |

> [!WARNING]
> Opening multiple concurrent connections to a strict remote
> server may trigger rate-limiting or firewall blocks. Verify
> your server's SSH daemon configuration before increasing
> the connection limit.

### Authentication Methods

Knot supports multiple authentication methods. Within your
configuration schema, the `authentication` block uses a
`type` property to define the chosen method.

| Type | Additional Properties | Description |
|---|---|---|
| `None` | N/A | Performs no authentication. |
| `Password` | N/A | Uses a password retrieved from your OS keyring or CLI prompt. When prompted, Knot offers to save entered passwords to the OS keyring, which can be managed via the [`modify`](../knot-modes/modification.html#credentials-amp-network) command. |
| `PrivateKey` | `key_path` (Req)<br>`cert_path` (Opt) | Authenticates using an SSH private key and optional certificate. |
