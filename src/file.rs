use crate::{BlockDevice, Fat32, Error};
use alloc::vec::Vec;

pub struct File<'fs, D: BlockDevice> {
    fs: &'fs mut Fat32<D>,
    chain: alloc::vec::Vec<u32>,
    size: u32,
    cursor: u64,
}

impl<'fs, D: BlockDevice> File<'fs, D> {
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        // calculer quelle partie de quels clusters lire
        // mettre au max `buf.len()` octets
        // mettre à jour cursor
        todo!()
    }

    pub fn read_to_end(&mut self) -> Result<alloc::vec::Vec<u8>, Error> {
        let mut v = alloc::vec::Vec::new();
        // boucle read jusqu’à EOF
        todo!()
    }
}
