#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod boot;
pub mod fat;
pub mod dir;
pub mod file;

use boot::BootSector;
use file::File;

/// Abstraction d'accès bloc → image disque, vrai disque, etc.
pub trait BlockDevice {
    fn read_sector(&mut self, lba: u64, buf: &mut [u8]) -> Result<(), Error>;
}

#[derive(Debug)]
pub enum Error {
    Io,
    InvalidFs,
    NotFound,
}

#[cfg(feature = "std")]
pub mod std_support {
    use std::fs::File as StdFile;
    use std::io::{Read, Seek, SeekFrom};

    use crate::{BlockDevice, Error};

    /// Implémentation de BlockDevice par-dessus un fichier d'image.
    pub struct StdBlockDevice {
        file: StdFile,
        pub sector_size: u64,
    }

    impl StdBlockDevice {
        pub fn open(path: &str, sector_size: u64) -> std::io::Result<Self> {
            let file = StdFile::open(path)?;
            Ok(Self { file, sector_size })
        }
    }

    impl BlockDevice for StdBlockDevice {
        fn read_sector(&mut self, lba: u64, buf: &mut [u8]) -> Result<(), Error> {
            let offset = lba * self.sector_size;

            self.file
                .seek(SeekFrom::Start(offset))
                .map_err(|_| Error::Io)?;

            self.file
                .read_exact(buf)
                .map_err(|_| Error::Io)?;

            Ok(())
        }
    }
}

/// Représente un FS FAT32 sur un BlockDevice.
pub struct Fat32<D: BlockDevice> {
    pub device: D,
    pub boot: BootSector,
    /// LBA de début de la première FAT
    pub fat_start_lba: u32,
    /// LBA de début de la zone data (cluster 2)
    pub data_start_lba: u32,
    /// Répertoire courant (cluster)
    pub cwd_cluster: u32,
}

impl<D: BlockDevice> Fat32<D> {
    /// Construit un Fat32 à partir d'un device (lit le secteur 0).
    pub fn new(mut device: D) -> Result<Self, Error> {
        let mut sector0 = [0u8; 512];
        device.read_sector(0, &mut sector0)?;

        let boot = BootSector::parse(&sector0)?;

        let fat_start_lba = boot.reserved_sectors as u32;
        let data_start_lba =
            fat_start_lba + (boot.num_fats as u32 * boot.sectors_per_fat);

        Ok(Self {
            device,
            boot,
            fat_start_lba,
            data_start_lba,
	    cwd_cluster: boot.root_cluster,
        })
    }

    pub fn bytes_per_sector(&self) -> u32 {
        self.boot.bytes_per_sector as u32
    }

    pub fn sectors_per_cluster(&self) -> u32 {
        self.boot.sectors_per_cluster as u32
    }

    /// Convertit un numéro de cluster en LBA du premier secteur de ce cluster.
    pub fn cluster_to_lba(&self, cluster: u32) -> u32 {
        self.data_start_lba + (cluster - 2) * self.sectors_per_cluster()
    }

    /// `resolve_path` est implémenté dans `dir.rs`
    /// pub fn resolve_path(&mut self, path: &str) -> Result<dir::DirEntry, Error>;

    /// Ouvre un fichier à partir de son chemin.
    pub fn open_file(&mut self, path: &str) -> Result<File<'_, D>, Error> {
        let entry = self.resolve_path(path)?;
        if entry.is_dir {
            return Err(Error::InvalidFs);
        }

        let mut chain = alloc::vec::Vec::new();
        // cluster_chain est implémenté dans fat.rs
        self.cluster_chain(entry.first_cluster, &mut chain)?;

        Ok(File::new(self, chain, entry.size))
    }

    /// Lit entièrement un fichier en mémoire.
    pub fn read_file(
        &mut self,
        path: &str,
    ) -> Result<alloc::vec::Vec<u8>, Error> {
        let mut f = self.open_file(path)?;
        f.read_to_end()
    }
}
