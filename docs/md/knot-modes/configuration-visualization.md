# Visualizing the Configuration

To view a summary of your current configuration, run the
following command:

```bash
knot config
# You can specify different path
knot config --config-path path/to/different/config
# Shorten version
knot config -c path/to/different/config
```

This is useful for quickly verifying your current
environment settings and active configuration state.

> [!NOTE]
> This command outputs a human-readable summary separated
> by headers, not valid TOML. Do not pipe this output into
> scripts or TOML parsers.

## Machine-Readable Formats

For pipeline integration or automated parsing, use the
`--format` (or `-f`) flag to output valid syntax.

Example for JSON format:
```bash
knot config --format json
# or shorten
knot config -f json
```

Knot currently supports the following formats:

* **TOML**: Tom's Obvious Minimal Language
* **JSON**: JavaScript Object Notation

Regardless of the chosen format, the output is grouped
into three primary objects or tables:

* **`main`**: Your primary configuration. This includes the
  source Knot and global settings (performance, features,
  and experimental toggles).
* **`remote`**: A list of all your configured remote Knots.
* **`ignore_patterns`**: A list of all exclude patterns
  parsed from your `knotignore` file.
