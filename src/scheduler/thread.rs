use core::sync::atomic::{AtomicU64, Ordering};
use alloc::boxed::Box;
use x86_64::{
    structures::paging::{Mapper, Size4KiB, FrameAllocator},
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

    pub arrival: u32,
    pub exec: u32,
    pub deadl: u32,
    pub period: u32,
    pub time: u32,
    pub alive: bool
}

impl Thread {
    pub fn new(mapper: &mut impl Mapper<Size4KiB>, frame_allocator: &mut impl FrameAllocator<Size4KiB>, function: Box<dyn FnOnce() -> !>) -> Result<Self, u64> {
        let stack_frame = memory::get_stack_frame(mapper, frame_allocator)?;
        // stack has to grow downwards because the stack is always beginning at the end of the adress space
        let mut stack = unsafe {
            context_switch::Stack::new(stack_frame.end)
        };
        let func_ptr = stack.method(function);
        let stack_ptr = stack.get_ptr();
        //let func_ptr = Box::into_raw(function);//function.downcast::<fn() -> !>();

        Ok(Thread {
        	id: ThreadId::new(),
            stack_ptr: Some(stack_ptr),
            stack_frame: Some(stack_frame),

            //f: func_ptr,

            arrival: 0,
            exec: 0,
            deadl: 0,
            period: 0,
            time: 0,
            alive: true,
        })
    }

    pub fn initialize(&mut self, arrival: u32, exec: u32, deadl: u32, period: u32, alive: bool) {
        self.arrival = arrival;
        self.exec = exec;
        self.deadl = deadl;
        self.period = period;
        self.time = exec;
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