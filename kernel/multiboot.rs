#[repr(C)]
pub struct MultibootHeader {
    
}

#[repr(C)]
pub struct MultibootInfo {
    flags: u32,

    // flags[0]
    mem_lower: u32,
    mem_upper: u32,

    // flags[1]
    boot_device: u32,

    // flags[2]
    cmdline: u32,

    // flags[3]
    mods_count: u32,
    mods_addr: u32,

    // flags[4] or flags[5]
    syms: [u8;12],

    // flags[6]
    mmap_length: u32,
    mmap_addr: u32,

    // flags[7]
    drives_length: u32,
    drives_addr: u32,

    // flags[8]
    config_table: u32,

    // flags[9]
    boot_loader_name: u32,

    // flags[10]
    apm_table: u32,

    // flags[11]
    vbe_control_info: u32,
    vbe_mode_info: u32,
    vbe_mode: u16,
    vbe_interface_seg: u16,
    vbe_interface_off: u16,
    vbe_interface_len: u16,

    // flags[12]
    framebuffer_addr: u64,
    framebuffer_pitch: u32,
    framebuffer_width: u32,
    framebuffer_height: u32,
    framebuffer_bpp: u8,
    framebuffer_type: u8,
    color_info: [u8;5],
}

impl MultibootInfo {
    pub fn flags(&self) -> u32 {
        self.flags
    }
}
