use rodio::Sink;

use crate::{audiobuffer::AudioBuffer, TerrasoundSource};

#[derive(Debug)]
pub enum AudioSourceError {
    NoMoreAudio,
}

pub struct AudioSource {
    pub buffers: Vec<AudioBuffer>,
    pub current_buffer_idx: usize,
    pub sink: Sink,
}

impl TerrasoundSource for AudioSource {
    fn get_next(&mut self) -> Result<AudioBuffer, AudioSourceError> {
        let buff_res = &self.buffers.get(self.current_buffer_idx);
        if buff_res.is_none() {
            return Err(AudioSourceError::NoMoreAudio);
        }

        self.current_buffer_idx += 1;
        Ok(buff_res.unwrap().clone())
    }

    fn play_next(&mut self) {
        let next_buffer = self.get_next().unwrap().clone();
        let sink = &self.sink;
        next_buffer.play(sink);
    }
}

impl AudioSource {
    pub fn add_buffer(&mut self, buffer: AudioBuffer) {
        self.buffers.push(buffer);
    }
}