# Archiving

The `knot archive` command suite provides tools to manage,
inspect, restore, and prune archived files on remote
targets.

You can interact with archives programmatically using direct
flags or visually using the built-in Terminal User Interface
(TUI).

## Global Options

The following flags can be applied to the base `knot archive`
command before specifying a subcommand:

* `-i, --index <INDEX>`: Specify the target index when
  multiple remote Knots exist. If omitted, Knot prompts
  interactively.
* `-c, --config-path <PATH>`: Provide a custom path to the
  configuration file or project workspace.
* `-n, --notifications`: Enable desktop notifications for
  the archiving process.

---

## Subcommands

### `resolve`

Launches the interactive TUI to explore and manage archived
files on the target remote.

```bash
knot archive resolve [OPTIONS]
```

* `--root-path <PATH>`: The root directory to inspect
  (default: `./`).

### `list`

Outputs a serialized inventory of remote archived files for
scripting or pipelines.

```bash
knot archive list [OPTIONS]
```

* `--root-path <PATH>`: The root directory to inspect.
* `--format <FORMAT>`: Output serialization format (`json` or
  `binary`, default: `json`).
* `--compress`: Compresses the output data (Base64-encoded).

### `recover`

Restores specific files or entire directories from a remote
archive back to your local workspace.

```bash
knot archive recover [OPTIONS]
```

* `--root-path <PATH>`: Root directory to inspect for
  recovery.
* `-t, --target <TARGET>`: Specific archived files or paths
  to restore from the remote tree.
* `-f, --force`: Overwrite existing local files.

### `remove`

Deletes remote archives to free disk space. Supports
targeting specific files or pruning automatically by age.

```bash
knot archive remove [OPTIONS]
```

* `--root-path <PATH>`: Root directory to inspect for
  removal.
* `-t, --target <TARGET>`: Specific archived files or paths
  to delete.
* `--older-than <DURATION>`: Automatically prune archives
  older than a specified duration (e.g., `30d` for 30 days,
  `2w` for 2 weeks) based on modification time.
* `--force`: Skip interactive confirmation prompts and force
  deletion.

### `compress`

Manually compresses specified directories and files into
remote Knot archives.

```bash
knot archive compress [OPTIONS]
```

* `-d, --dir <DIR>`: Directory to include in the archive.
* `-f, --file <FILE>`: Individual file to include in the
  archive.
