use crate::boot::BootSector;
use crate::file::File;
use alloc::vec::Vec;

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

// --- tes modules internes ---
pub mod boot;
pub mod fat;
pub mod dir;
pub mod file;

// --- Ton trait BlockDevice ---
pub trait BlockDevice {
    fn read_sector(&mut self, lba: u64, buf: &mut [u8]) -> Result<(), Error>;
}

#[derive(Debug)]
pub enum Error {
    Io,
    InvalidFs,
    
}

#[cfg(feature = "std")]
pub mod std_support {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};

    use crate::{BlockDevice, Error};

    pub struct StdBlockDevice {
        file: File,
        pub sector_size: u64,
    }

    impl StdBlockDevice {
        pub fn open(path: &str, sector_size: u64) -> std::io::Result<Self> {
            let file = File::open(path)?;
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

pub struct Fat32<D: BlockDevice> {
    pub device: D,
    pub boot: boot::BootSector,
    // plus tard : offsets, cache de FAT, etc.
}

impl<D: BlockDevice> Fat32<D> {
    pub fn open_file(&mut self, path: &str) -> Result<File<'_, D>, Error> {
        let entry = self.resolve_path(path)?;
        if !entry.is_dir {
            // construire la chaines de clusters + File
        } else {
            Err(Error::InvalidFs) // ou autre
        }
    }

    pub fn read_file(&mut self, path: &str) -> Result<alloc::vec::Vec<u8>, Error> {
        let mut f = self.open_file(path)?;
        f.read_to_end()
    }
}
