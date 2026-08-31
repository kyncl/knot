# Knot
**TUI utility for synchronizing multiple folders across all your devices.**

Knot helps you tie your data together across multiple devices. 
Instead of messy configurations, manage your synchronized directories 
(**Knots**) through an intuitive terminal interface.

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

---

# Install/Build 
**Requirements:**
- Cargo (1.89+)
- Make

```bash
git clone git@github.com:kyncl/knot.git 
cd knot
make install
```

# Knots
While doing configuration you will have to create some knots. We have primarily two version, source and remote.
Source is in 99% you current working device and remote is probably your server. Here comes the biggest problem.
Source knot can have any OS, as long as you have a compiled version. This should work on remote too, but the whole
communication is right now only tested on remote knot running Linux.

Is this pointless note? Most likely, but it's for people, who will open issues about, how this project 
doesn't work on windows servers through SSH. Right now this project isn't at stage, where I can make this work sorry.

Thanks and have fun <3.

# License & Commercial Use
Knot is licensed under the **GPLv3**. 

You can use it, tweak it, and share it—just keep the source open and keep 
my name on it. If you’re using this for commercial projects or making 
a profit from it, I’d appreciate a heads-up or clear attribution.
