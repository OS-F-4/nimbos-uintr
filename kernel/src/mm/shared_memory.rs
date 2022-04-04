use alloc::vec::Vec;
use crate::mm::{PAGE_SIZE, PhysAddr, PhysFrame};
use crate::sync::{LazyInit, UPSafeCell};

const IPC_PRIVATE: usize = 0;

const SHM_MAX_PAGE_NUM: usize = 256;

struct ShmSeg {
    creator_pid: usize,
    valid: bool,
    key: usize,
    page_frames: Vec<PhysFrame>,
}

impl ShmSeg {
    pub fn zero_init() -> Self {
        Self {
            creator_pid: 0,
            valid: false,
            key: 0,
            page_frames: Vec::new(),
        }
    }
}

pub struct ShmSegManager {
    shm_segments: UPSafeCell<Vec<ShmSeg>>,
}

pub static SHM_SEG_MANAGER: LazyInit<ShmSegManager> = LazyInit::new();

pub fn init_shared_memory() {
    SHM_SEG_MANAGER.init_by(ShmSegManager {
        shm_segments: unsafe { UPSafeCell::new(Vec::new()) },
    });
}

pub fn create_shm_seg(creator_pid: usize, key: usize, size: usize, shmflg: usize) -> isize {
    if key != IPC_PRIVATE {
        panic!("Unsupported key in syscall `shmget`! Currently only key = IPC_PRIVATE is supported.");
    }

    let page_num = (size + PAGE_SIZE - 1) / PAGE_SIZE;
    if page_num > SHM_MAX_PAGE_NUM || page_num <= 0 {
        return -1;
    }

    let mut segments = SHM_SEG_MANAGER.shm_segments.exclusive_access();
    let mut shmid = segments.len();
    for i in 0..segments.len() {
        if !segments[i].valid {
            shmid = i;
            break;
        }
    }
    if shmid == segments.len() {
        segments.push(ShmSeg::zero_init());
    }

    segments[shmid].creator_pid = creator_pid;
    segments[shmid].valid = true;
    segments[shmid].key = key;
    segments[shmid].page_frames.clear();
    for _ in 0..page_num {
        segments[shmid].page_frames.push(PhysFrame::alloc_zero().unwrap());
    }

    shmid as isize
}

pub fn get_shm_seg_paddr_vec(shmid: usize, shmaddr: usize, shmflg: usize) -> Option<Vec<PhysAddr>> {
    if shmaddr != 0 {
        panic!("Unsupported shmaddr in syscall `shmat`! Currently only shmaddr = 0 is supported.");
    }

    let segments = SHM_SEG_MANAGER.shm_segments.exclusive_access();
    if !segments[shmid].valid {
        return None;
    }

    let mut paddr_vec = Vec::new();
    for i in 0..segments[shmid].page_frames.len() {
        paddr_vec.push(segments[shmid].page_frames[i].start_paddr());
    }

    Some(paddr_vec)
}