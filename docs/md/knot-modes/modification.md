# Modification

The `knot modify` command suite allows you to update your
Knot configuration directly from the command line, removing
the need to manually edit the configuration file.

## Global Options

The base modify command accepts global options to specify
which configuration file to target:

* `-c, --config-path <PATH>`: Path to a specific
  configuration file or workspace folder.

---

## General Configuration

Use these subcommands to adjust global sync settings and
performance features:

* `caching`: Configure caching feature settings.
* `compression`: Configure file compression feature settings.
* `gitignore`: Configure `.gitignore` integration settings.
* `ignore-patterns`: Manage global ignore patterns (append or
  rewrite).
* `allow-size-limit`: Enable or disable the file size limit.
* `size-limit`: Set or update the maximum file size limit.

---

## Modifying Knots

You can modify properties for both your local source Knot and
your configured remote Knots.

The available subcommands for both are identical. However,
when modifying a remote Knot, you must specify the target
Knot using its index.

```bash
# Modifying the local source Knot
knot modify source <COMMAND>

# Modifying a specific remote Knot
knot modify remote -i <INDEX> <COMMAND>
```

### Connection & Path Settings

* `type`: Set the adapter type (e.g., `Local`, `SSH`).
* `path`: Update the target directory or remote path.
* `connections`: Set the maximum concurrent connection limit.

### Credentials & Network

* `credentials`: Interactively set or update authentication
  credentials via a prompt.
* `username`: Update the network authentication username.
* `host`: Update the target host address or domain name.
* `port`: Update the network connection port.
* `auth`: Change your authentication method (e.g., `Password`,
  `PrivateKey`).
* `password`: Rewrite or delete your securely stored password.

### Sync Behaviors

* `unique-behavior`: Update how the Knot handles newly
  discovered files existing on only one side. Attempting to
  run this on the source Knot returns an error, as behaviors
  apply only to remote Knots.
* `conflict-behavior`: Update how the Knot resolves files with
  the same path but differing content. Attempting to run this
  on the source Knot returns an error, as behaviors apply only
  to remote Knots.
