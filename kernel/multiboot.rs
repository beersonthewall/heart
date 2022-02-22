#[allow(dead_code)]
#[repr(C)]
pub struct MultibootInfo {
    pub flags: u32,

    // flags[0]
    pub mem_lower: u32,
    pub mem_upper: u32,

    // flags[1]
    pub boot_device: u32,

    // flags[2]
    pub cmdline: u32,

    // flags[3]
    pub mods_count: u32,
    pub mods_addr: u32,

    // flags[4] or flags[5]
    pub syms: [u8; 12],

    // flags[6]
    pub mmap_length: u32,
    pub mmap_addr: u32,

    // flags[7]
    pub drives_length: u32,
    pub drives_addr: u32,

    // flags[8]
    pub config_table: u32,

    // flags[9]
    pub boot_loader_name: u32,

    // flags[10]
    pub apm_table: u32,

    // flags[11]
    pub vbe_control_info: u32,
    pub vbe_mode_info: u32,
    pub vbe_mode: u16,
    pub vbe_interface_seg: u16,
    pub vbe_interface_off: u16,
    pub vbe_interface_len: u16,

    // flags[12]
    pub framebuffer_addr: u64,
    pub framebuffer_pitch: u32,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub framebuffer_bpp: u8,
    pub framebuffer_type: u8,
    pub color_info: [u8; 5],
}

#[allow(dead_code)]
impl MultibootInfo {
    pub fn flags(&self) -> u32 {
        self.flags
    }
}
