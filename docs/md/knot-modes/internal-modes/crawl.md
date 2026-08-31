# Crawl

The `knot crawl` command scans a local directory and outputs
its internal file structure. While primarily used internally
by Knot during synchronization, it is also useful for
programmatic inspection, debugging sync scopes, and generating
directory tree snapshots.

```bash
knot crawl
```

## Options

### Target & Filtering

Use these options to define the scan location and exclude
specific paths:

* `-p, --crawl-path <PATH>`: The target directory path to
  crawl. (Defaults to `./`).
* `-s, --size <SIZE>`: Skip files exceeding a specified size
  limit (e.g., `500MB`, `2GB`).
* `-g, --gitignore`: Respect `.gitignore` rules during the
  scan, omitting ignored files.
* `--ignore-patterns <PATTERNS>`: Provide additional custom
  file or directory pattern to ignore during the crawl. In case
  of multiple patterns, each pattern should have its own 
  `--ignore-patterns` flag.

### Output Formatting

By default, Knot prints the directory structure directly to
stdout. You can format or compress this output for use in
external scripts.

* `--format <FORMAT>`: The serialization format for the
  output. Accepts `json` or `binary` (encoded in Base64).
  (Defaults to `binary`).
* `--compress`: Compress the resulting output structure.

### Performance

* `-c, --caching`: Enable caching of scan results to speed up
  subsequent, repeated crawls.
