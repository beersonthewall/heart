use bit_field::BitField;
use core::arch::asm;

const APIC_MSR: u32 = 0x0000_001B;

pub fn init_boot_apic() {
    // Check and see if APIC is enabled
    assert!(apic_enabled());

    /*
    Steps:
    - reset APIC to known state
    - enable APIC timer
    - reset APIC timer counter
    - wait specific amount of time measured by a different clock
    - get number of ticks from APIC timer counter
    - adjust it to a second
    - divide by the quantum of your choice (results X)
    - make APIC timer fire an interrupt every X ticks
     */

    let apic_register_value = read_apic_msr();
    log!("apic msr register value: 0x{:x}", apic_register_value);

    let is_bootstrap_cpu = apic_register_value.get_bit(8);
    let is_enabled = apic_register_value.get_bit(11);
    let apic_base_address: usize = apic_register_value.get_bits(12..52);

    assert!(is_enabled);
    assert!(is_bootstrap_cpu);
    log!("APIC base address 0x{:x}", apic_base_address);

    let apic_base_address = apic_base_address as *mut u8;

    /*
     * APIC registers are aligned to 16-byte offsets and must be accessed using naturally-aligned
     * DWORD size read and writes. All other accesses cause undefined behavior.
     */

    let spurious = APICRegister::read(APICRegister::SpuriousInterruptVector, apic_base_address);
    APICRegister::write(
        APICRegister::SpuriousInterruptVector,
        apic_base_address,
        spurious | 1 << 8 | 32,
    );

    APICRegister::write(
        APICRegister::TimerDivideConfiguration,
        apic_base_address,
        0x3,
    );

    pit_prepare_sleep(1000);

    let init: u32 = 0xFFFFFFFF;
    APICRegister::write(APICRegister::TimerInitialCount, apic_base_address, init);

    pit_perform_sleep();

    let ticks_in_10_ms =
        init - APICRegister::read(APICRegister::TimerInitialCount, apic_base_address);

    APICRegister::write(
        APICRegister::TimerLVTEntry,
        apic_base_address,
        (1 << 16) | 32,
    );

    unsafe {
        asm!("sti");
    }
}

enum APICRegister {
    Id,
    Version,
    TPR,
    APR,
    PPR,
    EOI,
    RemoteRead,
    LDR,
    DFR,
    SpuriousInterruptVector,
    ISR,
    TMR,
    IRR,
    ESR,
    IntCmdLow,
    IntCmdHigh,
    TimerLVTEntry,
    ThermalLVTEntry,
    PerfCounterLVTEntry,
    LocalIntZeroVTE,
    LocalIntOneVTE,
    ErrorVTE,
    TimerInitialCount,
    TimerCurrentCount,
    TimerDivideConfiguration,
    ExtendedAPICFeature,
    ExtendedAPICControl,
    SEOI,
    IER,
    ExtendedInterruptLVT,
}

impl APICRegister {
    fn offset(&self) -> isize {
        match self {
            APICRegister::Id => 0x20,
            APICRegister::Version => 0x30,
            APICRegister::TPR => 0x80,
            APICRegister::APR => 0x90,
            APICRegister::PPR => 0xA0,
            APICRegister::EOI => 0xB0,
            APICRegister::RemoteRead => 0xC0,
            APICRegister::LDR => 0xD0,
            APICRegister::DFR => 0xE0,
            APICRegister::SpuriousInterruptVector => 0xF0,
            APICRegister::ISR => 0x100,
            APICRegister::TMR => 0x180,
            APICRegister::IRR => 0x200,
            APICRegister::ESR => 0x280,
            APICRegister::IntCmdLow => 0x300,
            APICRegister::IntCmdHigh => 0x310,
            APICRegister::TimerLVTEntry => 0x320,
            APICRegister::ThermalLVTEntry => 0x330,
            APICRegister::PerfCounterLVTEntry => 0x340,
            APICRegister::LocalIntZeroVTE => 0x350,
            APICRegister::LocalIntOneVTE => 0x360,
            APICRegister::ErrorVTE => 0x370,
            APICRegister::TimerInitialCount => 0x380,
            APICRegister::TimerCurrentCount => 0x390,
            APICRegister::TimerDivideConfiguration => 0x3E0,
            APICRegister::ExtendedAPICFeature => 0x400,
            APICRegister::ExtendedAPICControl => 0x410,
            APICRegister::SEOI => 0x420,
            APICRegister::IER => 0x480,
            APICRegister::ExtendedInterruptLVT => 0x500,
        }
    }

    fn read(register: APICRegister, apic_base_addr: *const u8) -> u32 {
        unsafe {
            let register: *const u32 = apic_base_addr.offset(register.offset()) as *const u32;
            register.read()
        }
    }

    fn write(register: APICRegister, apic_base_addr: *mut u8, value: u32) {
        unsafe {
            let register: *mut u32 = apic_base_addr.offset(register.offset()) as *mut u32;
            register.write(value);
        }
    }
}

fn read_apic_msr() -> usize {
    let reg_high: u32;
    let reg_low: u32;
    unsafe {
        asm!(
            "mov ecx, {msr}",
            "rdmsr",
            msr = const APIC_MSR,
            lateout("edx") reg_high,
            lateout("eax") reg_low,
        );
    }
    ((reg_high as usize) << 32) | (reg_low as usize)
}

fn write_apic_msr(value: usize) {
    unsafe {}
}

fn apic_enabled() -> bool {
    let edx: u32;
    const APIC_CPUID_FUNCTION_NUMBER: u32 = 0x0000_0001;
    unsafe {
        asm!("cpuid", in("eax") APIC_CPUID_FUNCTION_NUMBER, lateout("edx") edx);
    }
    edx.get_bit(9)
}

fn pit_prepare_sleep(_ms: u32) {}

fn pit_perform_sleep() {}
