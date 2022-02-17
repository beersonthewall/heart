use core::ops::Add;

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

impl Add<usize> for PhysicalAddress {
    type Output = Self;

    fn add(self, other: usize) -> Self::Output {
        Self(self.0 + other)
    }
}

#[derive(Copy, Clone)]
pub struct VirtualAddress(pub usize);

impl VirtualAddress {
    pub fn new(address: usize) -> Self {
        Self(address)
    }
}

#[derive(Copy, Clone)]
pub struct PhysicalAddressRange {
    pub base: PhysicalAddress,
    pub size: usize,
}

impl PhysicalAddressRange {
    pub fn new(base: PhysicalAddress, size: usize) -> Self {
        Self { base, size }
    }

    pub fn end(&self) -> PhysicalAddress {
        self.base + self.size
    }
}

#[derive(Copy, Clone)]
pub struct VirtualAddressRange {
    pub base: VirtualAddress,
    pub size: usize,
}

impl VirtualAddressRange {
    pub fn new(base: VirtualAddress, size: usize) -> Self {
        Self { base, size }
    }
}
