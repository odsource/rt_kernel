use core::sync::atomic::{AtomicU64, Ordering};
use alloc::boxed::Box;
use x86_64::VirtAddr;
use crate::memory;

#[derive(Debug)]
pub struct Thread {
    id: ThreadId,
    stack_ptr: Option<VirtAddr>,
    stack_frame: Option<memory::StackFrame>,
}

impl Thread {
    pub fn new(stack_ptr: VirtAddr, stack_frame: memory::StackFrame) -> Thread {
        Thread {
        	id: ThreadId::new(),
            stack_ptr: Some(stack_ptr),
            stack_frame: Some(stack_frame),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadId(u64);

impl ThreadId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        ThreadId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}