fn main() {
    let a: i32 = 1;
    unsafe {
        let p = &a as *const i32 as *const u8;
        let slice_raw = std::slice::from_raw_parts(p, 4);
        if slice_raw[0] == 1 {
            println!("little endian.");
        } else {
            println!("big endian.");
        }
    }
}
