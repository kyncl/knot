# CI/CD

Designing a terminal UI (TUI) application for CI/CD environments
is complex, but Knot is built to be as CI-friendly as possible.

Knot works out of the box in most CI/CD pipelines when
configured with specific flags and environment variables:

* **Non-interactive mode**: Commands designed for CI/CD
  environments support a non-interactive flag. Most likely
  the flag is called `--non-interactive`, but check the
  documentation for specific commands for usage details.

* **Environment variables**: To prevent visual artifacts in your
  build logs, configure the appropriate environment variables.
  Without them, the output functions may contain broken
  ANSI escape sequences. Set the following variables:
    - `CI=1`
    - `NO_COLOR=1`

> [!NOTE]
> CI/CD support is still under development. If you encounter
> any issues, please open a [GitHub issue](https://github.com/kyncl/knot/issues).
