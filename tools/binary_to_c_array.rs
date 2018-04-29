use std::io::Read;

fn main() {
    let mut data: Vec<u8> = Vec::new();

    std::io::stdin().read_to_end(&mut data).unwrap();
    let out = data.iter()
        .map(|b| format!("0x{:02x}", b))
        .fold("".to_string(), |a, b| format!("{},{}", a, b));

    println!("{{{}}}", &out[1..]);
}
