use crate::{
    mmu::PageFrameId,
    v_addr::{VAddr, VAddrLower, VAddrLowerSplit, VAddrSplit},
};

/// Indicates the number of levels of the PT.
/// The last level carries the physical addresses.
pub const PT_LEVELS: usize = 1;

/// How many child-ptrs stored per PT-Node.
pub const PT_NODE_ARR_SIZE: usize = 5;

/// Allocated on a per-process base (i.e. every process has a page-table)
/// Wrapper around nested PageTableNodes.
#[derive(Debug, Default)]
pub struct PageTable {
    /// Indicates which process this PT is for.
    pub pid: u32,

    /// Latest mem-page allocated in this table
    pub latest: u16,

    /// Root of the table-node-tree
    pub levels: [PageTableNode; PT_LEVELS],
}

impl PageTable {
    pub fn new(pid: u32) -> Self {
        Self {
            pid,
            latest: 0,
            levels: [PageTableNode::default(); PT_LEVELS],
        }
    }

    /// Adds a new memory page to the PT.
    /// Returns the virtual address at the place of allocation.
    pub fn add_mem_page(&mut self, page_frame_id: PageFrameId) -> Result<VAddr, String> {
        // Determine new v_address.
        // check for bounds on virtual address space: if latest > MAX_V_ADDR_SIZE {...}

        let v_addr_lower: VAddrLower = self.latest + 1;
        let v_addr_lower_split = VAddrLowerSplit::from(v_addr_lower);
        self.add_pte(v_addr_lower, page_frame_id)
            .ok_or(String::from("Failed allocation of new PTE"))?;
        self.latest += 1;
        let v_addr_split = VAddrSplit::new(v_addr_lower, 0); // zero because address is not needed.
        Ok(v_addr_split.join())
    }

    fn add_pte(&mut self, v_addr_lower: VAddrLower, page_frame_id: PageFrameId) -> Option<()> {
        let lower_split = VAddrLowerSplit::from(v_addr_lower);
        self.levels[lower_split.level_idx as usize].values[lower_split.pt_node_arr_idx as usize] =
            PtNodeEntry::new(page_frame_id);
        Some(())
    }

    /// Retrieves a memory page
    /// Returns the page table entry
    pub fn get_pte(&self, v_addr_lower: VAddrLower) -> Result<&PtNodeEntry, String> {
        if v_addr_lower > self.latest {
            return Err(format!("Invalid address: {v_addr_lower}"));
        }
        let lower_split = VAddrLowerSplit::from(v_addr_lower);
        Ok(&self.levels[lower_split.level_idx as usize].values
            [lower_split.pt_node_arr_idx as usize])
    }
}

/// Node
#[derive(Debug, Default, Clone, Copy)]
pub struct PageTableNode {
    /// Specifies the index to the next level
    /// or in case of the last level, holds the physical addresses.
    pub values: [PtNodeEntry; PT_NODE_ARR_SIZE],
}

/// Entry to PT-Nodes.
#[derive(Debug, Default, Clone, Copy)]
pub struct PtNodeEntry {
    /// Is PTE valid?
    /// Non-existing PTEs are marked with v_bit = false.
    pub v_bit: bool,

    /// Present in memory or swapped to disk?
    pub p_bit: bool,

    /// Modified in memory?
    /// P-bit must be set in that case.
    pub m_bit: bool,

    /// Page Frame ID.
    /// Either references RAM or offset on disk (based on p_bit-setting).
    pub page_frame_id: PageFrameId,
}

impl PtNodeEntry {
    pub fn new(page_frame_id: PageFrameId) -> Self {
        Self {
            page_frame_id,
            v_bit: true, // new is itialized
            p_bit: true,
            ..Default::default()
        }
    }
}

pub const MEMPAGE_SIZE: usize = 5;

/// Memory page.
#[derive(Debug, Clone, Copy)]
pub struct MemPage {
    pub data: [u8; MEMPAGE_SIZE],
}

impl Default for MemPage {
    fn default() -> Self {
        Self {
            data: [0_u8; MEMPAGE_SIZE],
        }
    }
}
