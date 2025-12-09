use crate::Error;            
use alloc::string::String;      

pub struct DirEntry {
    pub name: alloc::string::String,
    pub first_cluster: u32,
    pub size: u32,
    pub is_dir: bool,
}

fn parse_entry(entry: &[u8]) -> Option<DirEntry> {
    // 0x00 -> fin
    // 0xE5 -> supprimé
    // 0x0F -> LFN (à ignorer pour commencer)
    // sinon, c’est une entrée normale
}
