use crate::{BlockDevice, Fat32, Error};
use alloc::vec::Vec;

pub struct Fat<'a, D: BlockDevice> {
    pub fs: &'a mut Fat32<D>,
}

impl<D: BlockDevice> Fat32<D> {
    pub fn read_fat_entry(&mut self, cluster: u32) -> Result<u32, Error> {
        // calculer l’offset dans la FAT:
        // entry = cluster * 4 (FAT32)
        // secteur = fat_start + entry / bytes_per_sector
        // offset = entry % bytes_per_sector
        // etc.
        todo!()
    }

fn is_eoc(cluster: u32) -> bool {
    cluster >= 0x0FFFFFF8
}

    pub fn cluster_chain(&mut self, start: u32, out: &mut alloc::vec::Vec<u32>) -> Result<(), Error> {
        let mut current = start;
        loop {
            out.push(current);
            let next = self.read_fat_entry(current)?;
            if is_eoc(next) { // end of chain
                break;
            }
            current = next;
        }
        Ok(())
    }
}
