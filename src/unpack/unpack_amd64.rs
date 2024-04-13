use std::{arch::x86_64::*, ptr};

/// Unpacks 4-bit values from a vec of bytes into a vec of 16-bit values.
/// Returns an empty vec if the input length is not a multiple of 8 or is 0.
pub(crate) fn unpack4(src: &[u8], big_endian: bool) -> Vec<u16> {
    if src.len() % 8 != 0 || src.is_empty() {
        return Vec::new();
    }

    let mut dst = vec![0; src.len() * 2];
    let mut offset: usize = 0;

    #[cfg(all(feature = "nightly", target_feature = "avx512bw"))]
    unsafe {
        let endian_swap = _mm512_set4_epi64(
            0x08090a0b0c0d0e0f,
            0x0001020304050607,
            0x08090a0b0c0d0e0f,
            0x0001020304050607,
        );
        let lower_nibble_mask = _mm512_set1_epi32(0x0f0f0f0f);
        let permute_mask_lower = _mm512_set_epi64(
            0b1011, 0b1010, 0b0011, 0b0010, 0b1001, 0b1000, 0b0001, 0b0000,
        );
        let permute_mask_upper = _mm512_set_epi64(
            0b1111, 0b1110, 0b0111, 0b0110, 0b1101, 0b1100, 0b0101, 0b0100,
        );

        while offset + 64 <= src.len() {
            let mut a = _mm512_loadu_epi8(ptr::addr_of!(src[offset]) as *const i8);
            if big_endian {
                a = _mm512_shuffle_epi8(a, endian_swap);
            }

            // separate the upper and lower nibbles
            let mut a_upper = _mm512_srli_epi16(a, 4);
            a_upper = _mm512_and_epi32(a_upper, lower_nibble_mask);
            let a_lower = _mm512_and_epi32(a, lower_nibble_mask);

            // interleave the nibbles within 128-bit lanes
            let a_interleaved_lower = _mm512_unpacklo_epi8(a_lower, a_upper);
            let a_interleaved_upper = _mm512_unpackhi_epi8(a_lower, a_upper);

            // interleave the 128-bit lanes to get the final result
            let out_lower = _mm512_permutex2var_epi64(
                a_interleaved_lower,
                permute_mask_lower,
                a_interleaved_upper,
            );
            let out_upper = _mm512_permutex2var_epi64(
                a_interleaved_lower,
                permute_mask_upper,
                a_interleaved_upper,
            );

            // convert the 8-bit values to 16-bit values 32 at a time
            let out_lower_lower = _mm512_extracti64x4_epi64(out_lower, 0);
            let out_lower_lower_extended = _mm512_cvtepu8_epi16(out_lower_lower);

            let out_lower_upper = _mm512_extracti64x4_epi64(out_lower, 1);
            let out_lower_upper_extended = _mm512_cvtepu8_epi16(out_lower_upper);

            let out_upper_lower = _mm512_extracti64x4_epi64(out_upper, 0);
            let out_upper_lower_extended = _mm512_cvtepu8_epi16(out_upper_lower);

            let out_upper_upper = _mm512_extracti64x4_epi64(out_upper, 1);
            let out_upper_upper_extended = _mm512_cvtepu8_epi16(out_upper_upper);

            // store the 16-bit values in the destination
            _mm512_storeu_epi16(
                ptr::addr_of_mut!(dst[offset * 2]) as *mut i16,
                out_lower_lower_extended,
            );
            _mm512_storeu_epi16(
                ptr::addr_of_mut!(dst[offset * 2 + 32]) as *mut i16,
                out_lower_upper_extended,
            );
            _mm512_storeu_epi16(
                ptr::addr_of_mut!(dst[offset * 2 + 64]) as *mut i16,
                out_upper_lower_extended,
            );
            _mm512_storeu_epi16(
                ptr::addr_of_mut!(dst[offset * 2 + 96]) as *mut i16,
                out_upper_upper_extended,
            );

            offset += 64;
        }
    }

    #[cfg(target_feature = "avx2")]
    unsafe {
        let endian_swap = _mm256_set_epi64x(
            0x08090a0b0c0d0e0f,
            0x0001020304050607,
            0x08090a0b0c0d0e0f,
            0x0001020304050607,
        );
        let lower_nibble_mask = _mm256_set1_epi32(0x0f0f0f0f);

        while offset + 32 <= src.len() {
            let mut a = _mm256_loadu_si256(ptr::addr_of!(src[offset]) as *const __m256i);
            if big_endian {
                a = _mm256_shuffle_epi8(a, endian_swap);
            }

            // separate the upper and lower nibbles
            let mut a_upper = _mm256_srli_epi16(a, 4);
            a_upper = _mm256_and_si256(a_upper, lower_nibble_mask);
            let a_lower = _mm256_and_si256(a, lower_nibble_mask);

            // interleave the nibbles within 128-bit lanes
            let a_interleaved_lower = _mm256_unpacklo_epi8(a_lower, a_upper);
            let a_interleaved_upper = _mm256_unpackhi_epi8(a_lower, a_upper);

            // interleave the 128-bit lanes to get the final result
            let out_lower =
                _mm256_permute2x128_si256(a_interleaved_lower, a_interleaved_upper, 0b00100000);
            let out_upper =
                _mm256_permute2x128_si256(a_interleaved_lower, a_interleaved_upper, 0b00110001);

            // convert the 8-bit values to 16-bit values 16 at a time
            let out_lower_lower = _mm256_extracti128_si256(out_lower, 0);
            let out_lower_lower_extended = _mm256_cvtepu8_epi16(out_lower_lower);

            let out_lower_upper = _mm256_extracti128_si256(out_lower, 1);
            let out_lower_upper_extended = _mm256_cvtepu8_epi16(out_lower_upper);

            let out_upper_lower = _mm256_extracti128_si256(out_upper, 0);
            let out_upper_lower_extended = _mm256_cvtepu8_epi16(out_upper_lower);

            let out_upper_upper = _mm256_extracti128_si256(out_upper, 1);
            let out_upper_upper_extended = _mm256_cvtepu8_epi16(out_upper_upper);

            // store the 16-bit values in the destination
            _mm256_storeu_si256(
                ptr::addr_of_mut!(dst[offset * 2]) as *mut __m256i,
                out_lower_lower_extended,
            );
            _mm256_storeu_si256(
                ptr::addr_of_mut!(dst[offset * 2 + 16]) as *mut __m256i,
                out_lower_upper_extended,
            );
            _mm256_storeu_si256(
                ptr::addr_of_mut!(dst[offset * 2 + 32]) as *mut __m256i,
                out_upper_lower_extended,
            );
            _mm256_storeu_si256(
                ptr::addr_of_mut!(dst[offset * 2 + 48]) as *mut __m256i,
                out_upper_upper_extended,
            );

            offset += 32;
        }
    }

    #[cfg(target_feature = "sse4.2")]
    unsafe {
        let endian_swap = _mm_set_epi64x(0x08090a0b0c0d0e0f, 0x0001020304050607);
        let lower_nibble_mask = _mm_set1_epi32(0x0f0f0f0f);

        while offset + 16 <= src.len() {
            let mut a = _mm_loadu_si128(ptr::addr_of!(src[offset]) as *const __m128i);
            if big_endian {
                a = _mm_shuffle_epi8(a, endian_swap);
            }

            // separate the upper and lower nibbles
            let mut a_upper = _mm_srli_epi16(a, 4);
            a_upper = _mm_and_si128(a_upper, lower_nibble_mask);
            let a_lower = _mm_and_si128(a, lower_nibble_mask);

            // interleave the nibbles
            let mut out_lower = _mm_unpacklo_epi8(a_lower, a_upper);
            let mut out_upper = _mm_unpackhi_epi8(a_lower, a_upper);

            // convert the 8-bit values to 16-bit values 8 at a time
            let out_lower_lower_extended = _mm_cvtepu8_epi16(out_lower);
            out_lower = _mm_srli_si128(out_lower, 8);
            let out_lower_upper_extended = _mm_cvtepu8_epi16(out_lower);

            let out_upper_lower_extended = _mm_cvtepu8_epi16(out_upper);
            out_upper = _mm_srli_si128(out_upper, 8);
            let out_upper_upper_extended = _mm_cvtepu8_epi16(out_upper);

            // store the 16-bit values in the destination
            _mm_storeu_si128(
                ptr::addr_of_mut!(dst[offset * 2]) as *mut __m128i,
                out_lower_lower_extended,
            );
            _mm_storeu_si128(
                ptr::addr_of_mut!(dst[offset * 2 + 8]) as *mut __m128i,
                out_lower_upper_extended,
            );
            _mm_storeu_si128(
                ptr::addr_of_mut!(dst[offset * 2 + 16]) as *mut __m128i,
                out_upper_lower_extended,
            );
            _mm_storeu_si128(
                ptr::addr_of_mut!(dst[offset * 2 + 24]) as *mut __m128i,
                out_upper_upper_extended,
            );

            offset += 16;
        }
    }

    while offset + 8 <= src.len() {
        for i in 0..8 {
            let endian_offset = if big_endian {
                offset + (7 - i)
            } else {
                offset + i
            };
            dst[(offset + i) * 2] = (src[endian_offset] & 0x0f) as u16;
            dst[(offset + i) * 2 + 1] = ((src[endian_offset] & 0xf0) >> 4) as u16;
        }

        offset += 8;
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
    let mut offset: usize = 0;

    #[cfg(all(feature = "nightly", target_feature = "avx512bw"))]
    unsafe {
        let endian_swap = _mm512_set4_epi64(
            0x08090a0b0c0d0e0f,
            0x0001020304050607,
            0x08090a0b0c0d0e0f,
            0x0001020304050607,
        );
        let perm_a_32 = _mm256_set_epi64x(
            0x0014001312001110,
            0x0f000e0d000c000b,
            0x0a00090807000605,
            0x0004000302000100,
        );
        let perm_a_64 = _mm512_set_epi64(
            0x3f003e3d003c003b,
            0x3a00393837003635,
            0x0034003332003130,
            0x2f002e2d002c002b,
            0x2a00292827002625,
            0x0024002322002120,
            0x1f001e1d001c001b,
            0x1a00191817001615,
        );
        let shift_a_0 = _mm512_set_epi64(
            0x0000000200000004,
            0x0001000000030000,
            0x0001000000030000,
            0x0000000200000004,
            0x0001000000030000,
            0x0001000000030000,
            0x0000000200000004,
            0x0001000000030000,
        );
        let shift_a_1 = _mm512_set_epi64(
            0x0001000000030000,
            0x0001000000030000,
            0x0000000200000004,
            0x0001000000030000,
            0x0001000000030000,
            0x0000000200000004,
            0x0001000000030000,
            0x0001000000030000,
        );
        let shift_a_2 = _mm512_set_epi64(
            0x0001000000030000,
            0x0000000200000004,
            0x0001000000030000,
            0x0001000000030000,
            0x0000000200000004,
            0x0001000000030000,
            0x0001000000030000,
            0x0000000200000004,
        );
        let perm_b_32 = _mm256_set_epi64x(
            0x1413131211111000,
            0x0e0e0d000c0b0b0a,
            0x0909080006060500,
            0x0403030201010000,
        );
        let perm_b_64 = _mm512_set_epi64(
            0x3e3e3d003c3b3b3a,
            0x3939380036363500,
            0x3433333231313000,
            0x2e2e2d002c2b2b2a,
            0x2929280026262500,
            0x2423232221212000,
            0x1e1e1d001c1b1b1a,
            0x1919180016161500,
        );
        let shift_b_0 = _mm512_set_epi64(
            0x0003000600010004,
            0x0007000200050000,
            0x0007000200050000,
            0x0003000600010004,
            0x0007000200050000,
            0x0007000200050000,
            0x0003000600010004,
            0x0007000200050000,
        );
        let shift_b_1 = _mm512_set_epi64(
            0x0007000200050000,
            0x0007000200050000,
            0x0003000600010004,
            0x0007000200050000,
            0x0007000200050000,
            0x0003000600010004,
            0x0007000200050000,
            0x0007000200050000,
        );
        let shift_b_2 = _mm512_set_epi64(
            0x0007000200050000,
            0x0003000600010004,
            0x0007000200050000,
            0x0007000200050000,
            0x0003000600010004,
            0x0007000200050000,
            0x0007000200050000,
            0x0003000600010004,
        );
        let and_pattern = _mm512_set1_epi64(0x001f001f001f001f);

        while offset + 64 <= src.len() {
            let mut longs = _mm512_loadu_epi8(ptr::addr_of!(src[offset]) as *const i8);
            if big_endian {
                longs = _mm512_shuffle_epi8(longs, endian_swap);
            }

            // position the first set of inputs for each of the output values
            let a_32 =
                _mm256_maskz_permutexvar_epi8(0x5bb5bb5b, perm_a_32, _mm512_castsi512_si256(longs));
            let a_64 = _mm512_maskz_permutexvar_epi8(0xb5bb5bb5bb5bb5bb, perm_a_64, longs);

            // convert the 8-bit values to 16-bit values
            let mut a_0 = _mm512_cvtepu8_epi16(a_32);
            let mut a_1 = _mm512_cvtepu8_epi16(_mm512_castsi512_si256(a_64));
            let a_64 = _mm512_extracti64x4_epi64(a_64, 1);
            let mut a_2 = _mm512_cvtepu8_epi16(a_64);

            // shift the values left by the shift pattern
            a_0 = _mm512_sllv_epi16(a_0, shift_a_0);
            a_1 = _mm512_sllv_epi16(a_1, shift_a_1);
            a_2 = _mm512_sllv_epi16(a_2, shift_a_2);

            // position the second set of inputs for each of the output values
            let b_32 =
                _mm256_maskz_permutexvar_epi8(0xfeefeefe, perm_b_32, _mm512_castsi512_si256(longs));
            let b_64 = _mm512_maskz_permutexvar_epi8(0xefeefeefeefeefee, perm_b_64, longs);

            // convert the 8-bit values to 16-bit values
            let mut b_0 = _mm512_cvtepu8_epi16(b_32);
            let mut b_1 = _mm512_cvtepu8_epi16(_mm512_castsi512_si256(b_64));
            let b_64 = _mm512_extracti64x4_epi64(b_64, 1);
            let mut b_2 = _mm512_cvtepu8_epi16(b_64);

            // shift the values right by the shift pattern
            b_0 = _mm512_srlv_epi16(b_0, shift_b_0);
            b_1 = _mm512_srlv_epi16(b_1, shift_b_1);
            b_2 = _mm512_srlv_epi16(b_2, shift_b_2);

            // combine the inputs for each output and mask the values to the
            // lower 5 bits
            let mut out_0 = _mm512_or_si512(a_0, b_0);
            out_0 = _mm512_and_si512(out_0, and_pattern);

            let mut out_1 = _mm512_or_si512(a_1, b_1);
            out_1 = _mm512_and_si512(out_1, and_pattern);

            let mut out_2 = _mm512_or_si512(a_2, b_2);
            out_2 = _mm512_and_si512(out_2, and_pattern);

            // store the 16-bit values in the destination
            let dst_offset = offset / 8 * 12;
            _mm512_storeu_epi16(ptr::addr_of_mut!(dst[dst_offset]) as *mut i16, out_0);
            _mm512_storeu_epi16(ptr::addr_of_mut!(dst[dst_offset + 32]) as *mut i16, out_1);
            _mm512_storeu_epi16(ptr::addr_of_mut!(dst[dst_offset + 64]) as *mut i16, out_2);

            offset += 64;
        }
    }

    #[cfg(target_feature = "avx2")]
    unsafe {
        let endian_swap = _mm256_set_epi64x(
            0x08090a0b0c0d0e0f,
            0x0001020304050607,
            0x08090a0b0c0d0e0f,
            0x0001020304050607,
        );
        let perm_a_16 = _mm_set_epi64x(0x0a80090807800605, 0x8004800302800100u64 as i64);
        let perm_a_32 = _mm256_set_epi64x(
            0x0f800e0d800c800b,
            0x0a80090807800605,
            0x800c800b0a800908u64 as i64,
            0x0780060580048003,
        );
        let shift_a_0_lower = _mm256_set_epi64x(
            0x0000000000000002,
            0x0000000000000004,
            0x0000000100000000,
            0x0000000300000000,
        );
        let shift_a_0_upper = _mm256_set_epi64x(
            0x0000000100000000,
            0x0000000300000000,
            0x0000000100000000,
            0x0000000300000000,
        );
        let shift_a_1_lower = _mm256_set_epi64x(
            0x0000000100000000,
            0x0000000300000000,
            0x0000000000000002,
            0x0000000000000004,
        );
        let shift_a_1_upper = _mm256_set_epi64x(
            0x0000000000000002,
            0x0000000000000004,
            0x0000000100000000,
            0x0000000300000000,
        );
        let shift_a_2_lower = _mm256_set_epi64x(
            0x0000000100000000,
            0x0000000300000000,
            0x0000000100000000,
            0x0000000300000000,
        );
        let shift_a_2_upper = _mm256_set_epi64x(
            0x0000000100000000,
            0x0000000300000000,
            0x0000000000000002,
            0x0000000000000004,
        );
        let perm_b_16 = _mm_set_epi64x(0x0909088006060580, 0x0403030201010080);
        let perm_b_32 = _mm256_set_epi64x(
            0x0e0e0d800c0b0b0a,
            0x0909088006060580,
            0x0c0b0b0a09090880,
            0x0606058004030302,
        );
        let shift_b_0_lower = _mm256_set_epi64x(
            0x0000000300000006,
            0x0000000100000004,
            0x0000000700000002,
            0x0000000500000000,
        );
        let shift_b_0_upper = _mm256_set_epi64x(
            0x0000000700000002,
            0x0000000500000000,
            0x0000000700000002,
            0x0000000500000000,
        );
        let shift_b_1_lower = _mm256_set_epi64x(
            0x0000000700000002,
            0x0000000500000000,
            0x0000000300000006,
            0x0000000100000004,
        );
        let shift_b_1_upper = _mm256_set_epi64x(
            0x0000000300000006,
            0x0000000100000004,
            0x0000000700000002,
            0x0000000500000000,
        );
        let shift_b_2_lower = _mm256_set_epi64x(
            0x0000000700000002,
            0x0000000500000000,
            0x0000000700000002,
            0x0000000500000000,
        );
        let shift_b_2_upper = _mm256_set_epi64x(
            0x0000000700000002,
            0x0000000500000000,
            0x0000000300000006,
            0x0000000100000004,
        );
        let and_pattern = _mm256_set1_epi64x(0x001f001f001f001f);

        while offset + 32 <= src.len() {
            let mut longs = _mm256_loadu_si256(ptr::addr_of!(src[offset]) as *const __m256i);
            if big_endian {
                longs = _mm256_shuffle_epi8(longs, endian_swap);
            }
            let longs_perm = _mm256_permute4x64_epi64(longs, 0b11101001);

            // position the first set of inputs for each of the output values
            let a_16 = _mm_shuffle_epi8(_mm256_castsi256_si128(longs), perm_a_16);
            let a_32 = _mm256_shuffle_epi8(longs_perm, perm_a_32);

            // convert the 8-bit values to 16-bit values
            let mut a_0 = _mm256_cvtepu8_epi16(a_16);
            let mut a_1 = _mm256_cvtepu8_epi16(_mm256_castsi256_si128(a_32));
            let a_32_upper = _mm256_extracti128_si256(a_32, 1);
            let mut a_2 = _mm256_cvtepu8_epi16(a_32_upper);

            // convert the 16-bit values to 32-bit values
            let mut a_0_lower = _mm256_cvtepu16_epi32(_mm256_castsi256_si128(a_0));
            let a_0_upper = _mm256_extracti128_si256(a_0, 1);
            let mut a_0_upper = _mm256_cvtepu16_epi32(a_0_upper);

            let mut a_1_lower = _mm256_cvtepu16_epi32(_mm256_castsi256_si128(a_1));
            let a_1_upper = _mm256_extracti128_si256(a_1, 1);
            let mut a_1_upper = _mm256_cvtepu16_epi32(a_1_upper);

            let mut a_2_lower = _mm256_cvtepu16_epi32(_mm256_castsi256_si128(a_2));
            let a_2_upper = _mm256_extracti128_si256(a_2, 1);
            let mut a_2_upper = _mm256_cvtepu16_epi32(a_2_upper);

            // shift the values left by the shift pattern
            a_0_lower = _mm256_sllv_epi32(a_0_lower, shift_a_0_lower);
            a_0_upper = _mm256_sllv_epi32(a_0_upper, shift_a_0_upper);

            a_1_lower = _mm256_sllv_epi32(a_1_lower, shift_a_1_lower);
            a_1_upper = _mm256_sllv_epi32(a_1_upper, shift_a_1_upper);

            a_2_lower = _mm256_sllv_epi32(a_2_lower, shift_a_2_lower);
            a_2_upper = _mm256_sllv_epi32(a_2_upper, shift_a_2_upper);

            // convert the 32-bit values to 16-bit values
            a_0 = _mm256_packus_epi32(a_0_lower, a_0_upper);
            a_0 = _mm256_permute4x64_epi64(a_0, 0b11011000);

            a_1 = _mm256_packus_epi32(a_1_lower, a_1_upper);
            a_1 = _mm256_permute4x64_epi64(a_1, 0b11011000);

            a_2 = _mm256_packus_epi32(a_2_lower, a_2_upper);
            a_2 = _mm256_permute4x64_epi64(a_2, 0b11011000);

            // position the second set of inputs for each of the output values
            let b_16 = _mm_shuffle_epi8(_mm256_castsi256_si128(longs), perm_b_16);
            let b_32 = _mm256_shuffle_epi8(longs_perm, perm_b_32);

            // convert the 8-bit values to 16-bit values
            let mut b_0 = _mm256_cvtepu8_epi16(b_16);
            let mut b_1 = _mm256_cvtepu8_epi16(_mm256_castsi256_si128(b_32));
            let b_32_upper = _mm256_extracti128_si256(b_32, 1);
            let mut b_2 = _mm256_cvtepu8_epi16(b_32_upper);

            // convert the 16-bit values to 32-bit values
            let mut b_0_lower = _mm256_cvtepu16_epi32(_mm256_castsi256_si128(b_0));
            let b_0_upper = _mm256_extracti128_si256(b_0, 1);
            let mut b_0_upper = _mm256_cvtepu16_epi32(b_0_upper);

            let mut b_1_lower = _mm256_cvtepu16_epi32(_mm256_castsi256_si128(b_1));
            let b_1_upper = _mm256_extracti128_si256(b_1, 1);
            let mut b_1_upper = _mm256_cvtepu16_epi32(b_1_upper);

            let mut b_2_lower = _mm256_cvtepu16_epi32(_mm256_castsi256_si128(b_2));
            let b_2_upper = _mm256_extracti128_si256(b_2, 1);
            let mut b_2_upper = _mm256_cvtepu16_epi32(b_2_upper);

            // shift the values right by the shift pattern
            b_0_lower = _mm256_srlv_epi32(b_0_lower, shift_b_0_lower);
            b_0_upper = _mm256_srlv_epi32(b_0_upper, shift_b_0_upper);

            b_1_lower = _mm256_srlv_epi32(b_1_lower, shift_b_1_lower);
            b_1_upper = _mm256_srlv_epi32(b_1_upper, shift_b_1_upper);

            b_2_lower = _mm256_srlv_epi32(b_2_lower, shift_b_2_lower);
            b_2_upper = _mm256_srlv_epi32(b_2_upper, shift_b_2_upper);

            // convert the 32-bit values to 16-bit values
            b_0 = _mm256_packus_epi32(b_0_lower, b_0_upper);
            b_0 = _mm256_permute4x64_epi64(b_0, 0b11011000);

            b_1 = _mm256_packus_epi32(b_1_lower, b_1_upper);
            b_1 = _mm256_permute4x64_epi64(b_1, 0b11011000);

            b_2 = _mm256_packus_epi32(b_2_lower, b_2_upper);
            b_2 = _mm256_permute4x64_epi64(b_2, 0b11011000);

            // combine the inputs for each output and mask the values to the
            // lower 5 bits
            let mut out_0 = _mm256_or_si256(a_0, b_0);
            out_0 = _mm256_and_si256(out_0, and_pattern);

            let mut out_1 = _mm256_or_si256(a_1, b_1);
            out_1 = _mm256_and_si256(out_1, and_pattern);

            let mut out_2 = _mm256_or_si256(a_2, b_2);
            out_2 = _mm256_and_si256(out_2, and_pattern);

            // store the 16-bit values in the destination
            let dst_offset = offset / 8 * 12;
            _mm256_storeu_si256(ptr::addr_of_mut!(dst[dst_offset]) as *mut __m256i, out_0);
            _mm256_storeu_si256(
                ptr::addr_of_mut!(dst[dst_offset + 16]) as *mut __m256i,
                out_1,
            );
            _mm256_storeu_si256(
                ptr::addr_of_mut!(dst[dst_offset + 32]) as *mut __m256i,
                out_2,
            );

            offset += 32;
        }
    }

    // no SSE4.2-only implementation for unpack5 since there aren't variable
    // shift instructions

    while offset + 8 <= src.len() {
        // let dst_offset = offset / 8 * 12;
        // if big_endian {
        //     dst[dst_offset] = src[offset + 7] as u16 & 0x001f;
        //     dst[dst_offset + 1] = (src[offset + 6] << 3 | src[offset + 7] >> 5) as u16 & 0x001f;
        //     dst[dst_offset + 2] = (src[offset + 6] >> 2) as u16 & 0x001f;
        //     dst[dst_offset + 3] = (src[offset + 5] << 1 | src[offset + 6] >> 7) as u16 & 0x001f;
        //     dst[dst_offset + 4] = (src[offset + 4] << 4 | src[offset + 5] >> 4) as u16 & 0x001f;
        //     dst[dst_offset + 5] = (src[offset + 4] >> 1) as u16 & 0x001f;
        //     dst[dst_offset + 6] = (src[offset + 3] << 2 | src[offset + 4] >> 6) as u16 & 0x001f;
        //     dst[dst_offset + 7] = (src[offset + 3] >> 3) as u16 & 0x001f;
        //     dst[dst_offset + 8] = src[offset + 2] as u16 & 0x001f;
        //     dst[dst_offset + 9] = (src[offset + 1] << 3 | src[offset + 2] >> 5) as u16 & 0x001f;
        //     dst[dst_offset + 10] = (src[offset + 1] >> 2) as u16 & 0x001f;
        //     dst[dst_offset + 11] = (src[offset] << 1 | src[offset + 1] >> 7) as u16 & 0x001f;
        // } else {
        //     dst[dst_offset] = src[offset] as u16 & 0x1f;
        //     dst[dst_offset + 1] = (src[offset + 1] << 3 | src[offset] >> 5) as u16 & 0x001f;
        //     dst[dst_offset + 2] = (src[offset + 1] >> 2) as u16 & 0x001f;
        //     dst[dst_offset + 3] = (src[offset + 2] << 1 | src[offset + 1] >> 7) as u16 & 0x001f;
        //     dst[dst_offset + 4] = (src[offset + 3] << 4 | src[offset + 2] >> 4) as u16 & 0x001f;
        //     dst[dst_offset + 5] = (src[offset + 3] >> 1) as u16 & 0x001f;
        //     dst[dst_offset + 6] = (src[offset + 4] << 2 | src[offset + 3] >> 6) as u16 & 0x001f;
        //     dst[dst_offset + 7] = (src[offset + 4] >> 3) as u16 & 0x001f;
        //     dst[dst_offset + 8] = src[offset + 5] as u16 & 0x001f;
        //     dst[dst_offset + 9] = (src[offset + 6] << 3 | src[offset + 5] >> 5) as u16 & 0x001f;
        //     dst[dst_offset + 10] = (src[offset + 6] >> 2) as u16 & 0x001f;
        //     dst[dst_offset + 11] = (src[offset + 7] << 1 | src[offset + 6] >> 7) as u16 & 0x001f;
        // }

        let long = if big_endian {
            u64::from_be_bytes([
                src[offset],
                src[offset + 1],
                src[offset + 2],
                src[offset + 3],
                src[offset + 4],
                src[offset + 5],
                src[offset + 6],
                src[offset + 7],
            ])
        } else {
            u64::from_le_bytes([
                src[offset],
                src[offset + 1],
                src[offset + 2],
                src[offset + 3],
                src[offset + 4],
                src[offset + 5],
                src[offset + 6],
                src[offset + 7],
            ])
        };
        for j in 0..12 {
            dst[offset / 8 * 12 + j] = ((long >> (j * 5)) & 0x1f) as u16;
        }

        offset += 8;
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
    let mut offset: usize = 0;

    #[cfg(all(feature = "nightly", target_feature = "avx512bw"))]
    unsafe {
        let endian_swap = _mm512_set4_epi64(
            0x0c0d0e0f08090a0b,
            0x0405060700010203,
            0x0c0d0e0f08090a0b,
            0x0405060700010203,
        );

        while offset + 64 <= src.len() {
            let longs = _mm512_loadu_epi8(ptr::addr_of!(src[offset]) as *const i8);
            let swapped = _mm512_shuffle_epi8(longs, endian_swap);
            _mm512_storeu_epi8(ptr::addr_of_mut!(dst[offset]) as *mut i8, swapped);

            offset += 64;
        }
    }

    #[cfg(target_feature = "avx2")]
    unsafe {
        let endian_swap = _mm256_set_epi64x(
            0x0c0d0e0f08090a0b,
            0x0405060700010203,
            0x0c0d0e0f08090a0b,
            0x0405060700010203,
        );

        while offset + 32 <= src.len() {
            let longs = _mm256_loadu_si256(ptr::addr_of!(src[offset]) as *const __m256i);
            let swapped = _mm256_shuffle_epi8(longs, endian_swap);
            _mm256_storeu_si256(ptr::addr_of_mut!(dst[offset]) as *mut __m256i, swapped);

            offset += 32;
        }
    }

    #[cfg(target_feature = "ssse3")]
    unsafe {
        let endian_swap = _mm_set_epi64x(0x0c0d0e0f08090a0b, 0x0405060700010203);

        while offset + 16 <= src.len() {
            let longs = _mm_loadu_si128(ptr::addr_of!(src[offset]) as *const __m128i);
            let swapped = _mm_shuffle_epi8(longs, endian_swap);
            _mm_storeu_si128(ptr::addr_of_mut!(dst[offset]) as *mut __m128i, swapped);

            offset += 16;
        }
    }

    while offset + 4 <= src.len() {
        dst[offset..offset + 4].copy_from_slice(&[
            src[offset + 3],
            src[offset + 2],
            src[offset + 1],
            src[offset],
        ]);

        offset += 4;
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

    #[cfg(all(feature = "nightly", target_feature = "avx512bw"))]
    unsafe {
        let endian_swap = _mm512_set4_epi64(
            0x08090a0b0c0d0e0f,
            0x0001020304050607,
            0x08090a0b0c0d0e0f,
            0x0001020304050607,
        );

        while offset + 64 <= src.len() {
            let longs = _mm512_loadu_epi8(ptr::addr_of!(src[offset]) as *const i8);
            let swapped = _mm512_shuffle_epi8(longs, endian_swap);
            _mm512_storeu_epi8(ptr::addr_of_mut!(dst[offset]) as *mut i8, swapped);

            offset += 64;
        }
    }

    #[cfg(target_feature = "avx2")]
    unsafe {
        let endian_swap = _mm256_set_epi64x(
            0x08090a0b0c0d0e0f,
            0x0001020304050607,
            0x08090a0b0c0d0e0f,
            0x0001020304050607,
        );

        while offset + 32 <= src.len() {
            let longs = _mm256_loadu_si256(ptr::addr_of!(src[offset]) as *const __m256i);
            let swapped = _mm256_shuffle_epi8(longs, endian_swap);
            _mm256_storeu_si256(ptr::addr_of_mut!(dst[offset]) as *mut __m256i, swapped);

            offset += 32;
        }
    }

    #[cfg(target_feature = "ssse3")]
    unsafe {
        let endian_swap = _mm_set_epi64x(0x08090a0b0c0d0e0f, 0x0001020304050607);

        while offset + 16 <= src.len() {
            let longs = _mm_loadu_si128(ptr::addr_of!(src[offset]) as *const __m128i);
            let swapped = _mm_shuffle_epi8(longs, endian_swap);
            _mm_storeu_si128(ptr::addr_of_mut!(dst[offset]) as *mut __m128i, swapped);

            offset += 16;
        }
    }

    while offset + 8 <= src.len() {
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

        offset += 8;
    }

    dst
}
