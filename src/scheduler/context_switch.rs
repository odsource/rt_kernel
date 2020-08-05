use x86_64::VirtAddr;
use alloc::boxed::Box;
use core::mem;
use core::raw::TraitObject;
use crate::println;

// Assembler Part for register saving: switch processor state from old process to new one
/*
asm!(assembly template
   : output operands
   : input operands
   : clobbers
   : options
   );
*/
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn switch_context(new_stack_ptr: VirtAddr) {
	unsafe {
    	llvm_asm!(
    		"call switch_stack_ptr"
    		:
    		: "{rsi}"(new_stack_ptr)
    		: "rax", "rbx", "rcx", "rdx", "rbp", "rsp", "rsi", "rdi", "rflags", "memory", "r8", "r9", "r10", "r11", "r12", "r13", "r14"
    		: "intel", "volatile"
    	);
	}
}

global_asm!("
	.intel_syntax noprefix

	switch_stack_ptr:
		// Pushes the register to the stack (RFLAGS)
		//pushfq

		mov rax, rsp
		mov rsp, rsi

		// Pops the stack register to the register (RFLAGS)
		//popfq
		ret
");

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

    pub fn get_ptr(self) -> VirtAddr {
        self.ptr
    }

    // Write the loop-function to the stack
    pub fn method<T>(&mut self, function: T) {
    	let stack_size = mem::size_of::<T>();
        println!("Function size: {:?}", stack_size);
    	self.ptr -= stack_size;
    	let ptr: *mut T = self.ptr.as_mut_ptr();
    	unsafe {ptr.write(function)};
    }

}
