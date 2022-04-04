#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{exit, fork, IPC_PRIVATE, shmat, shmget, sleep, wait};

const NUM: usize = 64;

#[derive(Copy, Clone)]
struct Mapping {
    pid: isize,
    shmaddr: *mut isize,
}

#[no_mangle]
pub fn main() -> i32 {
    let mut pid_shmaddr_list = [Mapping { pid: 0, shmaddr: 0 as *mut isize }; NUM];
    for i in 0..NUM {
        let shmid = shmget(IPC_PRIVATE, 4, 0);
        let pid = fork();
        if pid == 0 {
            let addr = shmat(shmid, 0, 0) as *mut isize;
            sleep(100);
            unsafe {
                *addr = *addr * 2;
            }
            exit(0);
        } else {
            let addr = shmat(shmid, 0, 0) as *mut isize;
            unsafe {
                *addr = pid;
            }
            pid_shmaddr_list[i] = Mapping { pid, shmaddr: addr };
        }
    }

    sleep(200);
    let mut exit_code: i32 = 0;
    let mut cnt = 0;
    for _ in 0..NUM {
        let exit_pid = wait(&mut exit_code);
        for i in 0..NUM {
            if pid_shmaddr_list[i].pid == exit_pid {
                unsafe {
                    assert_eq!(exit_pid * 2, *pid_shmaddr_list[i].shmaddr);
                    cnt += 1;
                }
                break;
            }
        }
    }
    assert_eq!(cnt, NUM);

    println!("shm2 test OK!");
    0
}
