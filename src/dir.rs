use crate::{BlockDevice, Error, Fat32};
use alloc::{string::String, vec::Vec};
use core::str;

pub struct DirEntry {
    pub name: String,
    pub first_cluster: u32,
    pub size: u32,
    pub is_dir: bool,
}

fn parse_short_name(raw: &[u8; 11]) -> String {
    let base = str::from_utf8(&raw[0..8]).unwrap_or("").trim_end();
    let ext = str::from_utf8(&raw[8..11]).unwrap_or("").trim_end();

    if ext.is_empty() {
        base.to_string()
    } else {
        let mut s = String::with_capacity(base.len() + 1 + ext.len());
        s.push_str(base);
        s.push('.');
        s.push_str(ext);
        s
    }
}

/// Parse une entrée de 32 octets (short name, pas de LFN).
fn parse_entry(entry: &[u8]) -> Option<DirEntry> {
    if entry.len() < 32 {
        return None;
    }

    let first_byte = entry[0];

    // 0x00 → fin du répertoire
    if first_byte == 0x00 {
        return None;
    }

    // 0xE5 → entrée supprimée
    if first_byte == 0xE5 {
        return None;
    }

    // 0x0F dans les attributs → Long File Name (qu’on ignore)
    if entry[11] == 0x0F {
        return None;
    }

    let mut name_raw = [0u8; 11];
    name_raw.copy_from_slice(&entry[0..11]);
    let name = parse_short_name(&name_raw);

    let attr = entry[11];
    let is_dir = (attr & 0x10) != 0;

    let first_cluster_high =
        u16::from_le_bytes([entry[20], entry[21]]) as u32;
    let first_cluster_low =
        u16::from_le_bytes([entry[26], entry[27]]) as u32;
    let first_cluster = (first_cluster_high << 16) | first_cluster_low;

    let size = u32::from_le_bytes([
        entry[28],
        entry[29],
        entry[30],
        entry[31],
    ]);

    Some(DirEntry {
        name,
        first_cluster,
        size,
        is_dir,
    })
}

impl<D: BlockDevice> Fat32<D> {
    /// Lit toutes les entrées d’un répertoire à partir de son premier cluster.
    pub fn read_dir_cluster(
        &mut self,
        first_cluster: u32,
    ) -> Result<Vec<DirEntry>, Error> {
        let mut entries = Vec::new();

        let mut chain = alloc::vec::Vec::new();
        self.cluster_chain(first_cluster, &mut chain)?;

        let bytes_per_sector = self.bytes_per_sector() as usize;
        let sectors_per_cluster = self.sectors_per_cluster() as usize;
        let bytes_per_cluster = bytes_per_sector * sectors_per_cluster;

        let mut buf = Vec::with_capacity(chain.len() * bytes_per_cluster);
        buf.resize(chain.len() * bytes_per_cluster, 0);

        let mut offset = 0;
        for &cluster in &chain {
            let first_lba = self.cluster_to_lba(cluster);
            for s in 0..sectors_per_cluster {
                let slice = &mut buf[offset..offset + bytes_per_sector];
                self.device
                    .read_sector((first_lba + s as u32) as u64, slice)?;
                offset += bytes_per_sector;
            }
        }

        let mut i = 0;
        while i + 32 <= buf.len() {
            let entry_bytes = &buf[i..i + 32];

            if entry_bytes[0] == 0x00 {
                break; // fin des entrées
            }

            if let Some(e) = parse_entry(entry_bytes) {
                entries.push(e);
            }

            i += 32;
        }

        Ok(entries)
    }

    /// Liste le répertoire racine.
    pub fn list_root(&mut self) -> Result<Vec<DirEntry>, Error> {
        self.read_dir_cluster(self.boot.root_cluster)
    }

    /// Liste le répertoire courant.
    pub fn list_cwd(&mut self) -> Result<Vec<DirEntry>, Error> {
        self.read_dir_cluster(self.cwd_cluster)
    }

    /// Résout un chemin **à partir d’un cluster de départ**.
    fn resolve_from_cluster(
        &mut self,
        start_cluster: u32,
        path: &str,
    ) -> Result<DirEntry, Error> {
        let mut current_cluster = start_cluster;
        let mut last_entry = None;

        for part in path.split('/').filter(|p| !p.is_empty()) {
	    // .  => ne rien faire
	    if part == "." {
		continue;
	    }

	    // .. => remonter (via l’entrée ".." du répertoire)
	    if part == ".." {
		let entries = self.read_dir_cluster(current_cluster)?;
		let mut parent = None;
		for e in entries {
		    if e.name == ".." {
		        parent = Some(e);
		        break;
		    }
		}
		let p = parent.ok_or(Error::NotFound)?;
		current_cluster = p.first_cluster;
		last_entry = Some(p);
		continue;
	    }

	    let name = part;

        last_entry.ok_or(Error::NotFound)
    }

    /// Résout un chemin :
    /// - commençant par `/` → depuis la racine
    /// - sinon → depuis le répertoire courant (`cwd_cluster`)
    pub fn resolve_path(&mut self, path: &str) -> Result<DirEntry, Error> {
        if path.is_empty() || path == "/" {
            return Err(Error::InvalidFs); // la racine n’est pas un fichier
        }

        if path.starts_with('/') {
            self.resolve_from_cluster(self.boot.root_cluster, path)
        } else {
            self.resolve_from_cluster(self.cwd_cluster, path)
        }
    }

    /// Change le répertoire courant (comme `cd`).
    pub fn change_dir(&mut self, path: &str) -> Result<(), Error> {
        // chemins relatifs/absolus gérés par resolve_path
        let entry = if path == "/" {
            // cas spécial : retourner à la racine
            DirEntry {
                name: "/".to_string(),
                first_cluster: self.boot.root_cluster,
                size: 0,
                is_dir: true,
            }
        } else {
            self.resolve_path(path)?
        };

        if !entry.is_dir {
            return Err(Error::InvalidFs);
        }

        self.cwd_cluster = entry.first_cluster;
        Ok(())
    }
}
