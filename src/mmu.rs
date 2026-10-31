use crate::{
    page_table::{MemPage, PageTable, PtNodeEntry},
    v_addr::{PageOffset, VAddr, VAddrLower, VAddrLowerSplit, VAddrSplit},
};

/// Index to the memory-page-array
pub type PageFrameId = u16;

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
    pub latest_free_spot: PageFrameId,
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
    pub fn allocate_mem_page(&mut self, pid: u32) -> Result<VAddr, String> {
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

        let v_addr = pt.add_mem_page(pf_id)?;
        self.data[pf_id as usize] = Some(MemPage::default());
        Ok(v_addr)
    }

    /// Finds a free page frame in RAM.
    /// Makes page-replacement-call if necessary.
    /// Returns the page frame id or none.
    fn find_free_slot(&mut self) -> Option<PageFrameId> {
        if self.is_full {
            todo!("Handle full RAM with paging.")
        }

        let pf_id = self.latest_free_spot;
        self.latest_free_spot += 1;
        if self.latest_free_spot as usize == MEM_SIZE {
            self.is_full = true;
        }
        Some(pf_id)
    }

    /// Finds the PTE for a virtual address
    fn fetch_pte(&mut self, pid: u32, v_addr_lower: VAddrLower) -> Result<&PtNodeEntry, String> {
        // TODO: check cache

        let v_addr_split = VAddrLowerSplit::from(v_addr_lower);
        let pt: &mut PageTable = self
            .page_tables
            .iter_mut()
            .find(|pt| pt.pid == pid)
            .ok_or(format!("PID {pid} is not valid."))?;

        pt.get_pte(v_addr_split.join())
    }

    /// Finds the physical correspondent for a virtual address
    pub fn fetch_mem_page(
        &mut self,
        pid: u32,
        v_addr: VAddr,
    ) -> Result<(MemPage, PageOffset), String> {
        let v_addr_split = VAddrSplit::from(v_addr);
        let id = {
            let pte = self.fetch_pte(pid, v_addr_split.lower.join())?;
            println!("PTE: {:?}", pte);
            if !pte.p_bit {
                todo!("Fetch non-present memory page");
            }
            pte.page_frame_id as usize
        };
        let page = self.data[id].ok_or(format!("Memory page is missing for: {id}."))?;
        Ok((page, v_addr_split.offset))
    }
}

/// Struct to emulate the TLB
/// Stores cached v-address to p-address translation.
pub struct TLB {}
