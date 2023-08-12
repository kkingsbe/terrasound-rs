use terrasound::Terrasound;

fn main() {
    let terrasound = Terrasound::new(6968);
    terrasound.start();

    loop {}
}