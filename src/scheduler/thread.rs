use core::sync::atomic::{AtomicU64, Ordering};
use alloc::boxed::Box;
use x86_64::VirtAddr;

#[derive(Debug)]
pub struct Thread {
    id: ThreadId,
    stack_ptr: Option<VirtAddr>,
    // TODO: eigene Stack Partition für jeden Thread
}

impl Thread {
    pub fn new(stack_ptr: VirtAddr) -> Thread {
        Thread {
        	id: ThreadId::new(),
            stack_ptr: Some(stack_ptr),
            // TODO: Stack
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