
pub fn dm7_to_decimal(dm7: (u32, u32)) -> (f64, f64) {
    let lon = ((dm7.0 as f64) - (i32::MAX as f64)) / 10_000_000f64;
    let lat = ((dm7.1 as f64) - (i32::MAX as f64)) / 10_000_000f64;
    (lon, lat)
}