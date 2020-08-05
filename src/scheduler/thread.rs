use core::sync::atomic::{AtomicU64, Ordering};
use alloc::boxed::Box;
use x86_64::{
    structures::paging::{mapper::MapToError, Mapper, Size4KiB, FrameAllocator},
    VirtAddr,
};
use crate::memory;
use crate::println;
use super::EDF;
use super::context_switch;

#[derive(Debug, Copy, Clone)]
pub struct Thread {
    pub id: ThreadId,
    pub stack_ptr: Option<VirtAddr>,
    pub stack_frame: Option<memory::StackFrame>,

    //pub f: *mut fn() -> !,

    pub runtime: u64,
    pub deadline: u64,
    pub period: u64,
    pub remain_runtime: u64,
}

impl Thread {
    pub fn new(mapper: &mut impl Mapper<Size4KiB>, frame_allocator: &mut impl FrameAllocator<Size4KiB>, function: fn() -> !) -> Result <Self, MapToError<Size4KiB>> {
        let stack_frame = memory::get_stack_frame(mapper, frame_allocator)?;
        // stack has to grow downwards because the stack is always beginning at the end of the adress space
        let mut stack = unsafe {
            context_switch::Stack::new(stack_frame.end)
        };
        stack.method(function);
        let stack_ptr = stack.get_ptr();

        Ok(Thread {
        	id: ThreadId::new(),
            stack_ptr: Some(stack_ptr),
            stack_frame: Some(stack_frame),

            //f: func_ptr,

            runtime: 0,
            deadline: 0,
            period: 0,
            remain_runtime: 0,
        })
    }

    pub fn initialize(&mut self, runtime: u64, deadline: u64, period: u64) {
        self.runtime = runtime;
        self.deadline = deadline;
        self.period = period;
        self.remain_runtime = runtime;
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
