use alloc::collections::VecDeque;
use crate::println;
use x86_64::VirtAddr;
use crate::scheduler::thread::ThreadId;
use lazy_static::lazy_static;

pub mod context_switch;
pub mod thread;

static TIMER: u32 = 10;

lazy_static! {
    pub static ref EDF: Locked<EDFScheduler> = Locked::new(EDFScheduler::new());
}

pub struct EDFScheduler {
    threads: VecDeque<thread::Thread>,
    curr_thread: ThreadId,
}

impl EDFScheduler {
    pub fn new() -> Self {
        EDFScheduler {
            threads: VecDeque::new(),
            curr_thread: ThreadId::new(),
        }
    }

    pub fn schedule(&mut self) {
        if self.threads.len() > 1 {
            self.threads[0].time -= TIMER;
            if self.threads[0].deadl < self.threads[1].deadl {

            } else {
                //context(VirtAddr::new(self.threads.thread.stack_ptr));
            }
        } else {

        }
        
    }

    pub fn new_thread(&mut self, thread: Option<thread::Thread>) {
        self.calc_position(thread);
    }

    fn calc_position(&mut self, thread: Option<thread::Thread>) {
        // Just an easy calculation for the start
        // TODO: make a better calculation
        match thread {
            Some(t) => {
                for i in 0..self.threads.len() {
                    if self.threads[i].deadl < t.deadl {
                        self.threads.insert(i, t);
                        break;
                    }
                }
            },
            None => println!("Could not insert thread"),
        }
        
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

/// A wrapper around spin::Mutex to permit trait implementations.
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}