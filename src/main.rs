use crate::mmu::MMU;

pub mod mmu;
pub mod page_table;
pub mod swap;
pub mod v_addr;

fn main() {
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
}
