#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(rt_kernel::test_runner)]

extern crate alloc;

use core::panic::PanicInfo;
use rt_kernel::println;
use bootloader::{BootInfo, entry_point};
use alloc::boxed::Box;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
	use rt_kernel::memory;
    use rt_kernel::memory::BootInfoFrameAllocator;
    use x86_64::{structures::paging::{MapperAllSizes, Page}, VirtAddr};
    
    println!("Hello World{}", "!");

    rt_kernel::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };


    let x = Box::new(41);
    

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