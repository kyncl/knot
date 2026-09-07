# Adding and Removing Properties

Use the `add` and `remove` commands to manage configuration
properties that accept multiple values.

**Example:**

```bash
knot add remote

# You can also specify a custom configuration path:
knot add remote -c path/to/different/configuration
```

Running the `add` command opens an interactive CLI prompt
to configure the new remote Knot. The `remove` command
supports the exact same flags.

## Supported Properties for `add`

You can append values to the following properties:

| Property | Description |
|---|---|
| `remote` | Adds a new remote Knot to the configuration. |

## Supported Properties for `remove`

You can delete values from the following properties:

| Property | Description |
|---|---|
| `remote` | Removes remote Knots via a multi-select prompt. |
