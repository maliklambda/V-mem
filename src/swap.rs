use std::fs::{File, OpenOptions};

pub const SWAP_FILE_NAME: &str = "swap.bin";

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
