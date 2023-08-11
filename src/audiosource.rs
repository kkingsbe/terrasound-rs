use rodio::Sink;
use rodio::queue::queue;

use crate::audiobuffer::AudioBuffer;

#[derive(Debug)]
pub enum AudioSourceError {
    NoMoreAudio,
}

pub struct AudioSource {
    pub buffers: Vec<AudioBuffer>,
    pub current_buffer_idx: usize,
    pub sink: Sink,
}

impl AudioSource {
     fn get_next(&mut self) -> Result<&AudioBuffer, AudioSourceError> {
        let buff_res = &self.buffers.get(self.current_buffer_idx);
        if buff_res.is_none() {
            return Err(AudioSourceError::NoMoreAudio);
        }

        self.current_buffer_idx += 1;
        Ok(buff_res.unwrap())
    }

    pub fn add_buffer(&mut self, buffer: AudioBuffer) {
        self.buffers.push(buffer);
    }

    pub fn play_next(&mut self) {
        let next_buffer = self.get_next().unwrap().clone();
        let sink = &self.sink;
        next_buffer.play(sink);
    }
}