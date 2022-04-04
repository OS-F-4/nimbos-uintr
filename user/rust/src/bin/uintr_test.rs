#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::{exit, fork, IPC_PRIVATE, shmat, shmget, sleep, uintr_register_link, uintr_register_sender};

#[no_mangle]
pub fn main() -> i32 {
    // TODO: register handler

    let mut shmid = 0;
    let link_id = uintr_register_link(0, &shmid);
    let addr = shmat(shmid as isize, 0, 0);

    println!("Receiver process register link ok! addr = {}", addr);

    let len = 256;

    let pid = fork();
    if pid == 0 {
        let mut shmid = 0;
        let index = uintr_register_sender(link_id as usize, &shmid);
        let addr = shmat(shmid as isize, 0, 0);
        let start = addr as usize;
        for i in start..(start + len) {
            let addr: *mut u8 = i as *mut u8;
            unsafe {
                *addr = i as u8;
            }
        }
        println!("Sender process has finished writing. sender shm addr = {}", addr);

        //TODO: notice(senduipi)

        exit(0);
    } else {
        shmat(shmid as isize, 0, 0);  // No-op; try to make parent and child addr different
        let addr = shmat(shmid as isize, 0, 0);
        let start = addr as usize;
        println!("Receiver process will sleep for 100ms.");
        sleep(100);
        println!("Receiver process wakes up.");
        for i in start..(start + len) {
            let addr: *mut u8 = i as *mut u8;
            unsafe {
                assert_eq!(*addr, i as u8);
            }
        }
        println!("Multi process communicate ok! Receiver shm addr = {}", addr);
    }
    0
}
