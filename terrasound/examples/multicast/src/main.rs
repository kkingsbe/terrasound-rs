use resonator::{
    client::resonatorclient::ResonatorClient,
    common::audiobuffer::AudioBuffer as ResonatorAudioBuffer, server::resonator::ResonatorServer,
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
use terrasound::{audiobuffer::AudioBuffer, Terrasound};

fn create_sender(
    port: u16,
    juce_audio: Arc<Mutex<Vec<ResonatorAudioBuffer>>>,
    stop_signal: Arc<AtomicBool>,
) {
    thread::spawn(move || {
        let mut terrasound_piro = Terrasound::new_port_in_resonator_out(port, juce_audio.clone());
        terrasound_piro.start();

        // Wait for the network
        thread::sleep(Duration::from_secs(1));

        while !stop_signal.load(Ordering::SeqCst) {}
    });
}

fn create_receiver(
    resonator_audio: Arc<Mutex<Vec<ResonatorAudioBuffer>>>,
    stop_signal: Arc<AtomicBool>,
) {
    thread::spawn(move || {
        let mut terrasound_riso = Terrasound::new_resonator_in_speaker_out(resonator_audio.clone());
        terrasound_riso.start();

        // Wait for the network
        thread::sleep(Duration::from_secs(1));

        while !stop_signal.load(Ordering::SeqCst) {}
    });
}

fn main() {
    let sender_id = 1;
    let sender_port = 6967;
    let send_port = 8080;
    let recv_port = 8081;
    let num_receivers = 4;

    let stop_signal = Arc::new(AtomicBool::new(false));

    // Initialize Resonator server
    let server = ResonatorServer::new(send_port, recv_port);
    server.begin();

    {
        // Create the terrasound sender
        const P1_FILENAME: &str = "sound/256999__paul-evans__short-bach-melody.wav";
        let mut d1 = AudioBuffer::buffers_from_file(P1_FILENAME);
        let buf1: Vec<_> = d1.drain(..).map(|b| b.to_resonator()).collect();

        let juce_audio: Arc<Mutex<Vec<ResonatorAudioBuffer>>> = Arc::new(Mutex::new(buf1));

        for id_offset in 1..(num_receivers + 1) {
            let receiver_id = sender_id + id_offset;
            let sender: ResonatorClient = ResonatorClient::new(
                send_port,
                recv_port,
                sender_id,
                receiver_id,
                Some(juce_audio.clone()),
                None,
            );
            sender.begin();
        }

        create_sender(sender_port, juce_audio.clone(), stop_signal.clone());
    }

    // Create the receivers
    for id_offset in 1..(num_receivers + 1) {
        let receiver_id = sender_id + id_offset;
        let resonator_audio = Arc::new(Mutex::new(Vec::new()));
        let listener: ResonatorClient = ResonatorClient::new(
            send_port,
            recv_port,
            receiver_id,
            sender_id,
            None,
            Some(resonator_audio.clone()),
        );
        listener.begin();

        create_receiver(resonator_audio.clone(), stop_signal.clone());
    }

    // Wait for the audio to finish playing
    thread::sleep(Duration::from_secs(12));
    stop_signal.store(true, Ordering::SeqCst);

    println!("Terrasound terminating\n\n");
    thread::sleep(Duration::from_secs(1));
}
