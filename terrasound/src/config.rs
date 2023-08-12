pub const AUDIO_BUFFER_SIZE: usize = 2048;
pub const METADATA_SIZE: usize = 2 * std::mem::size_of::<i32>();
pub const BUFFER_SIZE: usize = (AUDIO_BUFFER_SIZE * std::mem::size_of::<f32>()) + METADATA_SIZE;
pub const PREBUFFER_SIZE: usize = 5;