use x86_64::VirtAddr;

// Assembler Part for register saving: switch processor state from old process to new one
pub fn assembler_test() {
	unsafe {
    	asm!("nop");
	}
}

// switch virtual memory mapping of the old process with the new one
pub struct Stack {
    ptr: VirtAddr,
}

impl Stack {
    pub unsafe fn new(ptr: VirtAddr) -> Self {
        Stack {
            ptr,
        }
    }

    pub fn get(self) -> VirtAddr {
        self.ptr
    }
}
