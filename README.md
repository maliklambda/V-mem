# Vmem
The result of this recreational programming session is a virtual memory system.
This project features a emulated memory management unit that has memory pages it can allocate to processes.
A process is defined through its PID, and that's all there is about processes here.
The base of this v-mem-system is a page-table that is allocated to each process through `fn allocate_pt(mmu, pid)`. 
It is then possible for memory pages to be allocated in the MMU and associated with a process.
The API enables this through `fn allocate_mem_page(mem, pid)`.
The memory pages are allocated sequentially in the MMU, 
but the virtual address returned from the allocation of the memory page may (and will) be the same for different processes.
The same virtual address may reference different memory pages for different processes.

## Virtual Addresses
Virtual addresses are 32 bits long.
They are split into upper and lower 16 bits. 
The upper ones are an offset into the virtual/physical memory page.
This implies that memory pages may be 2^16 bytes (as 16 bits are available for the offset) long at most.
The lower 16 bits are split into 8 bits for the level index into multi-level page tables (currently only a single level is supported)
and 8 bits for the index into the resulting PageTableEntry.

## Swapping
Pages are swapped out to disk (emulated by a single binary file) when the physical memory is full.
Since pages have a max-size of 16KiB and another 16bits make up a virtual address, the virtual address space that is available to every process is around 4GBs.
