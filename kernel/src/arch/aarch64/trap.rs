use core::arch::global_asm;

use cortex_a::registers::{ESR_EL1, FAR_EL1, VBAR_EL1};
use tock_registers::interfaces::{Readable, Writeable};

use super::TrapFrame;
use crate::drivers::interrupt::{handle_irq, IrqHandlerResult};
use crate::{syscall::syscall, task::CurrentTask};

global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" {
        fn exception_vector_base();
    }
    VBAR_EL1.set(exception_vector_base as usize as _);
}

#[repr(u8)]
#[derive(Debug)]
#[allow(dead_code)]
enum TrapKind {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[repr(u8)]
#[derive(Debug)]
#[allow(dead_code)]
enum TrapSource {
    CurrentSpEl0 = 0,
    CurrentSpElx = 1,
    LowerAArch64 = 2,
    LowerAArch32 = 3,
}

#[no_mangle]
fn invalid_exception(tf: &mut TrapFrame, kind: TrapKind, source: TrapSource) {
    panic!(
        "Invalid exception {:?} from {:?}:\n{:#x?}",
        kind, source, tf
    );
}

#[no_mangle]
fn handle_sync_exception(tf: &mut TrapFrame) {
    let esr = ESR_EL1.extract();
    trace!(
        "Trap @ {:#x}: ESR = {:#x} (EC {:#08b}, ISS {:#x})\n{:#x?}",
        tf.elr,
        esr.get(),
        esr.read(ESR_EL1::EC),
        esr.read(ESR_EL1::ISS),
        tf
    );
    match esr.read_as_enum(ESR_EL1::EC) {
        Some(ESR_EL1::EC::Value::Unknown) => {
            warn!("Unknown exception @ {:#x}, kernel killed it.", tf.elr);
            CurrentTask::get().exit(-1);
        }
        Some(ESR_EL1::EC::Value::SVC64) => {
            tf.r[0] = syscall(tf, tf.r[8] as _, tf.r[0] as _, tf.r[1] as _, tf.r[2] as _) as u64
        }
        Some(ESR_EL1::EC::Value::DataAbortLowerEL)
        | Some(ESR_EL1::EC::Value::InstrAbortLowerEL) => {
            let iss = esr.read(ESR_EL1::ISS);
            warn!(
                "Page Fault @ {:#x}, FAR={:#x}, ISS={:#x}, kernel killed it.",
                tf.elr,
                FAR_EL1.get(),
                iss
            );
            CurrentTask::get().exit(-1);
        }
        Some(ESR_EL1::EC::Value::DataAbortCurrentEL)
        | Some(ESR_EL1::EC::Value::InstrAbortCurrentEL) => {
            let iss = esr.read(ESR_EL1::ISS);
            panic!(
                "Kernel Page Fault @ {:#x}, FAR={:#x}, ISS={:#x}, kernel killed it.",
                tf.elr,
                FAR_EL1.get(),
                iss
            );
        }
        _ => {
            panic!(
                "Unsupported synchronous exception @ {:#x}: ESR={:#x} (EC {:#08b}, ISS {:#x})",
                tf.elr,
                esr.get(),
                esr.read(ESR_EL1::EC),
                esr.read(ESR_EL1::ISS),
            );
        }
    }
}

#[no_mangle]
fn handle_irq_exception(_tf: &mut TrapFrame) {
    if handle_irq(0) == IrqHandlerResult::Reschedule {
        CurrentTask::get().yield_now();
    }
}
