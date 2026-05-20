use crate::domain::DomainError;

pub struct Resolution {
    height: u32,
    width: u32,
}

impl Resolution {
    pub fn new(height: u32, width: u32) -> Result<Self, DomainError> {
        if height == 0 || width == 0 {
            return Err(DomainError::InvalidResolution);
        }

        Ok(Self { height, width })
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn is_4k(&self) -> bool {
        self.width >= 3840
    }

    pub fn is_hd(&self) -> bool {
        self.width >= 1280
    }

    pub fn pixel_count(&self) -> u64 {
        self.height as u64 * self.width as u64
    }

    pub fn aspect_ration(&self) -> f64 {
        self.width as f64 / self.height as f64
    }
}
