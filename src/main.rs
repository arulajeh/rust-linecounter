use std::env;
use std::fs::{File, read_dir};
use std::io::Read;
use std::path::{Path, PathBuf};

use bytecount;

const FILE_EXT_LIST: [&str; 41] = [
    "txt", "text", "md", "markdown", "log",
    "rs", "py", "js", "ts", "java", "c", "cpp", "h", "hpp",
    "go", "rb", "php", "swift", "kt", "scala", "r",
    "html", "htm", "css", "scss", "sass", "less",
    "xml", "svg", "json", "yaml", "yml", "toml", "ini",
    "csv", "tsv", "sql", "sh", "bash", "conf", "config",
];

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        show_help();
        return;
    }

    let mut buffer_size = 8 * 1024;
    let mut skip_empty = false;
    let mut recursive = false;

    let target = &args[1];

    // Parse flags
    for arg in args.iter().skip(2) {
        if let Some(size) = arg.strip_prefix("--buffer-size=") {
            buffer_size = parse_buffer_size(size);
        } else if arg == "--skip-empty" {
            skip_empty = true;
        } else if arg == "--recursive" {
            recursive = true;
        }
    }

    let start = std::time::Instant::now();

    let path = Path::new(target);

    let total = if path.is_dir() {
        process_directory(path, buffer_size, skip_empty, recursive)
    } else {
        process_file(path, buffer_size, skip_empty)
    };

    println!("Total lines: {}", total);
    println!("Time taken: {:?}", start.elapsed());
}

fn show_help() {
    println!("Usage: linecount <path> [OPTIONS]\n");
    println!("Options:");
    println!("  --buffer-size=<KB>   Set buffer size (default: 8 KB)");
    println!("  --skip-empty         Skip empty lines");
    println!("  --recursive          Process directories recursively");
    println!("  --help, -h           Show help");
}

fn parse_buffer_size(s: &str) -> usize {
    match s.parse::<usize>() {
        Ok(kb) => kb * 1024,
        Err(_) => {
            eprintln!("Invalid buffer size. Using 8 KB.");
            8 * 1024
        }
    }
}

fn is_valid_ext(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => {
            let ext_lower = ext.to_ascii_lowercase();
            FILE_EXT_LIST.iter().any(|allowed| allowed == &ext_lower)
        }
        None => false,
    }
}

fn process_file(path: &Path, buffer_size: usize, skip_empty: bool) -> i32 {
    if !is_valid_ext(path) {
        return 0;
    }

    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Cannot open {}: {}", path.display(), e);
            return 0;
        }
    };

    if skip_empty {
        count_nonempty_lines(&mut file, buffer_size)
    } else {
        count_newlines_fast(&mut file, buffer_size)
    }
}

fn count_newlines_fast(file: &mut File, buffer_size: usize) -> i32 {
    let mut buffer = vec![0u8; buffer_size];
    let mut total = 0;

    loop {
        let n = match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        };

        total += bytecount::count(&buffer[..n], b'\n') as i32;
    }

    total
}

fn count_nonempty_lines(file: &mut File, buffer_size: usize) -> i32 {
    let mut buffer = vec![0u8; buffer_size];
    let mut total = 0;
    let mut has_data = false;

    loop {
        let n = match file.read(&mut buffer) {
            Ok(0) => {
                if has_data {
                    total += 1;
                }
                break;
            }
            Ok(n) => n,
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        };

        for &b in &buffer[..n] {
            match b {
                b'\n' => {
                    if has_data {
                        total += 1;
                    }
                    has_data = false;
                }
                b'\r' | b' ' | b'\t' => {}
                _ => has_data = true,
            }
        }
    }

    total
}

fn process_directory(
    path: &Path,
    buffer: usize,
    skip_empty: bool,
    recursive: bool,
) -> i32 {
    let mut total = 0;

    let walker: Box<dyn Iterator<Item = PathBuf>> = if recursive {
        Box::new(walk_recursive(path))
    } else {
        Box::new(walk_shallow(path))
    };

    for p in walker {
        if p.is_file() {
            total += process_file(&p, buffer, skip_empty);
        }
    }

    total
}

fn walk_shallow(path: &Path) -> impl Iterator<Item = PathBuf> {
    read_dir(path)
        .unwrap()
        .filter_map(|e| e.ok().map(|d| d.path()))
}

fn walk_recursive(root: &Path) -> impl Iterator<Item = PathBuf> {
    let mut stack = vec![root.to_path_buf()];

    std::iter::from_fn(move || {
        while let Some(path) = stack.pop() {
            if path.is_dir() {
                if let Ok(entries) = read_dir(&path) {
                    for entry in entries.flatten() {
                        stack.push(entry.path());
                    }
                }
                continue;
            }
            return Some(path);
        }
        None
    })
}