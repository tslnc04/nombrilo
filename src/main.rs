use std::fs;

use flate2::read::GzDecoder;
use minecraft_tokei::nbt;

fn main() {
    let filename = "level.dat";
    let file = fs::File::open(filename).unwrap();
    let mut gz = GzDecoder::new(file);
    let (name, data) = nbt::Tag::parse_fully_formed(&mut gz).unwrap();
    println!("{}: {:?}", name, data);
}
