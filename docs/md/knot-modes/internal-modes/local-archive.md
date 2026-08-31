# Local Archive

The `knot local-archive` command suite mirrors the standard
`archive` command, but operates exclusively on the local
workspace rather than remote targets.

This allows you to inspect, restore, prune, and compress
archives locally without network connectivity or remote Knot
nodes.

## Global Options

* `-h, --help`: Print help information.

## Subcommands

Subcommands accept options identical to their remote
counterparts, executing all operations directly on local
filesystem archives.

### `resolve`

Launches the interactive TUI to visually inspect, explore, and
manage local archives.

```bash
knot local-archive resolve
```

### `list`

Outputs a serialized inventory of local archived files (in JSON
or binary format) for scripting or pipelines.

```bash
knot local-archive list
```

### `recover`

Restores specific files or directories from local archives back
to their active workspace state.

```bash
knot local-archive recover
```

### `remove`

Deletes local archives to free disk space. Supports targeting
specific files or pruning automatically by age (e.g., using
`--older-than 30d`).

```bash
knot local-archive remove
```

### `compress`

Manually compresses specified local files and directories into
Knot archives.

```bash
knot local-archive compress
```
