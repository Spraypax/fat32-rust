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
