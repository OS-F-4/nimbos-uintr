use crate::mm::{create_shm_seg, get_shm_seg_paddr_vec, UserOutPtr};
use crate::task::CurrentTask;

pub fn sys_uintr_register_receiver(handler: usize) -> isize {
    -1
}

pub fn sys_uintr_register_link(vector: usize, mut shmem_id: UserOutPtr<usize>) -> isize {
    let id = create_shm_seg(CurrentTask::get().pid().as_usize(), 0, 1024, 0);
    shmem_id.write(id as usize);
    id
}

pub fn sys_uintr_register_sender(link_id: usize, mut shmem_id: UserOutPtr<usize>) -> isize {
    shmem_id.write(link_id);
    0
}

pub fn sys_uintr_notice(index: usize) -> isize {
    -1
}

pub fn sys_uintr_uiret() -> isize {
    -1
}