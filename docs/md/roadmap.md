# Roadmap
This roadmap details planned features, architecture
improvements, and connection adapters for Knot.

> [!NOTE]
> These items are planned for future releases and are not yet
> implemented.

## Core Features & CLI

* [ ] **Delta Sync:** Implement byte-level differential
  transfers to send only changed file segments rather than full
  files.
* [X] **Remote Command:** Add a dedicated command to attach
  new remote targets (e.g., `knot add remote`).
* [X] **Init Refactor:** Improve the `knot init` wizard to use
  URI strings, eliminating redundant prompts for connection
  types.

> [!NOTE]
> When configuring a source Knot, the CLI displays a connection
> type menu. For remote Knots, it asks for credential string.
> Missing values are given by sequentially prompting for type,
> username and port (host must be give in the credential string).
>
> Typically, configuring a source requires two prompts, while
> each remote Knot requires four. You can skip non-essential
> prompts by applying default settings for general options.
> If you have ideas on how to improve this flow, pull requests
> are welcome on GitHub.

* [ ] **Path Variables:** Support environment and path
  variables directly inside configuration files.
* [ ] **Template Generation:** Allow users to generate
  configurations from predefined templates.
* [ ] **Pre/Post SSH Execution:** Support running custom shell
  commands before or after a synchronization sequence.
* [ ] **Passphrase Caching:** Implement secure caching for
  private key passphrases to avoid repeated prompts.
* [ ] **Safer password management:** Adding security features, like
  Zeroize for better security. Currently this shouldn't be an issue,
  because no password or any sensitive data are transferred between
  devices. Only way this would be useful, is when your device is
  already compromised. In that case you have bigger problems, but
  still I think it's great touch.
* [ ] **Remote Daemon Polling:** Expand Daemon mode to monitor
  remote Knots for changes using an extended polling delay.

## Adapters & Networking

* [ ] **SFTP Adapter:** Add native support for standard SFTP
  connections.
* [ ] **P2P Adapter:** Add a peer-to-peer connection adapter
  for decentralized syncing.
* [ ] **Alternative Topologies:** Support advanced
  synchronization models (e.g., mesh networks) beyond the
  star topology.

## Interfaces (UI/UX)

* [X] **TUI Dashboard:** Provide a terminal dashboard for
  managing Knot operations and monitoring sync status.

> [!NOTE]
> Because of Rust compiler bug, you cannot synchronize only in
> TUI (you must jump out of the TUI for the process). This bug
> cannot be fixed by Knot, because it's Rust compiler issue.

* [ ] **TUI Dashboard file manipulation:** There should be an option to
  manually select file for synchronization/deletion/archiving.
* [ ] **Graphical Interfaces:** Build dedicated desktop and
  mobile GUI applications.
* [ ] **Dynamic Key Locator:** Automate discovery of SSH
  private keys on the host system.

## Architecture & Testing

* [ ] **Workspace Refactoring:** Split the monolithic codebase
  into smaller, modular Rust crates.
* [ ] **System Logging:** Add comprehensive internal logging
  for debugging and audit trails.
* [ ] **Experimental Builds:** Provide experimental builds
  optimized for maximum performance with unsafe techniques.
  This won't change the whole project. Instead there will be
  new branch.
* [ ] **Expanded Test Coverage:** Add unit tests for SSH and
  archiving systems.
