use terrasound::Terrasound;

fn main() {
    let mut terrasound = Terrasound::new_port_in_speaker_out(6967);
    terrasound.start();

    loop {}
}