#![allow(dead_code, unused_variables)]

use NVML;
use device::Device;
use enum_wrappers::{Brand};
use std::thread;
use std::mem;
use std::sync::Arc;
use std::fmt::Debug;

pub trait ShouldPrint {
    fn should_print(&self) -> bool {
        true
    }
}

impl ShouldPrint for () {
    fn should_print(&self) -> bool {
        false
    }
}

impl<'a> ShouldPrint for &'a str {}
impl ShouldPrint for String {}
impl ShouldPrint for Brand {}
impl ShouldPrint for [i8; 16] {}

pub fn device<'nvml>(from: &'nvml NVML, index: usize) -> Device<'nvml> {
    from.device_by_index(0).expect(&format!("Could not get{} device by index 0", index))
}

/// Run the given test once.
pub fn single<T, S, A>(test: T) 
    where T: Fn(NVML) -> (S),
          S: Into<Option<A>>,
          A: Debug + ShouldPrint {
    let nvml_test = NVML::init().expect("init call failed");
    if let Some(s) = test(nvml_test).into() {
        if s.should_print() {
            print!("{:?} ... ", s);
        }
    }
}

/// Run the given test multiple times.
pub fn multi<T, S, A>(count: usize, test: T) 
    where T: Fn(NVML, usize) -> (S),
          S: Into<Option<A>>,
          A: Debug + ShouldPrint {
    for i in 0..count {
        let nvml_test = NVML::init().expect(&format!("init call{} failed", i));
        if let Some(s) = test(nvml_test, i).into() {
            if s.should_print() {
                println!("{:?}", s);
            }
        }
    }
}

/// Run the given test on multiple threads, initializing NVML for each thread.
pub fn multi_thread<T, S, A>(threads: usize, test: T) 
    where T: Fn(NVML, usize) -> (S) + Send + Sync + 'static,
          S: Into<Option<A>> + Send + Sync + 'static,
          A: Debug + ShouldPrint {
    let mut handles = Vec::with_capacity(mem::size_of::<thread::JoinHandle<S>>() * threads);
    let fn_arc = Arc::new(test);

    for i in 0..threads {
        let fn_clone = fn_arc.clone();

        handles.push(thread::spawn(move || {
            let nvml_test = NVML::init().expect(&format!("init call{} failed", i));
            fn_clone(nvml_test, i)
        }));
    }

    for (i, j) in handles.into_iter().enumerate() {
        let res = j.join().expect(&format!("handle{} join failed", i));
        if let Some(s) = res.into() {
            if s.should_print() {
                println!("{:?}", s);
            }
        }
    }
}

/// Run the given test on multiple threads with a single `Arc`-wrapped NVML.
pub fn multi_thread_arc<T, S, A>(threads: usize, test: T) 
    where T: Fn(Arc<NVML>, usize) -> (S) + Send + Sync + 'static,
          S: Into<Option<A>> + Send + Sync + 'static,
          A: Debug + ShouldPrint {
    let mut handles = Vec::with_capacity(mem::size_of::<thread::JoinHandle<S>>() * threads);
    let nvml_test = Arc::new(NVML::init().expect("init call failed"));
    let fn_arc = Arc::new(test);

    for i in 0..threads {
        let nvml_clone = nvml_test.clone();
        let fn_clone = fn_arc.clone();

        handles.push(thread::spawn(move || {
            fn_clone(nvml_clone, i)
        }));
    }

    for (i, j) in handles.into_iter().enumerate() {
        let res = j.join().expect(&format!("handle{} join failed", i));
        if let Some(s) = res.into() {
            if s.should_print() {
                println!("{:?}", s);
            }
        }
    }
}
