#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(rt_kernel::test_runner)]

use core::panic::PanicInfo;
use rt_kernel::println;
use bootloader::BootInfo;

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    // this function is the entry point, since the linker looks for a function
    // named `_start` by default
    
    println!("Hello World{}", "!");

    rt_kernel::init();

/*	create stack overflow
    fn stack_overflow() {
        stack_overflow(); // for each recursion, the return address is pushed
    }

    // trigger a stack overflow
    stack_overflow();
*/

/*	Page fault test
	let ptr = 0x2031b2 as *mut u32;
    // read from a code page
	unsafe { let x = *ptr; }
	println!("read worked");

	// write to a code page
	unsafe { *ptr = 42; }
	println!("write worked");
*/
/*	Read page table address
	use x86_64::registers::control::Cr3;

    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", level_4_page_table.start_address());
*/


    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    rt_kernel::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	println!("{}", _info);
    rt_kernel::hlt_loop();
}

// our panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rt_kernel::test_panic_handler(info)
}



// qemu-system-x86_64 -drive format=raw,file=target/target/debug/bootimage-rt_kernel.bin
// or cargo xrun