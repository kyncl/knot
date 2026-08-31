# Getting Started
Before you begin, ensure `knot` is installed on your local
machine. If you plan to use SSH communication, `knot` must
also be installed on the remote device. You will also need
a designated working directory.

If both your local and remote devices run Linux, Knot can
automatically copy the executable to the remote machine.
During synchronization, Knot checks if the binary exists
on the remote device. If it is missing, Knot prompts you
for the local executable path and copies it over for you.

> [!WARNING]
> This feature is **Linux only**, as it relies on native
> ecosystem tools. Attempting this on other operating
> systems may crash the program.

Navigate to your working directory and initialize your
project:
```bash
knot init
```

This command generates your configuration files inside a
`.knot` directory by default. For advanced setups, see the
[Configuration](configuration/index.html) chapter. This
folder contains:

* **`config.toml`:** The main configuration file containing
  general options and the source Knot settings.
* **`knotignore`:** Custom ignore patterns. The syntax is
  identical to a standard `.gitignore` file.
* **`knots.toml`:** Configuration settings for your remote
  Knots. These share the same properties as the source.

After initialization, run your first synchronization:

```bash
knot sync
```

This executes the synchronization process between your
source directory and your configured Knots.

> [!TIP]
> If you need assistance, run `knot help` or `knot --help`
> to view all available commands and flags.
