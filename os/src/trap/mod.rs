mod context;

use core::arch::global_asm;
use riscv::register::{stvec, scause, stval};
use riscv::register::stvec::Stvec;
use crate::syscall::syscall;
use crate::batch::run_next_app;

global_asm!(include_str!("trap.S"));

pub use context::TrapContext;

pub fn init() {
    unsafe extern "C" { fn __alltraps(); }
    unsafe {
        let stvec_val = Stvec::from_bits(__alltraps as usize);
        stvec::write(stvec_val);
    }
}

#[unsafe(no_mangle)]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause_val = scause::read().bits();
    let stval_val = stval::read();
    
    match scause_val {
        8 => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        15 | 7 => {
            println!("[kernel] PageFault in application, core dumped.");
            run_next_app();
        }
        2 => {
            println!("[kernel] IllegalInstruction in application, core dumped.");
            run_next_app();
        }
        _ => {
            panic!("Unsupported trap {}, stval = {:#x}!", scause_val, stval_val);
        }
    }
    cx
}
