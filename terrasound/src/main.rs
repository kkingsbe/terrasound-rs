mod audiosource;
mod audiobuffer;
mod metadata;
mod audiosocketserver;

use std::sync::{Arc, Mutex};
use audiosocketserver::AudioSocketServer;
use audiosource::AudioSource;
use rodio::{OutputStream, Sink};

const PREBUFFER_SIZE: usize = 5;

fn main() {
    //Create the audio stream & sink
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    //Create the audio source which stores the buffers
    let audio_source: Arc<Mutex<AudioSource>> = Arc::new(Mutex::new(AudioSource { 
        buffers: Vec::new(),
        current_buffer_idx: 0,
        sink,
    }));
    
    let client_connected: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    //Start the TCP socket server
    let server: AudioSocketServer = AudioSocketServer::new(6968);
    server.begin(client_connected.clone(), audio_source.clone());

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
}
