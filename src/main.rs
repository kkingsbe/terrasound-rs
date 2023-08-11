mod audiosource;
mod audiobuffer;
mod metadata;
mod audiosocketserver;

use std::sync::mpsc::{channel, Sender, Receiver};
use audiobuffer::AudioBuffer;
use audiosocketserver::AudioSocketServer;
use audiosource::AudioSource;
use rodio::{OutputStream, Sink};

const PREBUFFER_SIZE: usize = 10;

fn main() {
    //Create MPSC chanel
    let (tx, rx): (Sender<AudioBuffer>, Receiver<AudioBuffer>) = channel();
    
    //Create the audio stream & sink
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    //Create the audio source which stores the buffers
    let mut audio_source = AudioSource { 
        buffers: Vec::new(),
        current_buffer_idx: 0,
        sink,
    };
    
    let mut server_running: Box<bool> = Box::new(false);

    //Start the TCP server
    let server: AudioSocketServer = AudioSocketServer::new(6968);
    server.begin(&mut *server_running, tx.clone());

    let mut num_buffers: usize = 0;
    let mut playing = false;

    while *server_running {
        //Recieve a new buffer if able
        let new_buff_res = rx.recv();
        if new_buff_res.is_ok() {
            //Add the buffer to the audio source
            audio_source.add_buffer(new_buff_res.unwrap());
        }
        
        let current_prebuffer_size = num_buffers - audio_source.current_buffer_idx;
        let new_num_buffers = audio_source.buffers.len();

        //If the prebuffer is large enough, begin playback
        if !playing && current_prebuffer_size >= PREBUFFER_SIZE {
            playing = true;
        }

        if playing {
            if audio_source.current_buffer_idx >= new_num_buffers {
                continue;
            }

            audio_source.play_next();
        }
        num_buffers = new_num_buffers;
    }
}
