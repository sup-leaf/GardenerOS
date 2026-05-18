use riscv::register::sstatus;

#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: usize,
    pub sepc: usize,
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) { self.x[2] = sp; }
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        let sstatus_val = sstatus::read().bits();
        // 设置 SPP 为 User 模式 (清除第 8 位)
        let sstatus_val = sstatus_val & !(1 << 8);
        let mut cx = Self {
            x: [0; 32],
            sstatus: sstatus_val,
            sepc: entry,
        };
        cx.set_sp(sp);
        cx
    }
}
