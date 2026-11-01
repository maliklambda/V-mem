use std::io::Seek;

use crate::{
    page_table::{MemPage, PageTable, PtNodeEntry},
    pra::PraHelper,
    swap::Swap,
    v_addr::{PageOffset, VAddr, VAddrLower, VAddrLowerSplit, VAddrSplit},
};

/// Index to the memory-page-array
pub type PageFrameId = u16;

/// Max mem-size of data-array managed by MMU.
/// Measured in Memory pages.
/// So total_size_in_bytes = MEM_SIZE * MEM_PAGE_SIZE
pub const MEM_SIZE: usize = 2;

/// Struct that emulates the MMU
#[derive(Debug, Default)]
pub struct MMU {
    /// Indicates whether or not the data-array is full.
    pub is_full: bool,

    /// Tracks the upper-most pf-id
    pub latest_free_spot: PageFrameId,

    /// The rest of the data (i.e. memory pages)
    pub data: [Option<MemPage>; MEM_SIZE],

    /// Hold a reference to a page table for each process.
    pub page_tables: Vec<PageTable>,

    /// Helper variables for the PRA
    pub page_replacement_helper: PraHelper,

    /// Keep track of pages swapped out to disk.
    /// This would usually be held by the kernel.
    /// But since we do not have one here, the MMU keeps track of all swaps.
    pub swap: Swap,
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
        // This calls the PRA if no free slot is there.
        let pf_id = self.find_free_slot()?;
        println!("Allocating mem page @{pf_id}");

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
    fn find_free_slot(&mut self) -> Result<PageFrameId, String> {
        if self.is_full {
            return self.replace_and_swap();
        }

        let pf_id = self.latest_free_spot;
        self.latest_free_spot += 1;
        if self.latest_free_spot as usize == MEM_SIZE {
            self.is_full = true;
        }
        Ok(pf_id)
    }

    /// Replaces and swaps a memory page.
    /// Returns the now free page frame id (physical address).
    fn replace_and_swap(&mut self) -> Result<PageFrameId, String> {
        // TODO: Refactor so that PTE must be retrieved only once.
        let (pid, v_addr_lower) = self.page_replacement();
        let existing_pf_id = {
            let existing_pte = self.fetch_pte_mut(pid, v_addr_lower)?;
            existing_pte.p_bit = false;
            existing_pte.page_frame_id
        };
        println!("Freeing {existing_pf_id}");
        let mem_page = self.data[existing_pf_id as usize].ok_or(format!(
            "Expected mem page #{existing_pf_id} to be present."
        ))?;
        let offset = Self::swap_page_to_disk(&mut self.swap, mem_page)?;
        let existing_pte = self.fetch_pte_mut(pid, v_addr_lower)?; // This call should be
        // refactored away!
        existing_pte.page_frame_id = offset;
        Ok(existing_pf_id)
    }

    /// Implementation of a page replacement algorithm.
    /// Only selects the page, does NOT remove it!
    /// Mutable reference still needed for modifying helper fields.
    /// Returns the
    ///         (process id) + (lower bits of the virtual address)
    /// of the page that is to be replaced.
    fn page_replacement(&mut self) -> (u32, VAddrLower) {
        // Current implementation: Random selection
        let pt = self.page_tables.first().unwrap();
        let present_page_idx = pt.levels[0]
            .values
            .iter()
            .enumerate()
            .find(|(_, v)| v.p_bit)
            .map(|(i, _)| i)
            .unwrap();
        let v_addr_lower = VAddrLowerSplit::new(0, present_page_idx as u8).join();
        (pt.pid, v_addr_lower)
    }

    /// Swaps a page onto disk.
    /// Argument is the memory address of the page.
    /// Returns the disk offset to which the page was flushed as a page-frame-id.
    /// Note that the offset is multiplied by the size of a mem-page.
    fn swap_page_to_disk(swap: &mut Swap, mem_page: MemPage) -> Result<PageFrameId, String> {
        swap.swap_page_to_disk(mem_page)
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

    /// Finds the PTE for a virtual address
    /// Returns a mutable reference to it.
    fn fetch_pte_mut(
        &mut self,
        pid: u32,
        v_addr_lower: VAddrLower,
    ) -> Result<&mut PtNodeEntry, String> {
        // TODO: check cache

        let v_addr_split = VAddrLowerSplit::from(v_addr_lower);
        let pt: &mut PageTable = self
            .page_tables
            .iter_mut()
            .find(|pt| pt.pid == pid)
            .ok_or(format!("PID {pid} is not valid."))?;

        pt.get_pte_mut(v_addr_split.join())
    }

    /// Finds the physical correspondent for a virtual address.
    pub fn fetch_mem_page(
        &mut self,
        pid: u32,
        v_addr: VAddr,
    ) -> Result<(MemPage, PageOffset), String> {
        let v_addr_split = VAddrSplit::from(v_addr);
        let id = {
            let pte = self.fetch_pte(pid, v_addr_split.lower.join())?;
            let pf_id = pte.page_frame_id;
            println!("PTE: {:?}", pte);
            if !pte.v_bit {
                return Err(format!("Trying to fetch invalid address {v_addr}."));
            }
            if !pte.p_bit {
                let page = self.swap.fetch_page_from_disk(pf_id)?;
                return Ok((page, v_addr_split.offset));
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

#[test]
fn mem_page_allocation_basic() {
    let (pid1, pid2): (u32, u32) = (0, 1);
    let mut mmu = MMU::default();
    mmu.allocate_pt(pid1).unwrap();
    mmu.allocate_pt(pid2).unwrap();
    let v_addr1 = mmu.allocate_mem_page(pid1).unwrap();
    let (_mem_page1, offset1) = mmu.fetch_mem_page(pid1, v_addr1).unwrap();
    let v_addr2 = mmu.allocate_mem_page(pid2).unwrap();
    let (_mem_page2, offset2) = mmu.fetch_mem_page(pid1, v_addr1).unwrap();
    assert_eq!(v_addr1, v_addr2);
    assert_eq!(offset1, offset2);
}

#[test]
#[should_panic(
    expected = "called `Result::unwrap()` on an `Err` value: \"Trying to fetch invalid address 0."
)]
fn fetch_invalid_virtual_address_fails() {
    let non_existing_v_addr: VAddr = 0;
    let pid: u32 = 0;
    let mut mmu = MMU::default();
    mmu.allocate_pt(pid).unwrap();
    let (_mem_page1, _offset1) = mmu.fetch_mem_page(pid, non_existing_v_addr).unwrap();
}

#[test]
fn swap_to_disk() {
    // Prepare processes
    let (pid1, pid2): (u32, u32) = (0, 1);
    let mut mmu = MMU::default();
    mmu.allocate_pt(pid1).unwrap();
    mmu.allocate_pt(pid2).unwrap();
    println!("{:?}", mmu.data[0]);
    println!("{:?}", mmu);

    let v_addr1 = mmu.allocate_mem_page(pid1).unwrap();
    println!("VADDR1: {v_addr1}");
    let (mem_page, offset) = mmu.fetch_mem_page(pid1, v_addr1).unwrap();
    println!(
        "VADDR1: {v_addr1}; MemPage: {:?}; offset: {offset}",
        mem_page
    );
    let v_addr2 = mmu.allocate_mem_page(pid2).unwrap();
    let (mem_page, offset) = mmu.fetch_mem_page(pid1, v_addr1).unwrap();
    println!(
        "VADDR2: {v_addr2}; MemPage: {:?}; offset: {offset}",
        mem_page
    );
    let v_addr3 = mmu.allocate_mem_page(pid2).unwrap();
    println!("VADDR3_PRE: {v_addr3}");
    println!("{:?}", mmu.data[0]);
    println!("{:?}", mmu);
    let (mem_page, offset) = mmu.fetch_mem_page(pid2, v_addr3).unwrap();
    println!(
        "VADDR3: {v_addr3}; MemPage: {:?}; offset: {offset}",
        mem_page
    );
    let (mem_page, offset) = mmu.fetch_mem_page(pid1, v_addr1).unwrap();
    println!(
        "VADDR1: {v_addr1}; MemPage: {:?}; offset: {offset}",
        mem_page
    );
}
