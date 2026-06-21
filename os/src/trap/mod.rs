use riscv::register::stvec::Stvec;
mod context;

use core::arch::global_asm;
use core::arch::asm;
use riscv::register::{
    stvec,
    scause,
    stval,
    sie,
};
use crate::syscall::syscall;
use crate::task::{
    exit_current_and_run_next,
    suspend_current_and_run_next,
    current_user_token,
    current_trap_cx,
};
use crate::timer::set_next_trigger;
use crate::config::{TRAP_CONTEXT, TRAMPOLINE};

global_asm!(include_str!("trap.S"));

pub use context::TrapContext;

pub fn init() {
    set_kernel_trap_entry();
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(Stvec::from_bits(trap_from_kernel as usize));
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(Stvec::from_bits(TRAMPOLINE));
    }
}

#[unsafe(no_mangle)]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let cx = current_trap_cx();
    let scause = scause::read();
    let scause_val = scause.bits();
    let stval = stval::read();
    match scause_val {
        8 => {
            cx.sepc += 4;
            let result = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]);
            let cx = current_trap_cx();
            cx.x[10] = result as usize;
        }
        15 | 7 | 1 | 12 | 5 | 13 => {
            println!(
                "[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.",
                scause_val,
                stval,
                current_trap_cx().sepc,
            );
            exit_current_and_run_next(-2);
        }
        2 => {
            println!("[kernel] IllegalInstruction in application, core dumped.");
            exit_current_and_run_next(-3);
        }
        0x8000000000000005 => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!("Unsupported trap {:#x}, stval = {:#x}!", scause_val, stval);
        }
    }
    trap_return();
}

#[unsafe(no_mangle)]
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();
    unsafe extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,
            in("a1") user_satp,
            options(noreturn)
        );
    }
}

#[unsafe(no_mangle)]
pub fn trap_from_kernel() -> ! {
    panic!("a trap from kernel!");
}

pub fn enable_timer_interrupt() {
    unsafe { sie::set_stimer(); }
}
