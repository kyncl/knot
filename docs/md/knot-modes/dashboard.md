# Dashboard

Use Dashboard mode to visualize all files across your source
folder and Knot directories. This bypasses the visibility
limits of the Standard Resolver and Synchronization Report.

To start the dashboard, run:

```bash
knot dashboard

# Use a custom configuration path
knot dashboard --config-path path/to/config

# Or use the short flag
knot dashboard -c path/to/config
```

This command launches a terminal user interface (TUI) dashboard.
Use it to view synchronized, unique, conflicting or archived files.

## Actions

You can synchronize a specific Knot directly with the source.
This action generates a synchronization report highlighting
conflicting or unique files. Press `S` to start this process.

## Navigation

Use the following keyboard controls to navigate the dashboard:

- **Arrows**: Move between tabs, source and remote sections.
  On a selected directory, press `Right Arrow` to expand and
  `Left Arrow` to collapse.
- **Enter**: Select a section or toggle child directories.
- **Esc**: Deselect the active section.
- **Page Down / Space**: Jump down within a selected section.
- **Page Up**: Jump up within a selected section.
- **S**: Start synchronization on the selected Knot.
- **Q** or **Ctrl+Esc**: Quit the application.

> [!TIP]
> When highlighting the source or remote section, press
> `Down Arrow` to automatically select the folder.
