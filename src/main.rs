use crate::{mmu::MMU, v_addr::PageOffset};

pub mod mmu;
pub mod page_table;
pub mod pra;
pub mod swap;
pub mod v_addr;

fn main() {
    let (pid1, pid2): (u32, u32) = (0, 1);
    let mut mmu = MMU::default();
    mmu.allocate_pt(pid1).unwrap();
    mmu.allocate_pt(pid2).unwrap();

    // Fill physical memory
    let v_addr1 = mmu.allocate_mem_page(pid1).unwrap();
    let (_mem_page, _offset) = mmu.fetch_mem_page(pid1, v_addr1).unwrap();
    let _v_addr2 = mmu.allocate_mem_page(pid2).unwrap();
    let (_mem_page, _offset) = mmu.fetch_mem_page(pid1, v_addr1).unwrap();
    
    // Allocate another memory page -> one present page is swapped out
    let v_addr3 = mmu.allocate_mem_page(pid2).unwrap();
    let (_mem_page, _offset) = mmu.fetch_mem_page(pid2, v_addr3).unwrap();

    println!("VADDR1: {:?}", v_addr1);
    // Write to swapped out page
    let (start, data): (PageOffset, Vec<u8>) = (2, vec![8,9]);
    mmu.write_to_mem_page(pid1, v_addr1, start, data.clone()).unwrap();
    let (mem_page, _offset) = mmu.fetch_mem_page(pid1, v_addr1).unwrap();
    println!("MEMPAGE: {:?}", mem_page);
    assert_eq!(mem_page.unwrap().data[start as usize..start as usize + data.len()], data)
}
