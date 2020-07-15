use alloc::collections::VecDeque;

use crate::println;

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

    cpu_workload(&self) -> f32 {
        let mut wl: f32;
        for t in self.threads {
            wl += t.exec / t.deadl;
        }
        wl
    }

    fn calc_hyperperiod(&self) -> u32 {
        println!("{}", 0);
        0
    }
}
