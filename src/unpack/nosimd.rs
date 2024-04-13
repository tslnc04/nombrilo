/// Unpacks 4-bit values from a vec of bytes into a vec of 16-bit values.
/// Returns an empty vec if the input length is not a multiple of 8 or is 0.
pub(crate) fn unpack4(src: &[u8], big_endian: bool) -> Vec<u16> {
    if src.len() % 8 != 0 || src.is_empty() {
        return Vec::new();
    }

    let mut dst: Vec<u16> = vec![0; src.len() * 2];

    for i in (0..src.len()).step_by(8) {
        let long = if big_endian {
            u64::from_be_bytes([
                src[i],
                src[i + 1],
                src[i + 2],
                src[i + 3],
                src[i + 4],
                src[i + 5],
                src[i + 6],
                src[i + 7],
            ])
        } else {
            u64::from_le_bytes([
                src[i],
                src[i + 1],
                src[i + 2],
                src[i + 3],
                src[i + 4],
                src[i + 5],
                src[i + 6],
                src[i + 7],
            ])
        };
        for j in 0..16 {
            dst[i * 2 + j] = ((long >> (j * 4)) & 0x0f) as u16;
        }
    }

    dst
}

/// Unpacks 5-bit values from a vec of bytes into a vec of 16-bit values.
/// Returns an empty vec if the input length is not a multiple of 8 or is 0.
pub(crate) fn unpack5(src: &[u8], big_endian: bool) -> Vec<u16> {
    if src.len() % 8 != 0 || src.is_empty() {
        return Vec::new();
    }

    let mut dst: Vec<u16> = vec![0; src.len() / 8 * 12];

    for i in (0..src.len()).step_by(8) {
        let long = if big_endian {
            u64::from_be_bytes([
                src[i],
                src[i + 1],
                src[i + 2],
                src[i + 3],
                src[i + 4],
                src[i + 5],
                src[i + 6],
                src[i + 7],
            ])
        } else {
            u64::from_le_bytes([
                src[i],
                src[i + 1],
                src[i + 2],
                src[i + 3],
                src[i + 4],
                src[i + 5],
                src[i + 6],
                src[i + 7],
            ])
        };
        for j in 0..12 {
            dst[i / 8 * 12 + j] = ((long >> (j * 5)) & 0x1f) as u16;
        }
    }

    dst
}

/// Swaps the endianess of 32-bit values in a vec of bytes. Returns an empty vec
/// if the input length is not a multiple of 4 or is 0.
pub(crate) fn swap_endianness_32bit(src: &[u8]) -> Vec<u8> {
    if src.len() % 4 != 0 || src.is_empty() {
        return Vec::new();
    }

    let mut dst: Vec<u8> = vec![0; src.len()];

    for i in (0..src.len()).step_by(4) {
        dst[i..i + 4].copy_from_slice(&[src[i + 3], src[i + 2], src[i + 1], src[i]]);
    }

    dst
}

/// Swaps the endianess of 64-bit values in a vec of bytes. Returns an empty vec
/// if the input length is not a multiple of 8 or is 0.
pub(crate) fn swap_endianness_64bit(src: &[u8]) -> Vec<u8> {
    if src.len() % 8 != 0 || src.is_empty() {
        return Vec::new();
    }

    let mut dst: Vec<u8> = vec![0; src.len()];

    for i in (0..src.len()).step_by(8) {
        dst[i..i + 8].copy_from_slice(&[
            src[i + 7],
            src[i + 6],
            src[i + 5],
            src[i + 4],
            src[i + 3],
            src[i + 2],
            src[i + 1],
            src[i],
        ]);
    }

    dst
}
