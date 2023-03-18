fn main() {
    println!("Hello, world!");
}

const PADDING: [u8; 64] = [
    0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

fn pad_message(msg: &[u8]) -> Vec<u8> {
    let mut padded_msg = msg.to_vec();

    // Bits padding
    let id = msg.len() % 64;
    if id < 56 {
        padded_msg.extend_from_slice(&PADDING[0..(56 - id)]);
    } else {
        padded_msg.extend_from_slice(&PADDING[0..(120 - id)]);
    }

    // Length padding
    padded_msg.extend_from_slice(&(msg.len() * 8).to_le_bytes());

    padded_msg
}

// nonlinear functions at bit level: exor, mux, -, mux, -
fn f(j: usize, x: u32, y: u32, z: u32) -> u32 {
    match j {
        0..=15 => x ^ y ^ z,
        16..=31 => (x & y) | (!x & z),
        32..=47 => (x | !y) ^ z,
        48..=63 => (x & z) | (y & !z),
        64..=79 => x ^ (y | !z),
        _ => unreachable!(),
    }
}

// Added constants
fn K(j: usize) -> u32 {
    match j {
        0..=15 => 0x00000000,
        16..=31 => 0x5A827999,
        32..=47 => 0x6ED9EBA1,
        48..=63 => 0x8F1BBCDC,
        64..=79 => 0xA953FD4E,
        _ => unreachable!(),
    }
}

fn K_p(j: usize) -> u32 {
    match j {
        0..=15 => 0x50A28BE6,
        16..=31 => 0x5C4DD124,
        32..=47 => 0x6D703EF3,
        48..=63 => 0x7A6D76E9,
        64..=79 => 0x00000000,
        _ => unreachable!(),
    }
}

// Selection of message word
fn r(j: usize) -> usize {
    let r = match j {
        0..=15 => [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        16..=31 => [7, 4, 13, 1, 10, 6, 15, 3, 12, 0, 9, 5, 2, 14, 11, 8],
        32..=47 => [3, 10, 14, 4, 9, 15, 8, 1, 2, 7, 0, 6, 13, 11, 5, 12],
        48..=63 => [1, 9, 11, 10, 0, 8, 12, 4, 13, 3, 7, 15, 14, 5, 6, 2],
        64..=79 => [4, 0, 5, 9, 7, 12, 2, 10, 14, 1, 3, 8, 11, 6, 15, 13],
        _ => unreachable!(),
    };

    r[j % 16]
}

fn r_p(j: usize) -> usize {
    let r_p = match j {
        0..=15 => [5, 14, 7, 0, 9, 2, 11, 4, 13, 6, 15, 8, 1, 10, 3, 12],
        16..=31 => [6, 11, 3, 7, 0, 13, 5, 10, 14, 15, 8, 12, 4, 9, 1, 2],
        32..=47 => [15, 5, 1, 3, 7, 14, 6, 9, 11, 8, 12, 2, 10, 0, 4, 13],
        48..=63 => [8, 6, 4, 1, 3, 11, 15, 0, 5, 12, 2, 13, 9, 7, 10, 14],
        64..=79 => [12, 15, 10, 4, 1, 5, 8, 7, 6, 2, 13, 14, 0, 3, 9, 11],
        _ => unreachable!(),
    };

    r_p[j % 16]
}

// amount for rotate left (rol)
fn s(j: usize) -> usize {
    let ss = match j {
        0..=15 => [11, 14, 15, 12, 5, 8, 7, 9, 11, 13, 14, 15, 6, 7, 9, 8],
        16..=31 => [7, 6, 8, 13, 11, 9, 7, 15, 7, 12, 15, 9, 11, 7, 13, 12],
        32..=47 => [11, 13, 6, 7, 14, 9, 13, 15, 14, 8, 13, 6, 5, 12, 7, 5],
        48..=63 => [11, 12, 14, 15, 14, 15, 9, 8, 9, 14, 5, 6, 8, 6, 5, 12],
        64..=79 => [9, 15, 5, 11, 6, 8, 13, 12, 5, 12, 13, 14, 11, 8, 5, 6],
        _ => unreachable!(),
    };

    ss[j % 16]
}

fn s_p(j: usize) -> usize {
    let s_p = match j {
        0..=15 => [8, 9, 9, 11, 13, 15, 15, 5, 7, 7, 8, 11, 14, 14, 12, 6],
        16..=31 => [9, 13, 15, 7, 12, 8, 9, 11, 7, 7, 12, 7, 6, 15, 13, 11],
        32..=47 => [9, 7, 15, 11, 8, 6, 6, 14, 12, 13, 5, 14, 13, 13, 7, 5],
        48..=63 => [15, 5, 8, 11, 14, 14, 6, 14, 6, 9, 12, 9, 12, 5, 15, 8],
        64..=79 => [8, 5, 12, 9, 12, 5, 14, 6, 8, 13, 6, 5, 15, 13, 11, 11],
        _ => unreachable!(),
    };

    s_p[j % 16]
}

// initial value
fn h() -> [u32; 5] {
    [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0]
}

fn ripemd_160(msg: String) -> [u8; 20] {
    let padded_msg = pad_message(msg.as_bytes());
    assert!((padded_msg.len() * 8) % 512 == 0, "Invalid padding!");

    let [mut h0, mut h1, mut h2, mut h3, mut h4] = h();

    let x: Vec<u32> = padded_msg
        .chunks(4)
        .map(|x| u32::from_le_bytes(x.try_into().unwrap()))
        .collect();

    let n = x.len() / 16;

    for i in 0..n {
        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        let mut a_p = h0;
        let mut b_p = h1;
        let mut c_p = h2;
        let mut d_p = h3;
        let mut e_p = h4;

        for j in 0..80 {
            let t = (a
                .wrapping_add(f(j, b, c, d))
                .wrapping_add(x[i * 16 + r(j)])
                .wrapping_add(K(j)))
            .rotate_left(s(j).try_into().unwrap())
            .wrapping_add(e);
            a = e;
            e = d;
            d = c.rotate_left(10);
            c = b;
            b = t;

            let t = a_p
                .wrapping_add(f(79 - j, b_p, c_p, d_p))
                .wrapping_add(x[i * 16 + r_p(j)])
                .wrapping_add(K_p(j))
                .rotate_left(s_p(j).try_into().unwrap())
                .wrapping_add(e_p);
            a_p = e_p;
            e_p = d_p;
            d_p = c_p.rotate_left(10);
            c_p = b_p;
            b_p = t;
        }

        let t = h1.wrapping_add(c).wrapping_add(d_p);
        h1 = h2.wrapping_add(d).wrapping_add(e_p);
        h2 = h3.wrapping_add(e).wrapping_add(a_p);
        h3 = h4.wrapping_add(a).wrapping_add(b_p);
        h4 = h0.wrapping_add(b).wrapping_add(c_p);
        h0 = t;
    }

    let mut res: [u8; 20] = [0_u8; 20];
    for (chunk, v) in res.chunks_exact_mut(4).zip([h0, h1, h2, h3, h4]) {
        chunk.copy_from_slice(&v.to_le_bytes());
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_msg_padding() {
        let msg = "The saddest aspect of life right now is that science gathers knowledge faster than society gathers wisdom.".to_string();
        let padded_msg = pad_message(msg.as_bytes());
        assert_eq!(
            &padded_msg,
            &[
                84, 104, 101, 32, 115, 97, 100, 100, 101, 115, 116, 32, 97, 115, 112, 101, 99, 116,
                32, 111, 102, 32, 108, 105, 102, 101, 32, 114, 105, 103, 104, 116, 32, 110, 111,
                119, 32, 105, 115, 32, 116, 104, 97, 116, 32, 115, 99, 105, 101, 110, 99, 101, 32,
                103, 97, 116, 104, 101, 114, 115, 32, 107, 110, 111, 119, 108, 101, 100, 103, 101,
                32, 102, 97, 115, 116, 101, 114, 32, 116, 104, 97, 110, 32, 115, 111, 99, 105, 101,
                116, 121, 32, 103, 97, 116, 104, 101, 114, 115, 32, 119, 105, 115, 100, 111, 109,
                46, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 80, 3, 0, 0, 0, 0, 0, 0,
            ]
        );
    }

    #[test]
    fn test_ripemd_160() {
        let msg = "a".to_string();
        let hash = ripemd_160(msg);
        // assert!(hash == "0x0bdc9d2d256b3ee9daae347be6f4dc835a467ffe")
    }
}
