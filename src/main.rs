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

    x86_64::instructions::interrupts::disable();
    let exec = 25;
    let deadl = 123;
    let period = 123;

    if let Ok(mut t1) = scheduler::thread::Thread::new(&mut mapper, &mut frame_allocator, thread1_loop) {
        t1.initialize(exec, deadl, period);
        println!("Locking T1");
        scheduler::EDF.lock().new_thread(t1);
        println!("Unlocked T1");
    }
    
    let exec2 = 30;
    let deadl2 = 61;
    let period2 = 61;

    if let Ok(mut t2) = scheduler::thread::Thread::new(&mut mapper, &mut frame_allocator, thread2_loop) {
        t2.initialize(exec2, deadl2, period2);
        println!("Locking T2");
        scheduler::EDF.lock().new_thread(t2);
        println!("Unlocked T2");
    }

    let exec3 = 5;
    let deadl3 = 100;
    let period3 = 100;

    if let Ok(mut t3) = scheduler::thread::Thread::new(&mut mapper, &mut frame_allocator, thread3_loop) {
        t3.initialize(exec3, deadl3, period3);
        println!("Locking T3");
        scheduler::EDF.lock().new_thread(t3);
        println!("Unlocked T3");
    }


    println!("Locking START");
    let pair = scheduler::EDF.lock().start();
    println!("Unlocking START");
    x86_64::instructions::interrupts::enable();

    match pair {
    	Some((k,v)) => {
    		scheduler::context(v.stack_ptr.expect("No stack pointer inside thread!"));
    	},
    	None => println!("No pair context"),
    }

    println!("It did not crash!");
    
    // not needed because the Executor::run() function is marked as diverging
    // and thus never returns
    rt_kernel::hlt_loop();
}


fn thread1_loop() -> ! {
    let a: [u64; 2] = [1, 2];
    let mut i = 0;
    println!("Thread 1");
    loop {
    	if i == 10000000 {
    		i = 0;
    		println!("Thread {} executing", a[0]);
    	}
        i += 1;
        //println!("Thread {} executing", a[0]);
        //scheduler::yield_thread();
    }
}

fn thread2_loop() -> ! {
    let a: [u64; 2] = [1, 2];
    let mut i = 0;
    println!("Thread 2");

    loop {
    	if i == 10000000 {
    		i = 0;
    		/*
    		x86_64::instructions::interrupts::disable();
		    scheduler::EDF.lock().print_tree();
		    x86_64::instructions::interrupts::enable();
		    */
    		println!("Thread {} executing", a[1]);
    	}
        i += 1;
        //println!("Thread {} executing", a[1]);
        //scheduler::yield_thread();
    }
}

fn thread3_loop() -> ! {
    let a: [u64; 3] = [1, 2, 3];
    let mut i = 0;
    println!("Thread 3");

    loop {
    	if i == 10000000 {
    		i = 0;
    		/*
    		x86_64::instructions::interrupts::disable();
		    scheduler::EDF.lock().print_tree();
		    x86_64::instructions::interrupts::enable();
		    */
    		println!("Thread {} executing", a[2]);
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
