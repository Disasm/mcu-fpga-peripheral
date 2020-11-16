use icecompr::*;

fn main() {
    let target = include_bytes!("../data/example_8k.bin");
    let compressed = include_bytes!("../data/example_8k.cmp");

    let mut decoder = Decoder::new(compressed);
    let mut decompressed = Vec::new();
    loop {
        let mut buf = [0u8; 64];
        let n = decoder.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        decompressed.extend_from_slice(&buf[..n]);
    }

    std::fs::write("1.dec", &decompressed).unwrap();

    assert_eq!(decompressed.len(), target.len());
    let n = decompressed.iter().zip(target.iter()).filter(|&(a, b)| a == b).count();
    assert_eq!(n, decompressed.len());
}
