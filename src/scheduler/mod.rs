use alloc::collections::VecDeque;
use crate::println;
use x86_64::VirtAddr;
use crate::scheduler::thread::ThreadId;
use lazy_static::lazy_static;

pub mod context_switch;
pub mod thread;

lazy_static! {
    pub static ref EDF: EDFScheduler = EDFScheduler::new();
}


pub struct EDFTask {
    arrival: u32,
    exec: u32,
    deadl: u32,
    period: u32,
    instance: u32,
    alive: bool
}

pub struct EDFDeadline {

}

pub struct EDFScheduler {
    threads: VecDeque<EDFTask>,
    curr_thread: ThreadId,
}

impl EDFScheduler {
    pub fn new() -> Self {
        EDFScheduler {
            threads: VecDeque::new(),
            curr_thread: ThreadId::new(),
        }
    }

    pub fn schedule(&self) {
        context(VirtAddr::new(10));
    }

    pub fn new_thread(&self, thread: ThreadId) {

    }

    fn gcd(&self, m: u32, n: u32) -> u32 {
        if m == 0 {
            n
        } else {
            self.gcd(n % m, m)
        }
    }

    fn lcm(&self, a: u32, b: u32) -> u32 {
        a * b / self.gcd(a, b)
    }

    fn cpu_workload(&self) -> f32 {
        let mut wl: f32 = 0.0;

        for i in 0..self.threads.len() {
            wl += (self.threads[i].exec / self.threads[i].deadl) as f32;
        }
        wl
    }

    fn calc_hyperperiod(&self) -> u32 {
        println!("{}", 0);
        0
    }
}

pub fn context(ptr: VirtAddr) {
    context_switch::switch_context(ptr);
}