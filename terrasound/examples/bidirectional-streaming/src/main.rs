use resonator::{
    client::resonatorclient::ResonatorClient, common::audiobuffer::AudioBuffer,
    server::resonator::ResonatorServer,
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration
};
use terrasound::Terrasound;

fn create_peer(
    port: u16,
    juce_audio: Arc<Mutex<Vec<AudioBuffer>>>,
    resonator_audio: Arc<Mutex<Vec<AudioBuffer>>>,
    data_check: Vec<AudioBuffer>,
    stop_signal: Arc<AtomicBool>,
) {
    thread::spawn(move || {
        let mut terrasound_piro = Terrasound::new_port_in_resonator_out(port, juce_audio.clone());
        terrasound_piro.start();

        let mut terrasound_riso = Terrasound::new_resonator_in_speaker_out(resonator_audio.clone());
        terrasound_riso.start();

        // Wait for the network
        thread::sleep(Duration::from_secs(1));

        println!(
            "Peer at port {} received audio length: {}",
            port,
            resonator_audio.lock().unwrap().len()
        );
        assert!(resonator_audio.lock().unwrap().eq(&data_check));

        // Wait before stoping the main thread
        thread::sleep(Duration::from_millis(1000));

        stop_signal.store(true, Ordering::SeqCst);
    });
}

fn main() {
    const SAMPLES: i32 = 2048;
    const SAMPLE_RATE: i32 = 22050;

    let mut data = Vec::new();
    data.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    data.extend_from_slice(&SAMPLES.to_le_bytes());

    // Data from peer 1 has all 0's
    let mut buf1 = data.clone();
    buf1.extend_from_slice(&vec![0; SAMPLES as usize * 4]);

    // While data from peer 2 has all 1's
    let mut buf2 = data;
    buf2.extend_from_slice(&vec![1; SAMPLES as usize * 4]);

    // Should have different ports for binding to and listening from
    let peer1_port = 6967;
    let peer2_port = 6968;

    let stop_signal1 = Arc::new(AtomicBool::new(false));
    let stop_signal2 = Arc::new(AtomicBool::new(false));

    // Inputs
    let juce_audio1: Arc<Mutex<Vec<AudioBuffer>>> =
        Arc::new(Mutex::new(vec![AudioBuffer::from_bytes(
            &buf1.try_into().unwrap(),
        )]));
    let juce_audio2: Arc<Mutex<Vec<AudioBuffer>>> =
        Arc::new(Mutex::new(vec![AudioBuffer::from_bytes(
            &buf2.try_into().unwrap(),
        )]));

    // Outputs
    let resonator_audio1: Arc<Mutex<Vec<AudioBuffer>>> = Arc::new(Mutex::new(Vec::new()));
    let resonator_audio2: Arc<Mutex<Vec<AudioBuffer>>> = Arc::new(Mutex::new(Vec::new()));

    // Audio data from peer 2 will be received by peer 1 and vice versa
    let check_data1 = juce_audio2.lock().unwrap().clone();
    let check_data2 = juce_audio1.lock().unwrap().clone();

    // Create the Terrasound peers
    create_peer(
        peer1_port,
        juce_audio1.clone(),
        resonator_audio1.clone(),
        check_data1,
        stop_signal1.clone(),
    );
    create_peer(
        peer2_port,
        juce_audio2.clone(),
        resonator_audio2.clone(),
        check_data2,
        stop_signal2.clone(),
    );

    // Initialize Resonator server

    let send_port = 8080;
    let recv_port = 8081;

    let server = ResonatorServer::new(send_port, recv_port);
    server.begin();

    // Then initialize the clients

    let client1_id = 1;
    let client2_id = 2;

    let client1: ResonatorClient = ResonatorClient::new(
        send_port,
        recv_port,
        client1_id,
        client1_id,
        Some(juce_audio1.clone()),
        Some(resonator_audio2.clone()),
    );
    client1.begin();

    let client2: ResonatorClient = ResonatorClient::new(
        send_port,
        recv_port,
        client2_id,
        client2_id,
        Some(juce_audio2.clone()),
        Some(resonator_audio1.clone()),
    );
    client2.begin();

    while !stop_signal1.load(Ordering::SeqCst) && !stop_signal2.load(Ordering::SeqCst) {}

    println!("Terrasound terminating\n\n");
    thread::sleep(Duration::from_millis(2000));
}
