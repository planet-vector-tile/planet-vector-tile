mod tile;

fn main() {
    let t = tile::Tile::from_zxy(6, 5, 454);
    println!("max 59 bytes {} , tile {} !", 2u64.pow(59), t.h);
}
