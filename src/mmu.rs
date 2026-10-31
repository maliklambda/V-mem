use crate::page_table::{MemPage, PageTable};

/// Max mem-size of data-array managed by MMU.
/// Measured in Memory pages.
/// So total_size_in_bytes = MEM_SIZE * MEM_PAGE_SIZE
pub const MEM_SIZE: usize = 10;

/// Struct that emulates the MMU
#[derive(Debug, Default)]
pub struct MMU {
    /// Indicates whether or not the data-array is full.
    pub is_full: bool,

    /// Tracks the upper-most
    pub latest_free_spot: usize,
    pub data: [Option<MemPage>; MEM_SIZE],
    pub page_tables: Vec<PageTable>,
    // /// TODO: Add TLB-caching
    // tlb: TLB,
}

impl MMU {
    /// Allocates a new page-table.
    /// This is done during process initialization.
    pub fn allocate_pt(&mut self, pid: u32) -> Result<(), String> {
        if self.page_tables.iter().find(|pt| pt.pid == pid).is_some() {
            return Err(format!("PT for PID {pid} already exists."));
        }
        self.page_tables.push(PageTable::new(pid));
        Ok(())
    }

    /// Allocates a new memory page for process pid.
    /// Returns the physical address of the newly allocated mem page.
    pub fn allocate_mem_page(&mut self, pid: u32) -> Result<u32, String> {
        // find page-frame in RAM
        // Allocate the new page into free page frame
        let pf_id = self
            .find_free_slot()
            .ok_or(String::from("Cannot find free slot in memory."))?;

        // Update Page Table
        let pt: &mut PageTable = self
            .page_tables
            .iter_mut()
            .find(|pt| pt.pid == pid)
            .ok_or(format!("PID {pid} is not valid."))?;

        pt.add_mem_page(pf_id as u16)?;

        todo!("Alloc new mem page")
    }

    /// Finds a free page frame in RAM.
    /// Makes page-replacement-call if necessary.
    /// Returns the page frame id or none.
    fn find_free_slot(&mut self) -> Option<usize> {
        if self.is_full {
            todo!("Handle full RAM with paging.")
        }

        let pf_id = self.latest_free_spot;
        self.latest_free_spot += 1;
        if self.latest_free_spot == MEM_SIZE {
            self.is_full = true;
        }
        Some(pf_id)
    }
}

/// Struct to emulate the TLB
/// Stores cached v-address to p-address translation.
pub struct TLB {}
