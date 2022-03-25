pub const USER_ASPACE_BASE: usize = 0;
pub const USER_ASPACE_SIZE: usize = 0x7fff_ffff_f000;
pub const KERNEL_ASPACE_BASE: usize = 0xffff_ff80_0000_0000;
pub const KERNEL_ASPACE_SIZE: usize = 0x0000_007f_ffff_f000;

pub const PHYS_VIRT_OFFSET: usize = 0xffff_ff80_0000_0000;
