use fat32_rust::{Fat32, std_support::StdBlockDevice};

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  cli <image> ls [path]");
    eprintln!("  cli <image> cat <path>");
    eprintln!();
    eprintln!("Exemples :");
    eprintln!("  cli images/test_fat32.img ls /");
    eprintln!("  cli images/test_fat32.img cat /README.TXT");
}

fn main() {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();

    if args.len() < 2 {
        print_usage();
        return;
    }

    let image_path = args.remove(0);
    let cmd = args.remove(0);

    // Ouvre l'image disque en BlockDevice
    let dev = StdBlockDevice::open(&image_path, 512)
        .expect("Impossible d'ouvrir l'image disque");
    let mut fs = Fat32::new(dev)
        .expect("Impossible d'initialiser le FS FAT32");

    match cmd.as_str() {
        "ls" => {
            // chemin facultatif, par défaut "/"
            let path = if !args.is_empty() {
                args.remove(0)
            } else {
                "/".to_string()
            };

            if path == "/" {
                // liste la racine
                let entries = fs.list_root().expect("Erreur list_root()");
                for e in entries {
                    if e.is_dir {
                        println!("<DIR>  {}", e.name);
                    } else {
                        println!("       {}", e.name);
                    }
                }
            } else {
                // on résout le chemin et on liste si c'est un dossier
                let entry = fs
                    .resolve_path(&path)
                    .expect("Chemin introuvable");

                if !entry.is_dir {
                    println!("{}", entry.name);
                } else {
                    let entries = fs
                        .read_dir_cluster(entry.first_cluster)
                        .expect("Erreur lecture répertoire");
                    for e in entries {
                        if e.is_dir {
                            println!("<DIR>  {}", e.name);
                        } else {
                            println!("       {}", e.name);
                        }
                    }
                }
            }
        }

        "cat" => {
            if args.is_empty() {
                eprintln!("cat nécessite un chemin de fichier");
                print_usage();
                return;
            }

            let path = args.remove(0);

            let data = fs
                .read_file(&path)
                .expect("Erreur de lecture du fichier");

            // on affiche en supposant du texte
            print!("{}", String::from_utf8_lossy(&data));
        }

        _ => {
            eprintln!("Commande inconnue: {cmd}");
            print_usage();
        }
    }
}
