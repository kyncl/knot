# Initialization

The primary way to generate a new Knot configuration is by
initializing it through the CLI.

To start the interactive initialization wizard, run:

```bash
knot init
```

This command launches an interactive form that guides you
through setting up all necessary configuration options.

> [!WARNING]
> Running `knot init` in a directory with an existing `.knot`
> folder will overwrite your current configuration files
> without prompting. Use with caution.

## Interactive CLI Controls

The initialization wizard features custom input fields with
built-in shortcuts to speed up configuration.

### Path Autocomplete

When prompted to provide a local file or directory path, Knot
provides built-in path autocompletion:

* **Scanning:** As you type, Knot scans the input. If it
  matches a valid directory, it displays its contents.
* **Navigation:** Use the `Up` and `Down` arrow keys to
  highlight different files or folders.
* **Autocompletion:** Press `Tab` to autocomplete the
  currently highlighted option.
* **Confirmation:** Press `Enter` to confirm your final
  path selection.

### Multi-Select Menus

When prompted to select multiple options from a list, use
the following keyboard controls:

* `Right Arrow`: Select all options.
* `Left Arrow`: Deselect all options.
* `Up`/`Down Arrow`: Navigate between options.
* `Space`: Toggle the currently highlighted option.
* `Enter`: Confirm your final selections.

## Initialization Stages

The `knot init` wizard guides you through configuration in
sequential stages. Each prompt includes a brief help message
explaining the expected input.

### 1. General Configuration

The wizard begins by asking which general settings you want
to configure manually and which should use default values:

* **[Features](../configuration/general-configuration.html#features)**
* **[Performance](../configuration/general-configuration.html#performance)**
* **Ignore Patterns:** Enter ignore patterns one by one, or
  paste a comma-separated list to add multiple patterns
  simultaneously. Submit an empty input to finish.

### 2. Source Knot

Next, Knot prompts you to define your source Knot's
connection type and target path. If you select a network
protocol (like SSH) instead of `Local`, Knot prompts you for
connection credentials (username, hostname, and port).

You can enter these credentials field-by-field or supply
them all at once using one of the following URI formats:
`type://username@host:port`, `username@host:port`,
`username@host`, or `host`.

> [!TIP]
> If you use the URI format, you can omit the `type://`
> prefix if you already selected the connection type in the
> prior step (e.g., `admin@host:port`).

### 3. Remote Knots

Finally, Knot prompts you to configure your remote Knots.
This step loops, allowing you to configure as many remote
targets as needed before finalizing the configuration.
