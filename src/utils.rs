pub fn convert_to_u32(bytes: [u8; 4]) -> u32 {
    let mut v: u32 = 0;
    for (i, b) in bytes.iter().enumerate() {
        v += (*b as u32) << (i * 8);
    }
    v
}

pub fn convert_to_u8(mut v: u32) -> [u8; 4] {
    let mut bytes = [0; 4];
    for i in 0..4 {
        bytes[i] = (v & 0xff) as u8;
        v >>= 8;
    }
    bytes
}

pub fn concat(b1: [u8; 4], b2: [u8; 4]) -> [u8; 8] {
    let mut res = [0; 8];
    let mut i = 0;
    for b in b1.iter().chain(b2.iter()) {
        res[i] = *b;
        i += 1;
    }
    res
}
