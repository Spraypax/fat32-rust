use fat32_rust::{Fat32, std_support::StdBlockDevice};
use std::io::{self, Write};

fn main() {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  cli <image> shell");
        eprintln!("  cli <image> ls [path]");
        eprintln!("  cli <image> cat <path>");
        return;
    }

    let image = args.remove(0);
    let cmd = args.remove(0);

    let dev = StdBlockDevice::open(&image, 512).expect("open image failed");
    let mut fs = Fat32::new(dev).expect("init fat32 failed");

    match cmd.as_str() {
        "shell" => shell(&mut fs),
        "ls" => {
            let path = args.first().map(|s| s.as_str()).unwrap_or("/");
            cmd_ls(&mut fs, path);
        }
        "cat" => {
            let path = args.first().expect("cat needs a path");
            cmd_cat(&mut fs, path);
        }
        _ => eprintln!("Unknown command: {cmd}"),
    }
}

fn shell<D: fat32_rust::BlockDevice>(fs: &mut Fat32<D>) {
    let stdin = io::stdin();
    loop {
        print!("fat32> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        if stdin.read_line(&mut line).is_err() {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut it = line.split_whitespace();
        let cmd = it.next().unwrap();

        match cmd {
            "exit" | "quit" => break,
            "pwd" => println!("(cwd cluster = {})", fs.cwd_cluster),
            "ls" => {
                let path = it.next().unwrap_or(".");
                cmd_ls(fs, path);
            }
            "cd" => {
                let path = it.next().unwrap_or("/");
                match fs.change_dir(path) {
                    Ok(_) => {}
                    Err(e) => eprintln!("cd error: {:?}", e),
                }
            }
            "cat" => {
                if let Some(path) = it.next() {
                    cmd_cat(fs, path);
                } else {
                    eprintln!("cat needs a path");
                }
            }
            _ => eprintln!("Commands: ls, cd, cat, pwd, exit"),
        }
    }
}

fn cmd_ls<D: fat32_rust::BlockDevice>(fs: &mut Fat32<D>, path: &str) {
    if path == "/" {
        let entries = fs.list_root().expect("list_root failed");
        for e in entries {
            if e.is_dir {
                println!("<DIR>  {}", e.name);
            } else {
                println!("       {}", e.name);
            }
        }
        return;
    }

    if path == "." {
        let entries = fs.list_cwd().expect("list_cwd failed");
        for e in entries {
            if e.is_dir {
                println!("<DIR>  {}", e.name);
            } else {
                println!("       {}", e.name);
            }
        }
        return;
    }

    let entry = match fs.resolve_path(path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("ls: {:?} ({path})", e);
            return;
        }
    };

    if !entry.is_dir {
        println!("{}", entry.name);
        return;
    }

    let entries = fs
        .read_dir_cluster(entry.first_cluster)
        .expect("read_dir_cluster failed");

    for e in entries {
        if e.is_dir {
            println!("<DIR>  {}", e.name);
        } else {
            println!("       {}", e.name);
        }
    }
}

fn cmd_cat<D: fat32_rust::BlockDevice>(fs: &mut Fat32<D>, path: &str) {
    let data = match fs.read_file(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("cat: {:?} ({path})", e);
            return;
        }
    };
    print!("{}", String::from_utf8_lossy(&data));
}
