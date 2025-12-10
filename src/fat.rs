use crate::{BlockDevice, Error, Fat32};

pub struct Fat<'a, D: BlockDevice> {
    pub fs: &'a mut Fat32<D>,
}

impl<D: BlockDevice> Fat32<D> {
    /// Lit une entrée de FAT32 (numéro de cluster → valeur FAT brute).
    pub fn read_fat_entry(&mut self, cluster: u32) -> Result<u32, Error> {
        let bytes_per_sector = self.boot.bytes_per_sector as u32;

        // Offset en octets de l'entrée dans la FAT
        let fat_offset = cluster * 4;
        let sector = self.fat_start_lba + (fat_offset / bytes_per_sector);
        let offset_in_sector = (fat_offset % bytes_per_sector) as usize;

        // On suppose des secteurs de 512 octets pour l'image de test
        let mut buf = [0u8; 512];
        self.device.read_sector(sector as u64, &mut buf)?;

        let entry_bytes = &buf[offset_in_sector..offset_in_sector + 4];
        let raw = u32::from_le_bytes([
            entry_bytes[0],
            entry_bytes[1],
            entry_bytes[2],
            entry_bytes[3],
        ]);

        // FAT32 utilise 28 bits significatifs
        Ok(raw & 0x0FFF_FFFF)
    }

    fn is_eoc(cluster: u32) -> bool {
        // End Of Chain
        cluster >= 0x0FFF_FFF8
    }

    /// Remplit `out` avec la chaîne de clusters à partir de `start`.
    pub fn cluster_chain(
        &mut self,
        start: u32,
        out: &mut alloc::vec::Vec<u32>,
    ) -> Result<(), Error> {
        let mut current = start;
        loop {
            out.push(current);
            let next = self.read_fat_entry(current)?;
            if Self::is_eoc(next) {
                break;
            }
            current = next;
        }
        Ok(())
    }
}
