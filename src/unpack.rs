use std::simd::{prelude::*, LaneCount, SupportedLaneCount};

/// Unpack nybbles from a byte array into a short array. Assumes that the input
/// is little endian longs.
pub fn unpack4<const N: usize>(bytes: &[u8]) -> Vec<u16>
where
    LaneCount<N>: SupportedLaneCount,
{
    let mut result = Vec::with_capacity(bytes.len() * 2);
    for i in 0..bytes.len() / N {
        let mut a = Simd::<u8, N>::from_slice(&bytes[i * N..(i + 1) * N]);
        let mut b = a >> Simd::splat(4);
        a &= Simd::splat(0x0f);
        (a, b) = a.interleave(b);
        let extended_a = a.cast::<u16>();
        let extended_b = b.cast::<u16>();
        result.extend_from_slice(&extended_a.as_array()[..]);
        result.extend_from_slice(&extended_b.as_array()[..]);
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

/// Unpack quintets from a byte array into a short array. Assumes that the input
/// is little endian longs, and that there is an even number of longs.
pub fn unpack5(bytes: &[u8]) -> Vec<u16> {
    let mut result = Vec::with_capacity(bytes.len() / 8 * 12);

    for i in 0..bytes.len() / 16 {
        let longs: Simd<u8, 16> = Simd::from_slice(&bytes[i * 16..(i + 1) * 16]);
        // 0x0 marks where we want the value to be zero, which gets done through
        // and by zero. on x86 this could be done with 0x80 and vpshufb but rust
        // doesn't support that. it only costs 1 extra and operation
        let mut a = simd_swizzle!(
            longs,
            [
                0, 1, 0x0, 2, 3, 0x0, 4, 0x0, 5, 6, 0x0, 7, 8, 9, 0x0, 10, 11, 0x0, 12, 0x0, 13,
                14, 0x0, 15, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0
            ]
        );
        let mut b = simd_swizzle!(
            longs,
            [
                0x0, 0, 1, 1, 2, 3, 3, 4, 0x0, 5, 6, 6, 0x0, 8, 9, 9, 10, 11, 11, 12, 0x0, 13, 14,
                14, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0
            ]
        );
        a <<= Simd::from_array([
            0, 3, 0, 1, 4, 0, 2, 0, 0, 3, 0, 1, 0, 3, 0, 1, 4, 0, 2, 0, 0, 3, 0, 1, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        b >>= Simd::from_array([
            0, 5, 2, 7, 4, 1, 6, 3, 0, 5, 2, 7, 0, 5, 2, 7, 4, 1, 6, 3, 0, 5, 2, 7, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        a &= Simd::from_array([
            0x1f, 0x1f, 0, 0x1f, 0x1f, 0, 0x1f, 0, 0x1f, 0x1f, 0, 0x1f, 0x1f, 0x1f, 0, 0x1f, 0x1f,
            0, 0x1f, 0, 0x1f, 0x1f, 0, 0x1f, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        b &= Simd::from_array([
            0, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0, 0x1f, 0x1f, 0x1f, 0, 0x1f, 0x1f, 0x1f,
            0x1f, 0x1f, 0x1f, 0x1f, 0, 0x1f, 0x1f, 0x1f, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        a |= b;
        let extended_a = a.cast::<u16>();
        result.extend_from_slice(&extended_a.as_array()[..24]);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unpack4() {
        // nybbles 0 through 15 packed into a little endian long, then repeated
        let input: [u8; 16] = [
            0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe, 0x10, 0x32, 0x54, 0x76, 0x98, 0xba,
            0xdc, 0xfe,
        ];
        let expected: [u16; 32] = [
            0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf, 0x0,
            0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf,
        ];
        assert_eq!(unpack4::<2>(&input), expected);
        assert_eq!(unpack4::<4>(&input), expected);
        assert_eq!(unpack4::<8>(&input), expected);
        assert_eq!(unpack4::<16>(&input), expected);
    }

    #[test]
    fn test_unpack5() {
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
        assert_eq!(unpack5(&input), expected);
    }
}
