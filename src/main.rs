mod audiobuffer;
mod metadata;

use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use audiobuffer::AudioBuffer;
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
    let (tx, rx): (Sender<AudioBuffer>, Receiver<AudioBuffer>) = channel();
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let mut server_running: Box<bool> = Box::new(false);
    let mut processed_buffers: Vec<AudioBuffer> = Vec::new();
    start_server(&mut *server_running, tx.clone());
    let mut num_buffers: usize = 0;
    let mut current_buffer_idx: usize = 0;
    while *server_running {
        let new_buff_res = rx.recv();
        if new_buff_res.is_ok() {
            processed_buffers.push(new_buff_res.unwrap());
        }
        
        let current_prebuffer_size = num_buffers - current_buffer_idx;
        let new_num_buffers = processed_buffers.len();
        if current_prebuffer_size >= PREBUFFER_SIZE && new_num_buffers > num_buffers {
            if current_buffer_idx >= new_num_buffers {
                continue;
            }

            let current_buffer = processed_buffers.get(current_buffer_idx).unwrap();

            current_buffer.play(&sink);
            current_buffer_idx += 1;
        }
        num_buffers = new_num_buffers;
    }
}
