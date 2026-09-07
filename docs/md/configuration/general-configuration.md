# General Configuration

Knot is highly configurable, allowing you to tailor the
synchronization process to your environment. If a setting
is missing, the project is open-source and contributions
are welcome.

The main configuration file covers these primary domains:

* **General:** Global settings like ignore patterns.
  Knot reads `.knot/knotignore` using standard `.gitignore`
  syntax.
* **Performance:** Options to optimize sync speed, memory
  footprint, and resource usage. Found in the
  `[config.performance]` block.
* **Features:** Toggles for optional capabilities like
  caching, ignore files, and data compression. Found in
  the `[config.features]` block.
* **Experimental:** Unstable features that speed up long
  operations but might break the UI or cause race
  conditions. Found in the `[config.experimental]` block.
* **Source Knot:** Settings for your source Knot connection.
  Found in the `[source]` block.

An example of a complete main configuration file:

```toml
[config.performance]
size_limit = "15.00 GiB"
allow_size_limit = false

[config.features]
caching = true
gitignore = true
compress = false

[config.experimental]
# Enables asynchronous synchronization.
# WARNING: This speeds up syncing with multiple Remote
# Knots, but it can break the UI and cause race conditions.
async_sync = false

[source]
type = "Local"
path = "path/to/source/knot"
```

## Ignore Patterns

By default, Knot reads the `.knot/knotignore` file. Use
this file to specify file and directory patterns to exclude
from synchronization. The syntax matches a standard
`.gitignore` file.

The default `.knot/knotignore` file automatically excludes
`.git` and `.knot*` patterns. This prevents accidental
leaks of sensitive configuration data. Excluding `.git`
also avoids transferring large Git histories into production
environments.

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
| `caching` | `true` | Boolean | Stores directory structures to accelerate future syncs. |
| `gitignore` | `true` | Boolean | Reads workspace `.gitignore` files to exclude matches. |
| `compress` | `false` | Boolean | Compresses transfer data. Improves speed on slow networks but increases CPU load. |

## Experimental

The `[config.experimental]` block configures unstable features:

| Property | Default | Type | Description |
|---|---|---|---|
| `async_sync` | `false` | Boolean | Syncs asynchronously. May break UI or cause race conditions. |

> [!CAUTION]
> Only use experimental features if you understand the
> risks. They can cause unwanted behavior or data state
> corruption.

## Source Knot

The `[source]` block shares the same configuration schema as
remote Knots. Below are its primary properties:

| Property | Default | Type | Description |
|---|---|---|---|
| `type` | `"Local"` | Enum | Connection type. See [Knot Configuration](knot-configuration.html). |
| `path` | `"./"` | String | The absolute or relative path to your source directory. |

> [!TIP]
> For more context on source Knots or connection types,
> refer to the [Knots and Their Types](../knots-and-their-types.html)
> chapter.
