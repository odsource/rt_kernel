#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(rt_kernel::test_runner)]

extern crate rlibc;
extern crate alloc;

use core::panic::PanicInfo;
use rt_kernel::println;
use bootloader::{BootInfo, entry_point};
use rt_kernel::task::{Task, keyboard, executor::Executor};
use rt_kernel::scheduler;
use alloc::boxed::Box;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
	use rt_kernel::allocator;
    use rt_kernel::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;
    
    rt_kernel::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    #[cfg(test)]
    test_main();

    let exec = 20;
    let deadl = 60;
    let period = 60;

    if let Ok(mut t1) = scheduler::thread::Thread::new(&mut mapper, &mut frame_allocator, thread1_loop) {
        t1.initialize(exec, deadl, period);
        x86_64::instructions::interrupts::disable();
        scheduler::EDF.lock().new_thread(t1);
        x86_64::instructions::interrupts::enable();
    }
    
    let exec2 = 30;
    let deadl2 = 50;
    let period2 = 50;

    if let Ok(mut t2) = scheduler::thread::Thread::new(&mut mapper, &mut frame_allocator, thread2_loop) {
        t2.initialize(exec2, deadl2, period2);
        x86_64::instructions::interrupts::disable();
        scheduler::EDF.lock().new_thread(t2);
        x86_64::instructions::interrupts::enable();
    }

    //scheduler::EDF.lock().print_tree();
    scheduler::EDF.lock().start();

    println!("It did not crash!");

/*
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
*/
    // not needed because the Executor::run() function is marked as diverging
    // and thus never returns
    rt_kernel::hlt_loop();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

fn thread1_loop() -> ! {
	scheduler::EDF.force_unlock();
    let a: [u64; 2] = [1, 2];
    let mut i = 0;
    println!("Thread 1");
    loop {
    	if i == 100000000 {
    		i = 0;
    		println!("Thread {} executing", a[0]);
    	}
        i += 1;
        //println!("Thread {} executing", a[0]);
        //scheduler::yield_thread();
    }
}

fn thread2_loop() -> ! {
	scheduler::EDF.force_unlock();
    let a: [u64; 2] = [1, 2];
    let mut i = 0;
    println!("Thread 2");
    loop {
    	if i == 100000000 {
    		i = 0;
    		println!("Thread {} executing", a[1]);
    	}
        i += 1;
        //println!("Thread {} executing", a[1]);
        //scheduler::yield_thread();
    }
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
