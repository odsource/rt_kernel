use core::sync::atomic::{AtomicU64, Ordering};
use alloc::boxed::Box;
use x86_64::{
    structures::paging::{Mapper, mapper, Size4KiB, FrameAllocator},
    VirtAddr,
    PhysAddr,
};
use crate::memory;
use crate::println;
use super::EDF;

#[derive(Debug)]
pub struct Thread {
    id: ThreadId,
    stack_ptr: Option<VirtAddr>,
    stack_frame: Option<memory::StackFrame>,

    arrival: u32,
    exec: u32,
    deadl: u32,
    period: u32,
    alive: bool
}

impl Thread {
    pub fn new(mapper: &mut impl Mapper<Size4KiB>, frame_allocator: &mut impl FrameAllocator<Size4KiB>) -> Result<Self, u64> {
        let stack_frame = memory::get_stack_frame(mapper, frame_allocator)?;
        let stack_ptr = stack_frame.end;
        Ok(Thread {
        	id: ThreadId::new(),
            stack_ptr: Some(stack_ptr),
            stack_frame: Some(stack_frame),

            arrival: 0,
            exec: 0,
            deadl: 0,
            period: 0,
            instance: 0,
            alive: true,
        })
    }

    pub fn initialize(&mut self, arrival: u32, exec: u32, deadl: u32, period: u32, alive: bool) {
        self.arrival = arrival;
        self.exec = exec;
        self.deadl = deadl;
        self.period = period;
        self.alive = alive;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadId(u64);

impl ThreadId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        ThreadId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

fn thread_loop() -> ! {
    let thread_id = EDF.curr_thread;
    loop {
        println!("{:?}", thread_id);
    }
}