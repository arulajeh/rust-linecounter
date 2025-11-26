# Line Counter

A high-performance command-line utility for counting non-empty lines in text-based files. Built with Rust, this tool is optimized for speed, low memory usage, and reliable processing of large files.

## Features

- Efficient streaming using `BufReader`
- Counts only non-empty, non-whitespace lines
- Handles large files (GB-scale) without performance degradation
- Customizable buffer size through CLI flags
- Clear error handling for file access and I/O issues

## Usage

```bash
linecount <FILE_PATH> [--buffer-size=<BYTES>]
```

### Example
```bash
linecount logs/app.log --buffer-size=16384
```

Output:
```
Reading file: logs/app.log
Total non-empty lines: 102394
Time taken: 82.5ms
```

## Build

```bash
cargo build --release
```

The compiled binary will be available at:
```
target/release/linecount
```

## Notes

- This tool counts a line as “non-empty” if it contains any character other than whitespace.
- Supported file types depend on how you configure extension filtering in the source code; by default, common text-based extensions are allowed.

