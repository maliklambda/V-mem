use crate::mmu::MMU;

pub mod mmu;
pub mod page_table;
pub mod v_addr;

fn main() {
    let (pid1, pid2): (u32, u32) = (0, 1);
    let mut mmu = MMU::default();
    mmu.allocate_pt(pid1).unwrap();
    mmu.allocate_pt(pid2).unwrap();
    let v_addr = mmu.allocate_mem_page(pid1).unwrap();
    println!("{:?}", mmu.data[0]);
    let (mem_page, offset) = mmu.fetch_mem_page(pid1, v_addr).unwrap();
    println!("{:?}", mmu);
}
