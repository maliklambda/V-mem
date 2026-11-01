use std::{
    fs::{File, OpenOptions}, io::{Seek, Write}, os::unix::fs::FileExt,
};

use crate::{
    page_table::{MemPage, MEMPAGE_SIZE}, v_addr::PageOffset,
};

pub const SWAP_FILE_NAME: &str = "swap.disk";

/// Struct to enable page swapping.
#[derive(Debug)]
pub struct Swap {
    /// Pages are flushed here
    pub swap_file: File,

    /// When a page is read back into memory,
    /// a free slot appears. To handle this
    /// fragementation, a freelist is needed.
    pub free_swap_slots: Vec<usize>,
}

impl Default for Swap {
    fn default() -> Self {
        let swap_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(SWAP_FILE_NAME)
            .unwrap();
        Self {
            swap_file,
            free_swap_slots: vec![],
        }
    }
}

impl Swap {
    /// Returns the offset on disk.
    /// Note that the offset is divided by the size of a mempage.
    pub fn swap_page_to_disk(&mut self, page: MemPage) -> Result<PageOffset, String> {
        // TODO: use free-swap-slots
        // -> Current implementation: append-only
        let pos = self
            .swap_file
            .seek(std::io::SeekFrom::End(0))
            .map_err(|err| format!("Seek failed: {err}"))?;
        self.swap_file
            .write_all(&page.data)
            .map_err(|err| format!("Write failed: {err}"))?;
        Ok(pos as PageOffset / MEMPAGE_SIZE as PageOffset)
    }

    /// Reads a memory page from disk.
    pub fn fetch_page_from_disk(&mut self, page_id: PageOffset
    ) -> Result<MemPage, String> {
        let mut buf = MemPage::default();
        let offset = (page_id as usize * MEMPAGE_SIZE)as u64;
        self.swap_file
            .read_at(&mut buf.data, offset)
            .map_err(|err| format!("Read failed: {err}"))?;
        Ok(buf)
    }
}
