#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(rt_kernel::test_runner)]

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
    
    println!("Hello World{}", "!");

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

    let arrival = 0;
    let exec = 20;
    let deadl = 25;
    let period = 50;
    let alive = false;

    let t1 = match scheduler::thread::Thread::new(&mut mapper, &mut frame_allocator, thread_loop) {
    	Ok(t) => {
    		match t.stack_frame {
    			Some(sf) => println!(), //println!("{:?}, {:?}", sf.start, sf.end),
    			None => println!("Nothing inside!"),
    		}
    		
    		Some(t)
    	},
    	Err(_) => None,
    };

    match t1 {
    	Some(mut t) => t.initialize(arrival, exec, deadl, period, alive),
    	None => (),
    }

    let arrival2 = 0;
    let exec2 = 30;
    let deadl2 = 20;
    let period2 = 80;
    let alive2 = false;

    let t2 = match scheduler::thread::Thread::new(&mut mapper, &mut frame_allocator, thread_loop) {
    	Ok(t) => Some(t),
    	Err(_) => None,
    };

    match t2 {
    	Some(mut t) => t.initialize(arrival2, exec2, deadl2, period2, alive2),
    	None => (),
    }

    scheduler::EDF.lock().new_thread(t1);
    scheduler::EDF.lock().new_thread(t2);

    println!("It did not crash!");

    start_loop();
/*
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
*/
    // not needed because the Executor::run() function is marked as diverging
    // and thus never returns
    // rt_kernel::hlt_loop();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

fn start_loop() -> ! {
	loop {

	}
}

fn thread_loop() -> ! {
    let thread_id = scheduler::EDF.lock().curr_thread;
    loop {
        println!("{:?}", thread_id);
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