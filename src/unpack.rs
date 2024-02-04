use std::simd::{prelude::*, LaneCount, SupportedLaneCount};

/// Marker trait for SIMD lane counts that can hold 8 bytes when the element
/// type is a byte.
pub(crate) trait LongLaneCount: SupportedLaneCount {}
impl LongLaneCount for LaneCount<8> {}
impl LongLaneCount for LaneCount<16> {}
impl LongLaneCount for LaneCount<32> {}
impl LongLaneCount for LaneCount<64> {}

/// Marker trait for SIMD lane counts that can hold 4 bytes when the element
/// type is a byte.
pub(crate) trait IntLaneCount: SupportedLaneCount {}
impl IntLaneCount for LaneCount<4> {}
impl IntLaneCount for LaneCount<8> {}
impl IntLaneCount for LaneCount<16> {}
impl IntLaneCount for LaneCount<32> {}
impl IntLaneCount for LaneCount<64> {}

/// Unpack nybbles from a byte array into a short array. Assumes that the input
/// is little endian longs.
pub(crate) fn unpack4_le<const N: usize>(bytes: &[u8]) -> Vec<u16>
where
    LaneCount<N>: LongLaneCount,
{
    let mut result = Vec::with_capacity(bytes.len() * 2);
    for i in 0..bytes.len() / N {
        let a = Simd::<u8, N>::from_slice(&bytes[i * N..(i + 1) * N]);
        let (extended_a, extended_b) = unpack4(a);
        result.extend_from_slice(&extended_a.as_array()[..]);
        result.extend_from_slice(&extended_b.as_array()[..]);
    }

    result
}

pub(crate) fn unpack4_be<const N: usize>(bytes: &[u8]) -> Vec<u16>
where
    LaneCount<N>: LongLaneCount,
{
    let mut result = Vec::with_capacity(bytes.len() * 2);
    for i in 0..bytes.len() / N {
        let a = Simd::<u8, N>::from_slice(&bytes[i * N..(i + 1) * N])
            .swizzle_dyn(tiled(&[7, 6, 5, 4, 3, 2, 1, 0]));
        let (extended_a, extended_b) = unpack4(a);
        result.extend_from_slice(&extended_a.as_array()[..]);
        result.extend_from_slice(&extended_b.as_array()[..]);
    }

    result
}

/// Unpacks nybbles from a SIMD vector of little endian longs as bytes into two
/// SIMD vectors of shorts.
#[inline]
fn unpack4<const N: usize>(mut a: Simd<u8, N>) -> (Simd<u16, N>, Simd<u16, N>)
where
    LaneCount<N>: LongLaneCount,
{
    let mut b = a >> Simd::splat(4);
    a &= Simd::splat(0x0f);
    (a, b) = a.interleave(b);
    let extended_a = a.cast::<u16>();
    let extended_b = b.cast::<u16>();
    (extended_a, extended_b)
}

/// Unpack quintets from a byte array into a short array. Assumes that the input
/// is little endian longs, and that there is an even number of longs.
pub(crate) fn unpack5_le(bytes: &[u8]) -> Vec<u16> {
    let mut result = Vec::with_capacity(bytes.len() / 8 * 12);

    for i in 0..bytes.len() / 16 {
        let longs: Simd<u8, 16> = Simd::from_slice(&bytes[i * 16..(i + 1) * 16]);
        let extended = unpack5(longs);
        result.extend_from_slice(&extended.as_array()[..24]);
    }

    result
}

pub(crate) fn unpack5_be(bytes: &[u8]) -> Vec<u16> {
    let mut result = Vec::with_capacity(bytes.len() / 8 * 12);

    for i in 0..bytes.len() / 16 {
        let longs: Simd<u8, 16> = Simd::from_slice(&bytes[i * 16..(i + 1) * 16])
            .swizzle_dyn(tiled(&[7, 6, 5, 4, 3, 2, 1, 0]));
        let extended = unpack5(longs);
        result.extend_from_slice(&extended.as_array()[..24]);
    }

    result
}

// For a single little endian long,
// byte 777777777   666666666   555555555   444444444   333333333   222222222   111111111   000000000
// MSB  0000 llll | lkkk kkjj | jjji iiii | hhhh hggg | ggff fffe | eeee dddd | dccc ccbb | bbba aaaa  LSB
// a = long[0] & 0x1f
// b = long[1] << 3 & 0x1f | long[0] >> 5 & 0x1f
// c =                       long[1] >> 2 & 0x1f
// d = long[2] << 1 & 0x1f | long[1] >> 7 & 0x1f
// e = long[3] << 4 & 0x1f | long[2] >> 4 & 0x1f
// f =                       long[3] >> 1 & 0x1f
// g = long[4] << 2 & 0x1f | long[3] >> 6 & 0x1f
// h =                       long[4] >> 3 & 0x1f
// i = long[5] & 0x1f
// j = long[6] << 3 & 0x1f | long[5] >> 5 & 0x1f
// k =                       long[6] >> 2 & 0x1f
// l = long[7] << 1 & 0x1f | long[6] >> 7 & 0x1f
//
// To support two longs at once, the first 12 indices of the swizzle are copied
// and 8 added to them, then an extra 4 0s are added to the end. The shifts and
// ands are the same, just without adding 8.
#[inline]
fn unpack5(longs: Simd<u8, 16>) -> Simd<u16, 32> {
    // 0x0 marks where we want the value to be zero, which gets done through and
    // by zero
    let mut a = simd_swizzle!(
        longs,
        [
            0, 1, 0x0, 2, 3, 0x0, 4, 0x0, 5, 6, 0x0, 7, 8, 9, 0x0, 10, 11, 0x0, 12, 0x0, 13, 14,
            0x0, 15, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0
        ]
    );
    let mut b = simd_swizzle!(
        longs,
        [
            0x0, 0, 1, 1, 2, 3, 3, 4, 0x0, 5, 6, 6, 0x0, 8, 9, 9, 10, 11, 11, 12, 0x0, 13, 14, 14,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0
        ]
    );
    a <<= Simd::from_array([
        0, 3, 0, 1, 4, 0, 2, 0, 0, 3, 0, 1, 0, 3, 0, 1, 4, 0, 2, 0, 0, 3, 0, 1, 0, 0, 0, 0, 0, 0,
        0, 0,
    ]);
    b >>= Simd::from_array([
        0, 5, 2, 7, 4, 1, 6, 3, 0, 5, 2, 7, 0, 5, 2, 7, 4, 1, 6, 3, 0, 5, 2, 7, 0, 0, 0, 0, 0, 0,
        0, 0,
    ]);
    a &= Simd::from_array([
        0x1f, 0x1f, 0, 0x1f, 0x1f, 0, 0x1f, 0, 0x1f, 0x1f, 0, 0x1f, 0x1f, 0x1f, 0, 0x1f, 0x1f, 0,
        0x1f, 0, 0x1f, 0x1f, 0, 0x1f, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    b &= Simd::from_array([
        0, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0, 0x1f, 0x1f, 0x1f, 0, 0x1f, 0x1f, 0x1f,
        0x1f, 0x1f, 0x1f, 0x1f, 0, 0x1f, 0x1f, 0x1f, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    a |= b;
    a.cast::<u16>()
}

pub(crate) fn swap_endianness_32bit<const N: usize>(bytes: &mut [u8])
where
    LaneCount<N>: IntLaneCount,
{
    for i in 0..bytes.len() / N {
        let a = Simd::<u8, N>::from_slice(&bytes[i * N..(i + 1) * N]);
        let b = a.swizzle_dyn(tiled(&[3, 2, 1, 0]));
        bytes[i * N..(i + 1) * N].copy_from_slice(b.as_array());
    }
}

pub(crate) fn swap_endianness_64bit<const N: usize>(bytes: &mut [u8])
where
    LaneCount<N>: LongLaneCount,
{
    for i in 0..bytes.len() / N {
        let a = Simd::<u8, N>::from_slice(&bytes[i * N..(i + 1) * N]);
        let b = a.swizzle_dyn(tiled(&[7, 6, 5, 4, 3, 2, 1, 0]));
        bytes[i * N..(i + 1) * N].copy_from_slice(b.as_array());
    }
}

// Taken and modified from https://mcyoung.xyz/2023/11/27/simd-base64/
/// Generates a new vector made up of repeated "tiles" of identical
/// data.
const fn tiled<const N: usize>(tile: &[u8]) -> Simd<u8, N>
where
    LaneCount<N>: SupportedLaneCount,
{
    let mut out = [tile[0]; N];
    let mut i = 0;
    while i < N {
        out[i] = tile[i % tile.len()] + (i / tile.len() * tile.len()) as u8;
        i += 1;
    }
    Simd::from_array(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiled_32() {
        let expected =
            Simd::<u8, 16>::from_array([3, 2, 1, 0, 7, 6, 5, 4, 11, 10, 9, 8, 15, 14, 13, 12]);
        assert_eq!(tiled(&[3, 2, 1, 0]), expected);
    }

    #[test]
    fn test_tiled_64() {
        let expected =
            Simd::<u8, 16>::from_array([7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8]);
        assert_eq!(tiled(&[7, 6, 5, 4, 3, 2, 1, 0]), expected);
    }

    #[test]
    fn test_swap_endianness_64bit() {
        let mut input = [
            0x0123456789abcdef_u64.to_be_bytes(),
            0xfedcba9876543210_u64.to_be_bytes(),
        ]
        .concat();
        let expected = [
            0x0123456789abcdef_u64.to_le_bytes(),
            0xfedcba9876543210_u64.to_le_bytes(),
        ]
        .concat();
        swap_endianness_64bit::<16>(&mut input);
        assert_eq!(input, expected);
    }

    #[test]
    fn test_unpack4_le() {
        // nybbles 0 through 15 packed into a little endian long, then repeated
        let input: [u8; 16] = [
            0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe, 0x10, 0x32, 0x54, 0x76, 0x98, 0xba,
            0xdc, 0xfe,
        ];
        let expected: [u16; 32] = [
            0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf, 0x0,
            0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf,
        ];
        assert_eq!(unpack4_le::<8>(&input), expected);
        assert_eq!(unpack4_le::<16>(&input), expected);
    }

    #[test]
    fn test_unpack4_be() {
        // nybbles 0 through 15 packed into a big endian long, then repeated
        let input: [u8; 16] = [
            0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54,
            0x32, 0x10,
        ];
        let expected: [u16; 32] = [
            0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf, 0x0,
            0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf,
        ];
        assert_eq!(unpack4_be::<8>(&input), expected);
        assert_eq!(unpack4_be::<16>(&input), expected);
    }

    #[test]
    fn test_unpack5_le() {
        // quintets 0 through 23 packed into two little endian longs
        let input = [
            0b00000101_10101001_00101000_00111001_10001010_01000001_10001000_00100000_u64
                .to_le_bytes(),
            0b00001011_11011010_10110100_10011100_10100011_00000111_10111001_10101100_u64
                .to_le_bytes(),
        ]
        .concat();
        let expected: [u16; 24] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
        ];
        assert_eq!(unpack5_le(&input), expected);
    }

    #[test]
    fn test_unpack5_be() {
        // quintets 0 through 23 packed into two big endian longs
        let input = [
            0b00000101_10101001_00101000_00111001_10001010_01000001_10001000_00100000_u64
                .to_be_bytes(),
            0b00001011_11011010_10110100_10011100_10100011_00000111_10111001_10101100_u64
                .to_be_bytes(),
        ]
        .concat();
        let expected: [u16; 24] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
        ];
        assert_eq!(unpack5_be(&input), expected);
    }
}
