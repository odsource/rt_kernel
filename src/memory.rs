use x86_64::{
    structures::paging::{PageTable, PageTableFlags, Page, Mapper, mapper, OffsetPageTable, PhysFrame, Size4KiB, FrameAllocator},
    VirtAddr,
    PhysAddr,
};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use core::sync::atomic::{AtomicU64, Ordering};

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let regions = self.memory_map.iter();
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr()..r.range.end_addr());
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}


/// Initialize a new OffsetPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable
{
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

/// A FrameAllocator that always returns `None`.
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackFrame {
	start: VirtAddr,
	end: VirtAddr,
}

impl StackFrame {
	fn new(&mut self, start: VirtAddr, end: VirtAddr) {
		self.start;
		self.end;
	}
}

// 32 KiB für stack
pub fn get_stack_frame(mapper: &mut impl Mapper<Size4KiB>, frame_allocator: &mut impl FrameAllocator<Size4KiB>) -> Result<StackFrame, u64> {
	// Atomic operation to ensure there is no context switch
	static STACK: AtomicU64 = AtomicU64::new(0x888888880000);
	let new_stack_start = STACK.fetch_add(8 * Page::<Size4KiB>::SIZE, Ordering::SeqCst);
	let stack_start = Page::from_start_address(VirtAddr::new(new_stack_start)).expect("Stack start not accessible");
	let stack_end = stack_start + 8;

	// Flags:
	// present: frame is loaded into memory
	let present = PageTableFlags::PRESENT;
	// writable: makes the frame accessible
	let writable = PageTableFlags::WRITABLE;

	// allocate each page
	for p in Page::range(stack_start, stack_end) {
        let frame = frame_allocator.allocate_frame();

        let frame = match frame {
        	Some(v) => v,
        	None => return Err(1),
        };
        unsafe {
        	match mapper.map_to(p, frame, present | writable, frame_allocator) {
	        	Ok(mf) => mf.flush(),
	        	Err(_) => return Err(1),
	        };
        }
        
    }

	let sf = StackFrame{
		start: stack_start.start_address(),
		end: stack_end.start_address(),
	};
	Ok(sf)
}
