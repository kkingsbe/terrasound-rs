mod audiobuffer;
mod metadata;

use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use audiobuffer::AudioBuffer;
use metadata::Metadata;
use rodio::{OutputStream, Sink};

const audio_buffer_size: usize = 2048;
const metadata_size: usize = 2 * std::mem::size_of::<i32>();
const buffer_size: usize = (audio_buffer_size * std::mem::size_of::<f32>()) + metadata_size;
const prebuffer_size: usize = 10;

fn handle_client(mut tcp_stream: TcpStream, processed_buffers: &Arc<Mutex<Vec<AudioBuffer>>>, metadata: &Arc<Mutex<Metadata>>) {
    let mut buffer = [0; buffer_size];

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
        let metadata_bytes = &bytes[..metadata_size];
        let audio_bytes = &bytes[metadata_size..];

        let m_buff_size = i32::from_le_bytes(metadata_bytes[0..4].try_into().unwrap());
        let m_sample_rate = i32::from_le_bytes(metadata_bytes[4..8].try_into().unwrap());
        metadata.lock().unwrap().buffer_size = m_buff_size;
        metadata.lock().unwrap().sample_rate = m_sample_rate;

        let samples: Vec<f32> = audio_bytes.chunks_exact(4).map(|chunk| {
            f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
        })
        .collect();

        processed_buffers.lock().unwrap().push({
            AudioBuffer {
                samples,
                metadata: Metadata {
                    sample_rate: m_sample_rate,
                    buffer_size: m_buff_size,
                }
            }
        })
    }
}

fn start_server(server_running: &mut bool, processed_buffers: &Arc<Mutex<Vec<AudioBuffer>>>, metadata: &Arc<Mutex<Metadata>>) {
    *server_running = true;
    let listener: TcpListener = TcpListener::bind("127.0.0.1:6968").unwrap();
    let server_running_clone = server_running.clone();
    let processed_buffers_clone = processed_buffers.clone();
    let metadata_clone = metadata.clone();

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(tcp_stream) => {
                    let server_running_clone = server_running_clone.clone(); // Clone for the closure
                    let processed_buffers_clone = processed_buffers_clone.clone(); // Clone for the closure
                    let metadata_clone = metadata_clone.clone(); // Clone for the closure
                    thread::spawn(move || {
                        handle_client(tcp_stream, &processed_buffers_clone, &metadata_clone);
                    });
                }
                Err(e) => eprintln!("Error accepting connection: {}", e),
            }
        }
    });
}

fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let mut server_running: Box<bool> = Box::new(false);
    let processed_buffers: Arc<Mutex<Vec<AudioBuffer>>> = Arc::new(Mutex::new(Vec::new()));
    let metadata: Arc<Mutex<Metadata>> = Arc::new(Mutex::new(Metadata {
        ..Default::default()
    }));
    start_server(&mut *server_running, &processed_buffers, &metadata);
    let mut num_buffers: usize = 0;
    let mut current_buffer_idx: usize = 0;
    while *server_running {
        let current_prebuffer_size = num_buffers - current_buffer_idx;
        let new_num_buffers = processed_buffers.lock().unwrap().len();
        if current_prebuffer_size >= prebuffer_size && new_num_buffers > num_buffers {
            let temp_processed_buffers = processed_buffers.lock().unwrap();
            if current_buffer_idx >= temp_processed_buffers.len() {
                continue;
            }

            let current_buffer = temp_processed_buffers.get(current_buffer_idx).unwrap();

            current_buffer.play(&sink);
            current_buffer_idx += 1;
        }
        num_buffers = new_num_buffers;
    }
}
