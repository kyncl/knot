# Daemon

Daemon mode continuously monitors your local directory tree
for file changes and automatically triggers synchronization.
While it uses the same core execution logic as `knot sync`,
it operates as a continuous background process rather than a
one-off command.

When started, Daemon mode performs an initial synchronization
run and then enters an event-listening state. To prevent
rapid, redundant syncs during large file operations, it
applies a 1.5-second debouncing delay: after a change is
detected, Knot waits 1.5 seconds for file activity to settle
before starting the sync process.

> [!TIP]
> For details on how the underlying sync process works, see
> the [Synchronization](synchronization.html) documentation.

## Command Options

Most persistent settings are managed through configuration
files, but several CLI flags are available to override
settings when starting the daemon.

### Custom Configuration Path

Specify a non-default configuration path using the `-c` or
`--config-path` flag:

```bash
knot daemon --config-path path/to/different/config.toml

# Or using the shorthand flag:
knot daemon -c path/to/different/config.toml
```

### Interactive Mode

Unlike standard synchronization, Daemon mode runs
non-interactively by default. To enable interactive TUI
prompts during automated sync runs, pass the `--interactive`
flag:

```bash
knot daemon --interactive

# Or using the shorthand flag:
knot daemon -i
```

### System Notifications

Knot can send desktop notifications whenever a daemon sync
run completes or encounters an error.

To enable desktop notifications for the daemon process, use
the `-n` or `--notifications` flag:

```bash
knot daemon --notifications

# Or using the shorthand flag:
knot daemon -n
```

### Terminal Help

To view all available flags and options directly in your
terminal, run `knot daemon --help`:

```text
Continuously monitor local directory trees and automatically sync changes

Usage: knot daemon [OPTIONS]

Options:
  -c, --config-path <CONFIG_PATH>  Path to the configuration file or workspace folder
  -i, --interactive                Shows interactive TUI prompts
  -n, --notifications              Send desktop notifications whenever a sync completes or fails
  -h, --help                       Print help
```
