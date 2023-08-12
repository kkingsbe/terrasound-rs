pub mod config;
mod audiosource;
mod audiobuffer;
mod metadata;
mod audiosocketserver;

use std::{sync::{Arc, Mutex}, thread};
use audiosocketserver::AudioSocketServer;
use audiosource::AudioSource;
use config::PREBUFFER_SIZE;
use rodio::{OutputStream, Sink, OutputStreamHandle};

pub struct Terrasound {
    is_playing: Arc<Mutex<bool>>,
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    audio_source: Arc<Mutex<AudioSource>>,
    client_connected: Arc<Mutex<bool>>,
    server: AudioSocketServer,
}

impl Terrasound {
    pub fn new(port: u16) -> Terrasound {
        let (_stream, _stream_handle) = OutputStream::try_default().unwrap();

        Terrasound {
            is_playing: Arc::new(Mutex::new(false)),
            stream: _stream,
            stream_handle: _stream_handle.clone(),
            audio_source: Arc::new(Mutex::new(AudioSource { 
                buffers: Vec::new(),
                current_buffer_idx: 0,
                sink: Sink::try_new(&_stream_handle.clone()).unwrap()
            })),
            client_connected: Arc::new(Mutex::new(false)),
            server: AudioSocketServer::new(port),
        }
    }

    pub fn start(&self) {
        let client_connected = self.client_connected.clone();
        let audio_source = self.audio_source.clone();
        self.server.begin(self.client_connected.clone(), self.audio_source.clone());

        thread::spawn(move || {
            let mut num_buffers: usize = 0;
            let mut playing = false;

            loop {
                if !client_connected.lock().unwrap().clone() {
                    playing = false;
                    continue;
                }

                let mut temp_audio_source = audio_source.lock().unwrap();
                let current_prebuffer_size = num_buffers - temp_audio_source.current_buffer_idx;
                let new_num_buffers = temp_audio_source.buffers.len();

                //If the prebuffer is large enough, begin playback
                if !playing && current_prebuffer_size >= PREBUFFER_SIZE {
                    playing = true;
                }
                
                if playing {
                    if temp_audio_source.current_buffer_idx >= new_num_buffers {
                        continue;
                    }

                    temp_audio_source.play_next();
                }
                num_buffers = new_num_buffers;
            }
        });
    }

    pub fn stop(&self) {
        panic!("Not implemented");
    }
}