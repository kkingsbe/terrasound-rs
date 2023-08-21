use terrasound::Terrasound;

fn main() {
    let mut terrasound = Terrasound::new(6967);
    terrasound.start();

    loop {}
}