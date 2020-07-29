use alloc::collections::VecDeque;

use crate::println;

pub mod context_switch;

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
    curr_thread: u32,
}

impl EDFScheduler {
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

pub fn context() {
    context_switch::assembler_test();
}