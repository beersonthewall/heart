use core::ops::{Add, BitOr};

#[derive(Copy, Clone)]
pub struct PhysicalAddress(pub usize);

impl PhysicalAddress {
    pub fn new(address: usize) -> Self {
        // TODO assert validity
        Self(address)
    }
}

impl Add for PhysicalAddress {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl BitOr<u64> for PhysicalAddress {
    type Output = u64;

    fn bitor(self, rhs: u64) -> Self::Output {
        self.0 as u64 | rhs
    }
}

impl Add<usize> for PhysicalAddress {
    type Output = Self;

    fn add(self, other: usize) -> Self::Output {
        Self(self.0 + other)
    }
}

impl core::fmt::Display for PhysicalAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

#[derive(Copy, Clone)]
pub struct VirtualAddress(pub usize);

impl VirtualAddress {
    #[inline]
    pub fn new(address: usize) -> Self {
        // Cannonical virtual address form for x86_64 sign-extends bit 47.
        let mut addr = address;
        if addr & (1 << 47) > 0 {
            addr = addr | (0xFFFF << 48);
        }
        Self(addr)
    }
}

impl BitOr<usize> for VirtualAddress {
    type Output = usize;

    fn bitor(self, rhs: usize) -> Self::Output {
        self.0 | rhs
    }
}
