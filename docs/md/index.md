<div class="homepage-header">
<div style="min-width:64px; margin-bottom: 19.3044px;">
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="100 100 600 600">
      <path fill="oklch(70.6% .222 41.116)" fill-rule="evenodd" d="M165.29 165.29H282.65V400H400V282.65H517.36V165.29H634.72V282.65H517.36V400H634.72V634.72H517.36V517.36H400V634.72H282.65V517.36H165.29V165.29ZM400 400H517.36V517.36H400Z"></path>
    </svg>
</div>

# Knot Documentation
</div>


Knot is a TUI/CLI application written in Rust for directory
synchronization.

Instead of manually managing unsynchronized versions of
directories across devices, Knot handles them for you using
**Knots**. A Knot is a logical unit representing a specific
directory and its connection, which you can manage through
an intuitive terminal interface.

[<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="square" stroke-linejoin="miter" aria-hidden="true" style="width: 16px; height:16px;">
<path d="M9.5 21v-3.5l1-1.5C7 15.5 5 14 5 10.5L6.5 8 6 4.5l3.5 1.5h5l3.5-1.5-.5 3.5 1.5 2.5c0 3.5-2 5-5.5 5.5l1 1.5V21"></path><path d="M9.5 18.5H7l-1.5-2L4 15.5"></path>
<rect x="8.5" y="9.75" width="1.75" height="2.25" fill="currentColor" stroke="none"></rect><rect x="13.75" y="9.75" width="1.75" height="2.25" fill="currentColor" stroke="none">
</rect></svg> GitHub page](https://github.com/kyncl/knot)

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
### Performance
* **Smart file batching:** Instead of sending files
  sequentially in arbitrary byte chunks, Knot groups small
  files into compressed batches. This drastically speeds up
  synchronization for codebases containing many small files.
* **[Caching](configuration/general-configuration.html#features):** Crawling large directories can be slow. Knot
  caches the directory structure, allowing it to bypass
  unchanged files and save time during subsequent checks.
* **[Compression](configuration/general-configuration.html#features):** Knot compresses file content to speed
  up transfers over slow connections.

### Networking
* **[SSH Support](configuration/knot-configuration.html#connection-configuration):** Knot supports remote synchronization via
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
  directory with multiple remote Knots.
* **[Private key authentication](configuration/knot-configuration.html#connection-configuration)**
* **[Smart credentials](configuration/knot-configuration.html#connection-configuration):** Quickly initialize a connection by
  providing a URI format (`type://username@host:port` see [Knot Connection Types](knots-and-their-types.html#knot-connection-types)).

### User Experience
* **[Ignore files](configuration/general-configuration.html#ignore-patterns):** Knot ignores
  patterns from `.gitignore` or a custom `knotignore` file.
* **[Custom behaviors](configuration/knot-configuration.html#behaviors):** Behaviors let you override default
  syncing rules for specific Knots.
* **[Archiving](knot-modes/archiving.html):** Unique remote files are archived, compressed and renamed.
  You can recover or delete them later.
* **System notification**
* **[Shell autocomplete](shell-completion.html)**
* **[Daemon mode](knot-modes/daemon.html)**
* **Password saving into OS keyring**
* **[Config files](configuration/index.html)**
* **Temporary files:** To prevent data corruption during
  transfers, Knot writes to temporary files first. If a sync
  fails, Knot recovers safely without altering original
  data.
> [!NOTE]
> Recover is not automatic. You must re-run Knot to clean up
> temporal files and sync rest of the files.

## Requirements
* OS (Linux, macOS or Windows)
* Terminal
* Electricity (optional)
* Nerdfonts

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
variable.

> [!WARNING]
> Windows Server is theoretically supported but remains untested.
> Active support is not currently available for Windows Server
> environments.

## Quick Start

Before you start, you must create configuration:

```bash
knot init
```

After initialization, you can run the sync command:

```bash
knot sync
# In case your config is located in different directory
knot sync -c /path/to/your/config
```
And that's it. You synced your files, congrats.

## License & Commercial Use

Knot is licensed under the **GPLv3**.

You are free to use, modify, and distribute the software,
provided you keep the source open and retain the original
attribution. If you use Knot for commercial projects, a
courtesy notice or clear attribution is highly appreciated.

You read it all? Here's a cat for you.<br>/ᐠ•⩊•マ
