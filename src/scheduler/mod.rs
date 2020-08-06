use alloc::collections::BTreeMap;
use crate::{println, print};
use x86_64::VirtAddr;
use crate::scheduler::thread::ThreadId;
use lazy_static::lazy_static;
use crate::interrupts::{GLOBAL_TIME, PICS, InterruptIndex};

pub mod context_switch;
pub mod thread;

static TIMER: u32 = 10;

lazy_static! {
    pub static ref EDF: Locked<EDFScheduler> = Locked::new(EDFScheduler::new());
}

lazy_static! {
    pub static ref OLD_POINTER: Locked<OldPtr> = Locked::new(OldPtr::new());
}

pub struct OldPtr {
    ptr: VirtAddr,
}

impl OldPtr {
    fn new() -> Self {
        OldPtr { ptr: VirtAddr::new(0) }
    }

    pub fn set_ptr(&mut self, ptr: VirtAddr) {
        self.ptr = ptr;
    }

    pub fn get_ptr(&self) -> VirtAddr {
        self.ptr
    }
}

pub struct EDFScheduler {
    init: bool,
    tasks: BTreeMap<u64, thread::Thread>,
    active_task: u64,
    old_task: u64,
}

impl EDFScheduler {
    pub fn new() -> Self {
        EDFScheduler { init: false, tasks: BTreeMap::new(), active_task: 0, old_task: 0 }
    }

    pub fn start(&mut self) -> Option<(u64, thread::Thread)> {
        println!("Scheduler started");
        self.select_thread()
    }

    pub fn set_old_ptr(&mut self, ptr: VirtAddr) {
        if let Some(ot) = self.tasks.get_mut(&self.old_task) {
            ot.stack_ptr = Some(ptr);
            println!("Old stack pointer: {:?}", ptr);
        }
    }

    pub fn schedule(&mut self) -> Option<(u64, thread::Thread)> {
        if self.init == true {
            //println!("Inside init");
            //self.print_tree();
            if let Some(at) = self.tasks.get_mut(&self.active_task) {
                at.remain_runtime -= 1;

                if at.remain_runtime == 0 {
                    if let Some(mut rpt) = self.tasks.remove(&self.active_task) {
                        rpt.remain_runtime = rpt.runtime;
                        unsafe{ self.tasks.insert(GLOBAL_TIME + rpt.deadline, rpt) };
                    }
                }
            }
        
            return self.select_thread()
        }
        None
        //print!("After schedule");
    }

    fn select_thread(&mut self) -> Option<(u64, thread::Thread)> {
        let pair = self.tasks.first_key_value();
        match pair {
            Some((key,val)) => {
                if *key != self.active_task {
                    println!("Before context switch");
                    self.old_task = self.active_task;
                    self.active_task = *key;
                    if self.init == false {
                        self.init = true;
                    }
                    
                }
                Some((*key,*val))
            },
            None => None,
        }
        /*
        if let Some((key, val)) = self.tasks.first_key_value() {
            if *key != self.active_task {
                println!("Before context switch");
                self.old_task = self.active_task;
                self.active_task = *key;
                if self.init == false {
                    self.init = true;
                    first_context(val.stack_ptr.expect("No stack pointer inside thread!"));
                } else {
                    context(val.stack_ptr.expect("No stack pointer inside thread!")); 
                }
            }
        } 
        */ 
    }

/*
    pub fn update_stack_ptr(&mut self, stack_ptr: VirtAddr) {
        if let Some(ot) = self.tasks.get_mut(&self.old_task) {
            ot.stack_ptr = Some(stack_ptr);
            println!("Old stack pointer: {:?}", stack_ptr);
        }
        println!("Functions at all");
    }
*/
    pub fn new_thread(&mut self, t: thread::Thread) {
        let mut u = 0.0;
        for (_key, value) in self.tasks.iter() {
            u += value.runtime as f64 / value.period as f64;
        }
        u += t.runtime as f64 / t.period as f64;

        // single proccessor schedulability test
        if u > 1.0 {
            println!("Can't schedule task -> Processor utilization exceeded 100%.");
        } else {
            unsafe{ self.tasks.insert(GLOBAL_TIME +  t.deadline, t) };
        }
    }

    pub fn print_tree(&self) {
        for (key, value) in self.tasks.iter() {
            println!("Thread: dead: {}, remain: {}", key, value.remain_runtime);
        }
    }
}

pub fn context(stack_ptr: VirtAddr) {
    context_switch::switch_context(stack_ptr);
}

pub fn first_context(stack_ptr: VirtAddr) {
    context_switch::first_switch_context(stack_ptr);
}

pub fn yield_thread() {
    EDF.force_unlock();
    EDF.lock().schedule();
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

    pub fn try_lock(&self) -> Option<spin::MutexGuard<A>> {
        self.inner.try_lock()
    }

    pub fn force_unlock(&self) {
        unsafe {self.inner.force_unlock();}
    }
}
