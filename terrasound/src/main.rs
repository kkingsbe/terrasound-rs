use terrasound::Terrasound;

fn main() {
    let terrasound = Terrasound::new(6967);
    terrasound.start();

    loop {}
}