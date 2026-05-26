pub struct PendingTranscodeItem {
    pub file_name: String,
    pub size_bytes: u64,
    pub height: u32,
    pub width: u32,
    pub bits_per_pixel: f64,
    pub compression_potential: f64,
    pub crf: u8,
}

impl PendingTranscodeItem {
    pub fn estimated_output_bytes(&self) -> u64 {
        let ratio = (0.04 / self.bits_per_pixel).min(1.0);
        (self.size_bytes as f64 * ratio) as u64
    }

    pub fn estimated_gain_bytes(&self) -> i64 {
        self.size_bytes as i64 - self.estimated_output_bytes() as i64
    }
}
