# File System

The `file` command suite provides low-level, direct
manipulation of files and directories. These commands are
primarily used for internal operations, scripting, streaming
large payloads, and byte-level manipulation.

```bash
knot file <COMMAND> [OPTIONS]
```

## Read Operations

Commands for reading file contents, extracting byte ranges, and
streaming data to standard output (`stdout`).

### Read Full
Reads the entire contents of a file into memory.

```bash
knot file read-full <PATH>
```

* **`<PATH>`**: Target file path.
> [!CAUTION]
> Avoid using on large files to prevent Out-Of-Memory
> errors.

### Read Interval
Reads a specific byte range (chunk) from a target file.

```bash
knot file read-interval --start <START> --end <END> <PATH>
```

* **`<PATH>`**: Target file path.
* **`-s, --start <START>`**: Inclusive starting byte position.
* **`-e, --end <END>`**: Exclusive ending byte position.

### Read Stream
Reads data from a file and continuously streams it to `stdout`.

```bash
knot file read-stream <PATH>
```

* **`<PATH>`**: Target file path.

### Read Batch Stream
Accepts target file paths via standard input (`stdin`) and
streams each file's data back through `stdout`.

```bash
knot file read-batch-stream --root-path <ROOT_PATH> [OPTIONS]
```

* **`--root-path <ROOT_PATH>`**: Base directory containing the
  target files.
* **`--compression <COMPRESSION>`**: Compression type for the
  stream (`zstd` or `none`, default: `none`).

## Write Operations

Commands for injecting data, handling byte offsets, writing
payloads, and streaming from standard input (`stdin`).

### Write
Writes Base64-encoded raw bytes to a file. Supports writing
at a specific offset without truncating existing data.

```bash
knot file write --data <BASE64> [OPTIONS] <PATH>
```

* **`<PATH>`**: Target file path to modify.
* **`--data <BASE64>`**: Data payload encoded as Base64.
* **`-o, --offset <OFFSET>`**: Byte position to start writing
  at (default: `0`).

### Empty write
Overwrites a file entirely with the provided Base64 payload.

```bash
knot file empty-write --data <BASE64> <PATH>
```

* **`<PATH>`**: Target file path to overwrite.
* **`--data <BASE64>`**: Data payload encoded as Base64.

### Empty
Truncates an existing file to zero bytes, or creates a new
empty file if missing.

```bash
knot file empty <PATH>
```

* **`<PATH>`**: Path of the file to truncate or create.

### Write Stream
Streams data directly from `stdin` into a target file.

```bash
knot file write-stream [OPTIONS] <PATH>
```

* **`<PATH>`**: Destination file path.
* **`--temporal-path <PATH>`**: Temporary staging path for the
  incoming stream before moving to the target path.
* **`--expected-size <SIZE>`**: Known payload size for transfer
  validation.

### Write Batch Stream
Reads a tar-encoded stream from `stdin` and unpacks it into a
target root directory.

```bash
knot file write-batch-stream --root-path <ROOT_PATH> [OPTIONS]
```

* **`--root-path <ROOT_PATH>`**: Target root directory where
  batched files are unpacked.
* **`--compression <COMPRESSION>`**: Stream decompression algorithm
  (`zstd` or `none`, default: `none`).

## File & Directory Management

Standard utility commands for direct file tree manipulation.

### Create
Creates a new empty file.

```bash
knot file create <PATH>
```

* **`<PATH>`**: Path of the file to create.

### Create Directory
Creates a single directory.

```bash
knot file create-dir <PATH>
```

* **`<PATH>`**: Path of the directory to create.

### Create Directories
Creates multiple directory paths simultaneously.

```bash
knot file create-dirs --path <PATH>...
```

> [!TIP]
> If you need multiple directories, chain paths of each
> directories, like: `knot file create-dirs --path path/to/dir/1 --path path/to/dir/2`

* **`--path <PATH>`**: Target directory paths to create.

### Rename
Moves or renames a file or directory.

```bash
knot file rename <OLD_PATH> <NEW_PATH>
```

* **`<OLD_PATH>`**: Existing file or directory path.
* **`<NEW_PATH>`**: Destination path.

### Delete
Permanently deletes a file or directory.

```bash
knot file delete --path <PATH>
```

* **`--path <PATH>`**: Path of the target file or directory.
> [!NOTE]
> If the target path is a directory, it will be removed
> recursively, meaning all its children will be removed
> as well.
