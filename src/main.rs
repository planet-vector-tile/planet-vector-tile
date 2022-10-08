mod tile;

fn main() {
    let t = tile::Tile::from_zxy(6, 5, 454);
    println!("Hello, tile {} !", t.h);
}
