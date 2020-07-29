// Assembler Part for register saving: switch processor state from old process to new one
pub fn assembler_test() {
	unsafe {
    	asm!("nop");
	}
}
// switch virtual memory mapping of the old process with the new one