use std::simd::{prelude::*, LaneCount, SupportedLaneCount};

/// Unpacks 4-bit values from a vec of bytes into a vec of 16-bit values.
/// Returns an empty vec if the input length is not a multiple of 8 or is 0.
pub(crate) fn unpack4(src: &[u8], big_endian: bool) -> Vec<u16> {
    if src.len() % 8 != 0 || src.is_empty() {
        return Vec::new();
    }

    let mut dst: Vec<u16> = vec![0; src.len() * 2];
    let mut offset: usize = 0;

    offset = simd_unpack4::<64>(src, big_endian, &mut dst, offset);
    offset = simd_unpack4::<32>(src, big_endian, &mut dst, offset);
    offset = simd_unpack4::<16>(src, big_endian, &mut dst, offset);
    simd_unpack4::<8>(src, big_endian, &mut dst, offset);

    dst
}

/// Unpacks 5-bit values from a vec of bytes into a vec of 16-bit values.
/// Returns an empty vec if the input length is not a multiple of 8 or is 0.
pub(crate) fn unpack5(src: &[u8], big_endian: bool) -> Vec<u16> {
    if src.len() % 8 != 0 || src.is_empty() {
        return Vec::new();
    }

    let mut dst: Vec<u16> = vec![0; src.len() / 8 * 12];
    let mut offset: usize = 0;

    offset = simd_unpack5::<64>(src, big_endian, &mut dst, offset);
    offset = simd_unpack5::<32>(src, big_endian, &mut dst, offset);
    offset = simd_unpack5::<16>(src, big_endian, &mut dst, offset);
    simd_unpack5::<8>(src, big_endian, &mut dst, offset);

    dst
}

/// Swaps the endianess of 32-bit values in a vec of bytes. Returns an empty vec
/// if the input length is not a multiple of 4 or is 0.
pub(crate) fn swap_endianness_32bit(src: &[u8]) -> Vec<u8> {
    if src.len() % 4 != 0 || src.is_empty() {
        return Vec::new();
    }

    let mut dst: Vec<u8> = vec![0; src.len()];
    let mut offset: usize = 0;

    while offset + 63 < src.len() {
        let simd = Simd::<u8, 64>::from_slice(&src[offset..offset + 64]);
        let swapped = simd_swap_endianness_32bit(simd);
        dst[offset..offset + 64].copy_from_slice(swapped.as_array());

        offset += 64;
    }

    while offset + 31 < src.len() {
        let simd = Simd::<u8, 32>::from_slice(&src[offset..offset + 32]);
        let swapped = simd_swap_endianness_32bit(simd);
        dst[offset..offset + 32].copy_from_slice(swapped.as_array());

        offset += 32;
    }

    while offset + 15 < src.len() {
        let simd = Simd::<u8, 16>::from_slice(&src[offset..offset + 16]);
        let swapped = simd_swap_endianness_32bit(simd);
        dst[offset..offset + 16].copy_from_slice(swapped.as_array());

        offset += 16;
    }

    while offset + 7 < src.len() {
        let simd = Simd::<u8, 8>::from_slice(&src[offset..offset + 8]);
        let swapped = simd_swap_endianness_32bit(simd);
        dst[offset..offset + 8].copy_from_slice(swapped.as_array());

        offset += 8;
    }

    if offset + 3 < src.len() {
        dst[offset..offset + 4].copy_from_slice(&[
            src[offset + 3],
            src[offset + 2],
            src[offset + 1],
            src[offset],
        ]);
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
    let mut offset: usize = 0;

    while offset + 63 < src.len() {
        let simd = Simd::<u8, 64>::from_slice(&src[offset..offset + 64]);
        let swapped = simd_swap_endianness_64bit(simd);
        dst[offset..offset + 64].copy_from_slice(swapped.as_array());

        offset += 64;
    }

    while offset + 31 < src.len() {
        let simd = Simd::<u8, 32>::from_slice(&src[offset..offset + 32]);
        let swapped = simd_swap_endianness_64bit(simd);
        dst[offset..offset + 32].copy_from_slice(swapped.as_array());

        offset += 32;
    }

    while offset + 15 < src.len() {
        let simd = Simd::<u8, 16>::from_slice(&src[offset..offset + 16]);
        let swapped = simd_swap_endianness_64bit(simd);
        dst[offset..offset + 16].copy_from_slice(swapped.as_array());

        offset += 16;
    }

    if offset + 7 < src.len() {
        dst[offset..offset + 8].copy_from_slice(&[
            src[offset + 7],
            src[offset + 6],
            src[offset + 5],
            src[offset + 4],
            src[offset + 3],
            src[offset + 2],
            src[offset + 1],
            src[offset],
        ]);
    }

    dst
}

/// Unpacks 4-bit values packed into longs in `src` with endianness specified by
/// `big_endian` into `dst` using `N` lanes. Starts at `offset` in `src` and
/// returns the new offset.
fn simd_unpack4<const N: usize>(
    src: &[u8],
    big_endian: bool,
    dst: &mut [u16],
    mut offset: usize,
) -> usize
where
    LaneCount<N>: SupportedLaneCount,
{
    while offset + N <= src.len() {
        let mut simd = Simd::<u8, N>::from_slice(&src[offset..offset + N]);
        if big_endian {
            simd = simd_swap_endianness_64bit(simd);
        }

        // separate the upper and lower nibbles
        let mut lower = simd & Simd::splat(0x0f);
        let mut upper = simd >> Simd::splat(4);

        // interleave the nibbles
        (lower, upper) = lower.interleave(upper);

        // convert the 8-bit values to 16-bit values
        let extended_lower = lower.cast::<u16>();
        let extended_upper = upper.cast::<u16>();

        // store the 16-bit values in the destination
        dst[offset * 2..offset * 2 + N].copy_from_slice(extended_lower.as_array());
        dst[offset * 2 + N..offset * 2 + N * 2].copy_from_slice(extended_upper.as_array());

        offset += N;
    }

    offset
}

const PERM_A_PATTERN: [u8; 12] = [0, 1, 0x0, 2, 3, 0x0, 4, 0x0, 5, 6, 0x0, 7];
const SHIFT_A_PATTERN: [u8; 12] = [0, 3, 0, 1, 4, 0, 2, 0, 0, 3, 0, 1];
const AND_A_PATTERN: [u8; 12] = [0x1f, 0x1f, 0, 0x1f, 0x1f, 0, 0x1f, 0, 0x1f, 0x1f, 0, 0x1f];
const PERM_B_PATTERN: [u8; 12] = [0x0, 0, 1, 1, 2, 3, 3, 4, 0x0, 5, 6, 6];
const SHIFT_B_PATTERN: [u8; 12] = [0, 5, 2, 7, 4, 1, 6, 3, 0, 5, 2, 7];
const AND_B_PATTERN: [u8; 12] = [
    0, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0, 0x1f, 0x1f, 0x1f,
];

/// Unpacks 5-bit values packed into longs in `src` with endianness specified by
/// `big_endian` into `dst` using `N` lanes. Starts at `offset` in `src` and
/// returns the new offset.
fn simd_unpack5<const N: usize>(
    src: &[u8],
    big_endian: bool,
    dst: &mut [u16],
    mut offset: usize,
) -> usize
where
    LaneCount<N>: SupportedLaneCount,
    LaneCount<{ N / 2 }>: SupportedLaneCount,
{
    let perm_a_half = tiled::<{ N / 2 }>(&PERM_A_PATTERN, 8);
    let perm_a_full = tiled_with_offset::<N>(&PERM_A_PATTERN, N / 2, 8);

    let shift_a_half = tiled::<{ N / 2 }>(&SHIFT_A_PATTERN, 0);
    let shift_a_full = tiled_with_offset::<N>(&SHIFT_A_PATTERN, N / 2, 0);

    let and_a_half = tiled::<{ N / 2 }>(&AND_A_PATTERN, 0);
    let and_a_full = tiled_with_offset::<N>(&AND_A_PATTERN, N / 2, 0);

    let perm_b_half = tiled::<{ N / 2 }>(&PERM_B_PATTERN, 8);
    let perm_b_full = tiled_with_offset::<N>(&PERM_B_PATTERN, N / 2, 8);

    let shift_b_half = tiled::<{ N / 2 }>(&SHIFT_B_PATTERN, 0);
    let shift_b_full = tiled_with_offset::<N>(&SHIFT_B_PATTERN, N / 2, 0);

    let and_b_half = tiled::<{ N / 2 }>(&AND_B_PATTERN, 0);
    let and_b_full = tiled_with_offset::<N>(&AND_B_PATTERN, N / 2, 0);

    while offset + N <= src.len() {
        let mut simd = Simd::<u8, N>::from_slice(&src[offset..offset + N]);
        if big_endian {
            simd = simd_swap_endianness_64bit(simd);
        }

        let mut a_half = simd.resize(0).swizzle_dyn(perm_a_half);
        let mut a_full = simd.swizzle_dyn(perm_a_full);

        a_half <<= shift_a_half;
        a_full <<= shift_a_full;

        a_half &= and_a_half;
        a_full &= and_a_full;

        let mut b_half = simd.resize(0).swizzle_dyn(perm_b_half);
        let mut b_full = simd.swizzle_dyn(perm_b_full);

        b_half >>= shift_b_half;
        b_full >>= shift_b_full;

        b_half &= and_b_half;
        b_full &= and_b_full;

        let out_half = a_half | b_half;
        let out_full = a_full | b_full;

        let extended_half = out_half.cast::<u16>();
        let extended_full = out_full.cast::<u16>();

        let dst_offset = offset / 8 * 12;
        dst[dst_offset..dst_offset + N / 2].copy_from_slice(extended_half.as_array());
        dst[dst_offset + N / 2..dst_offset + N / 2 + N].copy_from_slice(extended_full.as_array());

        offset += N;
    }

    offset
}

// Taken and modified from https://mcyoung.xyz/2023/11/27/simd-base64/
/// Generates a new vector made up of repeated tiles, adding `increase` to each
/// element every time the tile is repeated.
fn tiled<const N: usize>(tile: &[u8], increase: usize) -> Simd<u8, N>
where
    LaneCount<N>: SupportedLaneCount,
{
    let mut out = [0; N];
    let mut i = 0;
    while i < N {
        out[i] = tile[i % tile.len()] + (i / tile.len() * increase) as u8;
        i += 1;
    }
    Simd::from_array(out)
}

// Taken and modified from https://mcyoung.xyz/2023/11/27/simd-base64/
/// Generates a new vector made up of repeated tiles, adding `increase` to each
/// element every time the tile is repeated. The tiling is modified as though it
/// is picking up from where the previous tiling left off, where `prev` elements
/// were generated.
fn tiled_with_offset<const N: usize>(tile: &[u8], prev: usize, increase: usize) -> Simd<u8, N>
where
    LaneCount<N>: SupportedLaneCount,
{
    let mut out = [0; N];
    let mut i = 0;
    while i < N {
        out[i] = tile[(i + prev) % tile.len()] + ((i + prev) / tile.len() * increase) as u8;
        i += 1;
    }
    Simd::from_array(out)
}

const ENDIAN_SWAP_32BIT: [u8; 4] = [3, 2, 1, 0];
const ENDIAN_SWAP_64BIT: [u8; 8] = [7, 6, 5, 4, 3, 2, 1, 0];

/// Swaps the endianness of 32-bit values in a SIMD vector of bytes.
#[inline]
fn simd_swap_endianness_32bit<const N: usize>(simd: Simd<u8, N>) -> Simd<u8, N>
where
    LaneCount<N>: SupportedLaneCount,
{
    simd.swizzle_dyn(tiled(&ENDIAN_SWAP_32BIT, ENDIAN_SWAP_32BIT.len()))
}

/// Swaps the endianness of 64-bit values in a SIMD vector of bytes.
#[inline]
fn simd_swap_endianness_64bit<const N: usize>(simd: Simd<u8, N>) -> Simd<u8, N>
where
    LaneCount<N>: SupportedLaneCount,
{
    simd.swizzle_dyn(tiled(&ENDIAN_SWAP_64BIT, ENDIAN_SWAP_64BIT.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiled_multiple() {
        let tile_pattern = [0, 1, 2, 3];
        let expected =
            Simd::<u8, 16>::from_array([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        assert_eq!(tiled(&tile_pattern, tile_pattern.len()), expected);
    }

    #[test]
    fn test_tiled_non_multiple() {
        let tile_pattern = [0, 1, 2, 3, 4];
        let expected =
            Simd::<u8, 16>::from_array([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        assert_eq!(tiled(&tile_pattern, tile_pattern.len()), expected);
    }

    #[test]
    fn test_tiled_with_offset_multiple() {
        let tile_pattern = [0, 1, 2, 3];
        let expected =
            Simd::<u8, 16>::from_array([4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]);
        assert_eq!(
            tiled_with_offset(&tile_pattern, 4, tile_pattern.len()),
            expected
        );
    }

    #[test]
    fn test_tiled_with_offset_non_multiple() {
        let tile_pattern = [0, 1, 2, 3, 4];
        let expected =
            Simd::<u8, 16>::from_array([4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]);
        assert_eq!(
            tiled_with_offset(&tile_pattern, 4, tile_pattern.len()),
            expected
        );
    }

    #[test]
    fn test_tiled_with_offset_no_offset_multiple() {
        let tile_pattern = [0, 1, 2, 3];
        let expected =
            Simd::<u8, 16>::from_array([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        assert_eq!(
            tiled_with_offset(&tile_pattern, 0, tile_pattern.len()),
            expected
        );
    }

    #[test]
    fn test_tiled_with_offset_no_offset_non_multiple() {
        let tile_pattern = [0, 1, 2, 3, 4];
        let expected =
            Simd::<u8, 16>::from_array([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        assert_eq!(
            tiled_with_offset(&tile_pattern, 0, tile_pattern.len()),
            expected
        );
    }
}
