use crate::mm::{create_shm_seg, get_shm_seg_paddr_vec};
use crate::task::CurrentTask;

pub fn sys_shmget(key: usize, size: usize, shmflg: usize) -> isize {
    create_shm_seg(CurrentTask::get().pid().as_usize(), key, size, shmflg)
}

pub fn sys_shmat(shmid: usize, shmaddr: usize, shmflg: usize) -> isize {
    let paddr_vec = get_shm_seg_paddr_vec(shmid, shmaddr, shmflg);
    if let Some(shared_paddr_vec) = paddr_vec {
        let start_addr = CurrentTask::get().map_shared_frames(shared_paddr_vec);
        if let Some(addr) = start_addr {
            addr.as_usize() as isize
        } else {
            -1
        }
    } else {
        -1
    }
}

pub fn sys_shmdt() -> isize {
    -1
}

pub fn sys_shmctl() -> isize {
    -1
}