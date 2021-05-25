use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

// Returns a mutable reference to the active level 4 page table.
//
// This function is unsafe because the caller must ensure that the
// complete physical memory has been mapped to virtual memory at the
// passed `physical_memory_offset`. This function should only be
// invoked once to avoid aliasing mutable references which can cause
// undefined behaviour.
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe (call only once to avoid having mutable ref aliasing)
}

// Initialize a new OffsetPageTable, an abstraction to handle translation and
// mapping.
//
// This function is unsafe because the caller must guarantee that the complete
// physical memory is mapped to virtual memory at the given `physical_memory_offset`.
// This function must not be called more than once to prevent mutable reference
// aliasing.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

// Maps page to the VGA buffer, i.e. writing to the start of the page would be
// the same as writing directly to the VGA buffer
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    // TODO: Having more than one reference to the same physical address
    // can cause undefined behaviour
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe { mapper.map_to(page, frame, flags, frame_allocator) };
    map_to_result.expect("map_to failed").flush();
}

pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    // Next frame to return
    next: usize,
}

impl BootInfoFrameAllocator {
    // Create a FrameAllocator from the passed memory map
    //
    // This function is unsafe because the caller must guarantee
    // that the passed memory map is valid, i.e. frames that are
    // marked as USABLE are indeed really unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    // Returns an iterator over the usable frames specified in
    // the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // Iterate over memory regions
        let regions = self.memory_map.iter();
        // Filter usable regions only
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        // Map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // Transform to iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
