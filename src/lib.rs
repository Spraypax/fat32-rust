#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod boot;
pub mod fat;
pub mod dir;
pub mod file;

// un trait pour l’accès disque
pub trait BlockDevice {
    fn read_sector(&mut self, lba: u64, buf: &mut [u8]) -> Result<(), Error>;
}

#[derive(Debug)]
pub enum Error {
    Io,
    InvalidFs,
}

// façade principale
pub struct Fat32<D: BlockDevice> {
    device: D,
    // boot info, offsets...
}
