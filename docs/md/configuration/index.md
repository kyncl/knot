# Configuration

This section covers how to configure Knot for your specific
environment. You can manage configurations by manually
editing configuration files or by using the `knot modify`
and `knot init` commands.

By default, Knot looks for a `config.toml` file inside a
`.knot` directory in your workspace. Knot also supports
flexible configuration layouts. This allows you to swap
between multiple environments (such as `prod.toml` or
`dev.toml`) while maintaining a centralized setup.

When manually editing `config.toml` and `knots.toml`, Knot
accepts multiple casing formats for enums. Supported formats
include `kebab-case`, `snake_case`, `camelCase`, `PascalCase`,
`lowercase`, and space-separated words. While Knot parses
all of these, choose one and use it consistently.

> [!NOTE]
> Case sensitivity still applies to values where casing alters
> meaning, such as unit measurements (e.g., `KiB` for kibibytes
> versus `Kib` for kibibits).

## Configuration Rules

Many Knot commands allow you to override the default
configuration path using the `-c` or `--config-path` flags.
If you find a command that uses a different flag, please
open a [GitHub issue](https://github.com/kyncl/knot/issues).

For example, to run a synchronization with a custom config:

```bash
knot sync --config-path path/to/different/config.toml
# Or using the shorthand flag:
knot sync -c path/to/different/config.toml
```

When you pass a custom configuration path, Knot resolves the
configuration directory and primary file based on these rules:

* **Directory Prefix:** Custom configuration directories must
  begin with the `.knot` prefix (e.g., `.knot_custom`).
* **Nested Config Files:** If you specify a file inside a valid
  configuration directory (e.g., `.knot/prod.toml`), Knot uses
  that file as the main configuration. The parent folder serves
  as the configuration directory.
* **Root Config Files:** If you specify a file in the workspace
  root (e.g., `prod_knot.toml`), Knot uses it as the main
  configuration. It defaults to the standard `.knot` folder for
  other required settings (such as ignore files).
* **Directory Targets:** If you provide a directory path (or no
  arguments), Knot targets `config.toml` inside the `.knot`
  folder within that directory.

## Path Resolution Examples

| CLI Input | Resolved Config File | Resolved Config Dir |
|---|---|---|
| (None) or `.` | `./.knot/config.toml` | `./.knot` |
| `./project` | `./project/.knot/config.toml` | `./project/.knot` |
| `./.knot_custom` | `./.knot_custom/config.toml` | `./.knot_custom` |
| `./.knot/prod.toml` | `./.knot/prod.toml` | `./.knot` |
| `./.knot_custom/dev.toml` | `./.knot_custom/dev.toml` | `./.knot_custom` |
| `./prod_knot.toml` | `./prod_knot.toml` | `./.knot` |

> [!TIP]
> You can store `prod.toml` and `dev.toml` together in your
> `.knot` directory. This lets you swap the main configuration
> file while sharing the same `knotignore` and `knots.toml`
> files.
