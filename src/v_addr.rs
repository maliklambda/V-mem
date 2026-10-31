/// Virtual address
pub type VAddr = u32;

/// Offset in Memory Page (upper bits in Vitrual Addr)
pub type PageOffset = u16;

/// Structure of a virtual address
#[derive(Debug)]
pub struct VAddrSplit {
    /// Indices into page table
    pub lower: VAddrLowerSplit,

    /// Memory Page Offset into physical memory page
    pub offset: PageOffset,
}

impl VAddrSplit {
    pub fn new(lower: VAddrLower, offset: PageOffset) -> Self {
        Self {
            lower: VAddrLowerSplit::from(lower),
            offset,
        }
    }

    pub fn join(&self) -> VAddr {
        let lower_joined: VAddrLower = self.lower.join();
        ((lower_joined as VAddr) << 16) | self.offset as VAddr
    }
}

impl From<VAddr> for VAddrSplit {
    fn from(value: VAddr) -> Self {
        let low_bits = (value >> 16) as VAddrLower;
        let lower = VAddrLowerSplit::from(low_bits);
        let offset = (value & 0xffff) as PageOffset;
        Self { lower, offset }
    }
}

#[test]
fn test_v_addr_split_join() {
    let v_addr_lower: VAddrLower = u16::MAX;
    let v_addr_lower_split = VAddrLowerSplit::from(v_addr_lower);
    let control = v_addr_lower_split.join();
    assert_eq!(control, v_addr_lower, "VAddrLower split/join failed");

    let v_addr: VAddr = 1234125;
    let v_addr_split = VAddrSplit::from(v_addr);
    let control = v_addr_split.join();
    assert_eq!(control, v_addr, "VAddr split/join failed");
}

/// Virtual address without PageFrameId (lower bits in Vitrual Addr)
/// Correspond to level and pt-node-array indices.
pub type VAddrLower = u16;

#[derive(Debug)]
pub struct VAddrLowerSplit {
    /// Currently unused!
    /// Index into intermediate page tables
    pub level_idx: u8,

    /// Index to PTE in PT-Node-Array
    pub pt_node_arr_idx: u8,
}

impl From<VAddrLower> for VAddrLowerSplit {
    fn from(value: VAddrLower) -> Self {
        let low_bits = (value >> 8) as u8;
        let high_bits = (value & 0xff) as u8;
        Self {
            level_idx: low_bits,
            pt_node_arr_idx: high_bits,
        }
    }
}

impl VAddrLowerSplit {
    pub fn new(level_idx: u8, pt_node_arr_idx: u8) -> Self {
        Self {
            level_idx,
            pt_node_arr_idx,
        }
    }

    pub fn join(&self) -> VAddrLower {
        ((self.level_idx as VAddrLower) << 8) | self.pt_node_arr_idx as VAddrLower
    }
}
