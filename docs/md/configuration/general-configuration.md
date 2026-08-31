# General Configuration

Knot is highly configurable, allowing you to tailor the
synchronization process to your environment. If a setting
is missing, Knot is open-source—contributions are welcome.

The main configuration file covers these primary domains:
* **General:** Global settings, such as ignore patterns. Knot
  automatically reads patterns from `.knot/knotignore` using
  standard `.gitignore` syntax.
* **Performance:** Options to optimize synchronization speed,
  memory footprint, and resource usage. This refers to
  `config.performance` block.
* **Features:** Toggles for optional capabilities, such as
  caching, ignore files, and data compression. This refers
  to `config.features` block.
* **Source Knot:** Settings of [source Knot](#source-knot).
  This refers to `source` block.

An example of a complete main configuration file:

```toml
[config.performance]
size_limit = "15.00 GiB"
allow_size_limit = false

[config.features]
caching = true
gitignore = true
compress = false

[source]
type = "Local"
path = "path/to/source/knot"
```

## Ignore Patterns

By default, Knot reads the `.knot/knotignore` file. Use this
file to specify file and directory patterns you want to
exclude from synchronization. The syntax is identical to a
standard `.gitignore` file.

The default `.knot/knotignore` file automatically excludes
`.git` and `.knot*` patterns. This prevents accidental leaks
of sensitive configuration data. Excluding `.git` also avoids
transferring large Git histories into production environments.

## Performance

The `[config.performance]` block contains these properties:

| Property | Default | Type | Description |
|---|---|---|---|
| `allow_size_limit` | `false` | Boolean | Enables skipping files that exceed the `size_limit` threshold. |
| `size_limit` | `"15GiB"` | String | The maximum file size synced. Accepts standard byte formats (e.g., `"5G"`, `"15KB"`). |

## Features

The `[config.features]` block toggles application behaviors:

| Property | Default | Type | Description |
|---|---|---|---|
| `caching` | `true` | Boolean | Stores the directory structure from previous runs to accelerate subsequent syncs. |
| `gitignore` | `true` | Boolean | Reads `.gitignore` files in your workspace to automatically exclude matching files. |
| `compress` | `false` | Boolean | Compresses data during transfer. Improves speed on slow networks but increases CPU load. |

## Source Knot

The `[source]` block shares the same configuration schema as
remote Knots. Below are its primary properties:

| Property | Default | Type | Description |
|---|---|---|---|
| `type` | `"Local"` | Enum | Defines the connection type. See [Knot Configuration](knot-configuration.html). |
| `path` | `"./"` | String | The absolute or relative path to your source directory. |

> [!TIP]
> For more context on source Knots or connection types, refer
> to the [Knots and Their Types](../knots-and-their-types.html)
> chapter.
