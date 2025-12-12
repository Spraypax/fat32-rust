use crate::{BlockDevice, Error, Fat32};
use alloc::vec::Vec;
use core::cmp;

pub struct File<'fs, D: BlockDevice> {
    pub(crate) fs: &'fs mut Fat32<D>,
    pub(crate) chain: Vec<u32>,
    pub(crate) size: u32,
    pub(crate) cursor: u64,
}

impl<'fs, D: BlockDevice> File<'fs, D> {
    pub fn new(fs: &'fs mut Fat32<D>, chain: Vec<u32>, size: u32) -> Self {
        Self {
            fs,
            chain,
            size,
            cursor: 0,
        }
    }

    /// Lit jusqu'à buf.len() octets à partir de la position courante.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if buf.is_empty() {
            return Ok(0);
        }

        let file_size = self.size as u64;
        if self.cursor >= file_size {
            return Ok(0); // EOF
        }

        let remaining = (file_size - self.cursor) as usize;
        let to_read = cmp::min(buf.len(), remaining);

        let bytes_per_sector = self.fs.bytes_per_sector() as usize;
        let sectors_per_cluster = self.fs.sectors_per_cluster() as usize;
        let bytes_per_cluster = bytes_per_sector * sectors_per_cluster;

        let mut remaining_to_read = to_read;
        let mut written = 0;
        let mut pos_in_file = self.cursor as usize;

        while remaining_to_read > 0 {
            let cluster_index = pos_in_file / bytes_per_cluster;
            let offset_in_cluster = pos_in_file % bytes_per_cluster;

            if cluster_index >= self.chain.len() {
                break;
            }

            let cluster = self.chain[cluster_index];
            let first_lba = self.fs.cluster_to_lba(cluster);

            // Lire tout le cluster en mémoire
            let mut cluster_buf = vec![0u8; bytes_per_cluster];

            let mut off = 0;
            for s in 0..sectors_per_cluster {
                let slice = &mut cluster_buf[off..off + bytes_per_sector];
                self.fs
                    .device
                    .read_sector((first_lba + s as u32) as u64, slice)?;
                off += bytes_per_sector;
            }

            let available_in_cluster = bytes_per_cluster - offset_in_cluster;
            let to_copy = cmp::min(available_in_cluster, remaining_to_read);

            buf[written..written + to_copy]
                .copy_from_slice(&cluster_buf[offset_in_cluster..offset_in_cluster + to_copy]);

            written += to_copy;
            remaining_to_read -= to_copy;
            pos_in_file += to_copy;
        }

        self.cursor += written as u64;
        Ok(written)
    }

    pub fn read_to_end(&mut self) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();
        let mut tmp = [0u8; 1024];
        loop {
            let n = self.read(&mut tmp)?;
            if n == 0 {
                break;
            }
            out.extend_from_slice(&tmp[..n]);
        }
        Ok(out)
    }
}
