pub struct Color {
    pub packed: u32
}
impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            packed: ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32),
        }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    pub const fn r(&self) -> u8 {
        ((self.packed >> 24) & 0xFF) as u8
    }
    pub const fn g(&self) -> u8 {
        ((self.packed >> 16) & 0xFF) as u8
    }
    pub const fn b(&self) -> u8 {
        ((self.packed >> 8) & 0xFF) as u8
    }
    pub const fn a(&self) -> u8 {
        (self.packed & 0xFF) as u8
    }

    pub const fn lerp(&self, other: &Self, t: f32) -> Self {
        let r = self.r() as f32;
        let g = self.g() as f32;
        let b = self.b() as f32;
        let a = self.a() as f32;
        Self::new(
            (r + (other.r() as f32 - r) * t) as u8,
            (g + (other.g() as f32 - g) * t) as u8,
            (b + (other.b() as f32 - b) * t) as u8,
            (a + (other.a() as f32 - a) * t) as u8
        )
    }
}

pub struct Colorf {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32
}
impl Colorf {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self{ r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    pub const fn lerp(&self, other: &Self, t: f32) -> Self {
        Self::new(
            self.r + (other.r - self.r) * t,
            self.g + (other.g - self.g) * t,
            self.b + (other.b - self.b) * t,
            self.a + (other.a - self.a) * t
        )
    }
}