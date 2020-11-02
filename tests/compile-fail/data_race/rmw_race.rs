// ignore-windows: Concurrency on Windows is not supported yet.

use std::thread::spawn;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Copy, Clone)]
struct EvilSend<T>(pub T);

unsafe impl<T> Send for EvilSend<T> {}
unsafe impl<T> Sync for EvilSend<T> {}

static SYNC: AtomicUsize = AtomicUsize::new(0);

pub fn main() {
    let mut a = 0u32;
    let b = &mut a as *mut u32;
    let c = EvilSend(b);

    unsafe {
        let j1 = spawn(move || {
            *c.0 = 1;
            SYNC.store(1, Ordering::Release);
        });

        let j2 = spawn(move || {
            if SYNC.swap(2, Ordering::Relaxed) == 1 {
                // Blocks the acquire-release sequence
                SYNC.store(3, Ordering::Relaxed);
            }
        });

        let j3 = spawn(move || {
            if SYNC.load(Ordering::Acquire) == 3 {
                *c.0 //~ ERROR Data race
            }else{
                0
            }
        });

        j1.join().unwrap();
        j2.join().unwrap();
        j3.join().unwrap();
    }
}