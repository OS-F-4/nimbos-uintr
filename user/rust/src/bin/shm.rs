#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::{exit, fork, IPC_PRIVATE, shmat, shmget, sleep};

#[no_mangle]
pub fn main() -> i32 {
    let len: usize = 114514;

    let shmid = shmget(IPC_PRIVATE, len, 0);
    let addr = shmat(shmid, 0, 0);
    let start = addr as usize;

    for i in start..(start + len) {
        let addr: *mut u8 = i as *mut u8;
        unsafe {
            *addr = i as u8;
        }
    }
    for i in start..(start + len) {
        let addr: *mut u8 = i as *mut u8;
        unsafe {
            assert_eq!(*addr, i as u8);
        }
    }
    println!("single process shm ok! addr = {}", addr);

    let shmid = shmget(IPC_PRIVATE, len, 0);
    let pid = fork();
    if pid == 0 {
        let addr = shmat(shmid, 0, 0);
        let start = addr as usize;
        for i in start..(start + len) {
            let addr: *mut u8 = i as *mut u8;
            unsafe {
                *addr = i as u8;
            }
        }
        println!("Child process has finished writing. Child shm addr = {}", addr);
        exit(0);
    } else {
        shmat(shmid, 0, 0);  // No-op; try to make parent and child addr different
        let addr = shmat(shmid, 0, 0);
        let start = addr as usize;
        println!("Father process will sleep for 100ms.");
        sleep(100);
        println!("Father process wakes up.");
        for i in start..(start + len) {
            let addr: *mut u8 = i as *mut u8;
            unsafe {
                assert_eq!(*addr, i as u8);
            }
        }
        println!("Multi process shm ok! Parent shm addr = {}", addr);
    }
    0
}
