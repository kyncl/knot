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

<video src="https://github.com/user-attachments/assets/90a7ecb1-286b-4901-bbb3-b8edc213fe87" width="600" controls></video>
<video src="https://github.com/user-attachments/assets/63b0ca00-98e5-42b5-9c82-9c4bc39217b5" width="600" controls></video>

## Features
If you want to read about Knot's features, check out [docs homepage](docs/md/index.md)

## Requirements
* OS (Linux, macOS or Windows)
* Terminal
* Electricity (optional)
* Nerdfonts (optional, you'll see weird symbols otherwise)

## Installation
You can install Knot by downloading a pre-compiled release
from [GitHub](https://github.com/kyncl/knot/releases) or by
building the application from source.

### Building from Source
**Prerequisites:**
* Cargo (1.89+)
* Git
* Make (optional)

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

# Documentation
If you need to check out documentation, you can find it inside [docs](https://github.com/kyncl/knot/tree/main/docs) folder. 

The whole documentation is made by [Vault](https://github.com/kyncl/vault).

## License & Commercial Use

Knot is licensed under the **GPLv3**.

You are free to use, modify, and distribute the software,
provided you keep the source open and retain the original
attribution. If you use Knot for commercial projects, a
courtesy notice or clear attribution is highly appreciated.

You read it all? Here's a cat for you.

/ᐠ•⩊•マ
