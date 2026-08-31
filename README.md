<div align="center">
<img src="docs/html/icon.svg" width="150" alt="Knot Icon">

<h1>Knot</h1>
</div>

**TUI/CLI application written in Rust for directory synchronization.**

Instead of manually managing unsynchronized versions of
directories across devices, Knot handles them for you using
**Knots**.

[![Build Status](https://github.com/kyncl/knot/workflows/CI/badge.svg?style=for-the-badge)](https://github.com/kyncl/knot/actions)

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg?style=for-the-badge)](https://www.gnu.org/licenses/gpl-3.0)
![supported platforms](https://img.shields.io/badge/platform-linux%20|%20windows%20|%20macos-success?style=for-the-badge)
![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg?style=for-the-badge)
[![Rust](https://img.shields.io/badge/Made%20with-Rust-orange.svg?style=for-the-badge)](https://www.rust-lang.org/)
[![Built With Ratatui](https://img.shields.io/badge/Built_With_Ratatui-000?logo=ratatui&logoColor=fff&style=for-the-badge)](https://ratatui.rs/)

[![Latest Release](https://img.shields.io/github/v/release/kyncl/knot?style=for-the-badge)](https://github.com/kyncl/knot/releases)
[![Open Issues](https://img.shields.io/github/issues/kyncl/knot?style=for-the-badge)](https://github.com/kyncl/knot/issues)
[![Last Commit](https://img.shields.io/github/last-commit/kyncl/knot?style=for-the-badge)](https://github.com/kyncl/knot)
[![Contributors](https://img.shields.io/github/contributors/kyncl/knot?style=for-the-badge)](https://github.com/kyncl/knot/graphs/contributors)

## Features
While Knot has a minimalist design, its underlying features
are highly capable and designed to speed up your workflow
significantly.

### Performance
* **Fast asynchronous algorithms:** Knot uses the Tokio and
  Rayon crates to execute CPU-bound and I/O operations
  concurrently. This makes synchronization significantly
  faster than synchronous alternatives.
* **Smart file batching:** Instead of sending files
  sequentially in arbitrary byte chunks, Knot groups small
  files into compressed batches. This drastically speeds up
  synchronization for codebases containing many small files.
* **Caching:** Crawling large directories can be slow. Knot
  caches the directory structure, allowing it to bypass
  unchanged files and save time during subsequent checks.
* **Compression:** Knot compresses file content to speed
  up transfers over slow connections. This trade-off
  increases CPU usage while reducing network payload size.

### Networking
* **SSH Support:** Knot supports remote synchronization via
  SSH. The `knot` binary must be installed on the remote
  device. On Unix, Knot checks for the binary in
  `~/.local/bin/knot` by default.
* **Connection pooling:** Knot distributes remote operations
  across multiple SSH sessions to increase throughput.

> [!WARNING]
> High connection limits might cause the remote server to
> block you, depending on its SSH daemon configuration.
> Verify your settings or consult the server administrator.

* **1:N Connections:** Knot can synchronize one source
  directory with multiple remote Knots simultaneously. A
  remote Knot can be another local directory or a separate
  SSH connection.
* **Private key authentication:** Knot fully supports SSH
  authentication via standard private keys or passwords.
* **Smart credentials:** Quickly initialize a connection by
  providing a URI format (e.g., `ssh://username@host:22`,
  `type://username@host:port` see Knot Connection Types).
  Knot handles the underlying configuration automatically.

### User Experience
* **CLI initialization:** Knot requires a configuration file
  to run. You can quickly generate this using the `knot init`
  command.
* **Ignore files:** Prevent sensitive or unnecessary files
  from syncing via Gitignore integration. Knot ignores
  patterns in `.gitignore` or a custom `knotignore` file.
* **Terminal interface:** Built with the Ratatui crate, the
  TUI makes it easy to navigate project data, resolve
  conflicts, and view archived files.
* **Custom behaviors:** Modify how Knot handles new files
  and resolves conflicts. Behaviors let you override default
  syncing rules for specific Knots.
* **Archiving:** Archive files to remove them from the
  source directory without permanent deletion. Archived
  files are compressed and stored on the remote Knot for
  later recovery.
* **System notifications:** Knot uses native OS
  notifications to alert you when background operations
  complete, freeing you from watching the terminal.
* **Shell autocomplete:** Autocomplete scripts are available
  for most major shells to accelerate command navigation.
* **Daemon mode:** A built-in file scanner runs continuously
  to detect directory changes. Synchronization starts
  automatically when changes occur.

> [!NOTE]
> The daemon scanner uses debouncing. It waits briefly
> after detecting a change to ensure file writes are
> complete before syncing.

* **Password saving:** Securely save passwords to your OS
  keyring to avoid re-entering them.
* **Config files:** Configurations are stored in the `.knot`
  folder. This allows for easy modification and version
  control of your sync settings across environments.
* **Temporary files:** To prevent data corruption during
  transfers, Knot writes to temporary files first. If a sync
  fails, Knot recovers safely without altering original
  data.

## Requirements
* A modern operating system (Linux, macOS, or Windows)
* A terminal emulator
* Internet or local network connection
* Electricity (optional)

## Installation
You can install Knot by downloading a pre-compiled release
from [GitHub](https://github.com/kyncl/knot/releases) or by
building the application from source.

### Building from Source
**Prerequisites:**
* Cargo (1.89+)
* Git
* Make (optional, for Unix)

#### Unix Systems
To build and install Knot on Unix systems, run the following
commands in your terminal:

```bash
git clone https://github.com/kyncl/knot.git
cd knot
make install

# To test the installation:
knot --version
```

#### Windows Systems

Currently, the Knot `Makefile` lacks Windows support.
However, you can compile Knot using Cargo:

```bash
cargo build --release

# To test the compiled binary directly:
.\target\release\knot.exe --version
```

After building, move the executable (.\target\release\knot.exe)
into a directory included in your system's PATH environment
variable. This allows you to run the knot command globally.

> [!WARNING]
> Windows Server is theoretically supported but remains untested.
> Active support is not currently available for Windows Server
> environments.

## Quick Start

Navigate to the directory you want to synchronize, then
create a configuration file by running:

```bash
knot init
```

After providing the requested values in the prompt, start
the synchronization process:

```bash
knot sync
```

# Documentation
If you need to check out documentation, you can! It's safe inside [docs](https://github.com/kyncl/knot/tree/main/docs) folder. 

The whole documentation is made by [Vault](https://github.com/kyncl/vault).

## License & Commercial Use

Knot is licensed under the **GPLv3**.

You are free to use, modify, and distribute the software,
provided you keep the source open and retain the original
attribution. If you use Knot for commercial projects, a
courtesy notice or clear attribution is highly appreciated.
