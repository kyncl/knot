# Synchronization

Synchronization is the primary and most frequently used
command in Knot. It synchronizes your local source
directory with your configured remote Knots.

## Execution Process

When you run a synchronization, Knot performs the
following steps:

1. Validates the configuration file.
2. Scans the current state of your local source directory.
3. Concurrently scans all target remote Knots.
4. Displays a TUI overview detailing new, modified, and
   conflicting files (skipped in non-interactive mode).
5. Synchronizes targets sequentially (e.g., source with
   the first remote Knot, then source with the second).
6. Completes post-sync tasks (e.g., sends a system
   notification if enabled).

## Command Options

Most persistent settings are managed through configuration
files, but several CLI flags are available for
session-specific overrides.

### Custom Configuration Path

Specify a non-default configuration path using the `-c` or
`--config-path` flag:

```bash
knot sync --config-path path/to/different/config.toml

# Or using the shorthand flag:
knot sync -c path/to/different/config.toml
```

### Non-Interactive Mode

To run Knot inside automation scripts or CI/CD pipelines
without manual input, use the non-interactive flag. This
suppresses all TUI elements and confirmation prompts,
executing the sync run automatically.

```bash
knot sync --non-interactive

# Or using the shorthand flag:
knot sync -i
```

### System Notifications

Knot can send OS-level desktop notifications upon
completing a synchronization run.

To enable notifications for a run, use the `-n` or
`--notifications` flag:

```bash
knot sync --notifications

# Or using the shorthand flag:
knot sync -n
```

### Terminal Help

To view all available flags and options directly in your
terminal, run `knot sync --help`:

```text
Synchronize directory trees across configured knots

Usage: knot sync [OPTIONS]

Options:
  -c, --config-path <CONFIG_PATH>  Path to the configuration file or workspace folder
  -i, --non-interactive            Disable interactive TUI prompts (useful for scripts and CI/CD pipelines)
  -n, --notifications              Sends notification about the synchronization process
  -h, --help                       Print help
```
