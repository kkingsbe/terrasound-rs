#[derive(Debug)]
pub struct Metadata {
    pub sample_rate: i32,
    pub buffer_size: i32,
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            sample_rate: 0,
            buffer_size: 0,
        }
    }
}