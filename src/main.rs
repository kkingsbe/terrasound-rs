mod audiosource;
mod audiobuffer;
mod metadata;

use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use audiobuffer::AudioBuffer;
use audiosource::AudioSource;
use metadata::Metadata;
use rodio::{OutputStream, Sink};

const AUDIO_BUFFER_SIZE: usize = 2048;
const METADATA_SIZE: usize = 2 * std::mem::size_of::<i32>();
const BUFFER_SIZE: usize = (AUDIO_BUFFER_SIZE * std::mem::size_of::<f32>()) + METADATA_SIZE;
const PREBUFFER_SIZE: usize = 10;

fn handle_client(mut tcp_stream: TcpStream, tx: Sender<AudioBuffer>) {
    let mut buffer = [0; BUFFER_SIZE];

    loop {
        let size = match tcp_stream.read(&mut buffer) {
            Ok(size) if size == 0 => {
                println!("Client closed connection");
                break;
            }
            Ok(size) => size,
            Err(_) => {
                println!("Client error");
                break;
            }
        };

        let bytes = &buffer[..];
        let metadata_bytes = &bytes[..METADATA_SIZE];
        let audio_bytes = &bytes[METADATA_SIZE..];

        let m_buff_size = i32::from_le_bytes(metadata_bytes[0..4].try_into().unwrap());
        let m_sample_rate = i32::from_le_bytes(metadata_bytes[4..8].try_into().unwrap());

        let samples: Vec<f32> = audio_bytes.chunks_exact(4).map(|chunk| {
            f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
        })
        .collect();

        let new_buff = AudioBuffer {
            samples,
            metadata: Metadata {
                sample_rate: m_sample_rate,
                buffer_size: m_buff_size,
            }
        };

        tx.send(new_buff).unwrap();
    }
}

fn start_server(server_running: &mut bool, tx: Sender<AudioBuffer>) {
    *server_running = true;
    let listener: TcpListener = TcpListener::bind("127.0.0.1:6968").unwrap();
    let server_running_clone = server_running.clone();

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(tcp_stream) => {
                    let server_running_clone = server_running_clone.clone(); // Clone for the closure
                    let tx = tx.clone();
                    thread::spawn(move || {
                        handle_client(tcp_stream, tx);
                    });
                }
                Err(e) => eprintln!("Error accepting connection: {}", e),
            }
        }
    });
}

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
    start_server(&mut *server_running, tx.clone());
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
