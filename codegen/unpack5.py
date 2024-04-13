def repeat_pattern(
    pattern: list[int], repeat: int, increase: bool = False
) -> list[int]:
    """Repeats the pattern, but increasing the indices by 8 each time, unless
    the value is 0x80, if increase is True."""
    result = []
    for i in range(repeat):
        for j in range(len(pattern)):
            if pattern[j] == 0x80:
                result.append(0x80)
            else:
                result.append(pattern[j] + (8 * i if increase else 0))
    return result


def intify_pattern(
    pattern: list[int], size: int = 1, replace: bool = True
) -> list[int]:
    """Zero extend a list of bytes to size bytes (where size is a power of 2)
    and then convert to a list of 64 bit integers. Replaces 0x80 with 0 when
    replace is True."""
    assert len(pattern) % 8 == 0

    bytes_per_result = 8 // size
    results = []
    for i in range(0, len(pattern), bytes_per_result):
        result = 0
        for j in range(0, 8, size):
            pattern_byte = pattern[i + j // size]
            pattern_byte = 0 if pattern_byte == 0x80 and replace else pattern_byte
            result |= pattern_byte << (j * 8)
        results.append(result)
    return results


def hexlist_ints(ints: list[int], reverse: bool = True) -> str:
    """Converts a list of 64 bit integers to a string of hex values, reversing
    the order if reverse is True."""
    ints = ints[::-1] if reverse else ints
    return ", ".join([f"0x{x:016x}" for x in ints])


def generate_mask(pattern: list[int]) -> int:
    """Generates a mask for the given patter, where 0x80 corresponds to a 0 bit
    and any other value corresponds to a 1 bit."""
    mask = 0
    for i in range(len(pattern)):
        mask |= (pattern[i] != 0x80) << i
    return mask


def shift_indices(pattern: list[int], shift: int) -> list[int]:
    """Shifts the indices in the pattern by the given amount, but preserves
    0x80."""
    return [x + shift if x != 0x80 else x for x in pattern]


def generate_intrinsics_avx512(
    perm_pattern: list[int],
    shift_pattern: list[int],
    name: str,
    shift_right: bool = False,
) -> list[str]:
    """Generates the intrinsics for the given 12 byte pattern using AVX-512."""
    assert len(perm_pattern) == 12
    assert len(shift_pattern) == 12

    repeated_perm = repeat_pattern(perm_pattern, 8, increase=True)
    split_32, split_64 = repeated_perm[:32], repeated_perm[32:]
    mask_32 = generate_mask(split_32)
    mask_64 = generate_mask(split_64)
    intify_32 = hexlist_ints(intify_pattern(split_32))
    intify_64 = hexlist_ints(intify_pattern(split_64))

    repeated_shift = repeat_pattern(shift_pattern, 8)
    shift_0 = hexlist_ints(intify_pattern(repeated_shift[:32], 2))
    shift_1 = hexlist_ints(intify_pattern(repeated_shift[32:64], 2))
    shift_2 = hexlist_ints(intify_pattern(repeated_shift[64:], 2))

    code = [
        # Create the patterns for permutations
        f"let perm_{name}_32 = _mm256_set_epi64x({intify_32});",
        f"let perm_{name}_64 = _mm512_set_epi64({intify_64});",
        # Create the patterns for shifts
        f"let shift_{name}_0 = _mm512_set_epi64({shift_0});",
        f"let shift_{name}_1 = _mm512_set_epi64({shift_1});",
        f"let shift_{name}_2 = _mm512_set_epi64({shift_2});",
        # Permute the values, code starting here goes inside the loop
        f"let {name}_32 = _mm256_maskz_permutexvar_epi8(0x{mask_32:08x}, perm_{name}_32, _mm512_castsi512_si256(longs));",
        f"let {name}_64 = _mm512_maskz_permutexvar_epi8(0x{mask_64:16x}, perm_{name}_64, longs);",
        # Extend the 8 bit values to 16 bit values
        f"let mut {name}_0 = _mm512_cvtepu8_epi16({name}_32);",
        f"let mut {name}_1 = _mm512_cvtepu8_epi16(_mm512_castsi512_si256({name}_64));",
        f"let {name}_64 = _mm512_extracti64x4_epi64({name}_64, 1);",
        f"let mut {name}_2 = _mm512_cvtepu8_epi16({name}_64);",
        # Shift the values
        f"{name}_0 = _mm512_s{'r' if shift_right else 'l'}lv_epi16({name}_0, shift_{name}_0);",
        f"{name}_1 = _mm512_s{'r' if shift_right else 'l'}lv_epi16({name}_1, shift_{name}_1);",
        f"{name}_2 = _mm512_s{'r' if shift_right else 'l'}lv_epi16({name}_2, shift_{name}_2);",
    ]

    return code


def generate_intrinsics_avx2(
    perm_pattern: list[int],
    shift_pattern: list[int],
    name: str,
    shift_right: bool = False,
) -> list[str]:
    """Generates the intrinsics for the given 12 byte pattern using AVX2."""
    assert len(perm_pattern) == 12
    assert len(shift_pattern) == 12

    repeated_perm = repeat_pattern(perm_pattern, 4, increase=True)
    split_16, split_32 = repeated_perm[:16], repeated_perm[16:]
    # To account for in lane shuffles, shift the indices in the first half by 8 and the second half by 16
    split_32 = shift_indices(split_32[:16], -8) + shift_indices(split_32[16:], -16)
    intify_16 = hexlist_ints(intify_pattern(split_16, replace=False))
    intify_32 = hexlist_ints(intify_pattern(split_32, replace=False))

    repeated_shift = repeat_pattern(shift_pattern, 4)
    shift_0_lower = hexlist_ints(intify_pattern(repeated_shift[:8], 4))
    shift_0_upper = hexlist_ints(intify_pattern(repeated_shift[8:16], 4))
    shift_1_lower = hexlist_ints(intify_pattern(repeated_shift[16:24], 4))
    shift_1_upper = hexlist_ints(intify_pattern(repeated_shift[24:32], 4))
    shift_2_lower = hexlist_ints(intify_pattern(repeated_shift[32:40], 4))
    shift_2_upper = hexlist_ints(intify_pattern(repeated_shift[40:], 4))

    code = [
        # Create the patterns for permutations
        f"let perm_{name}_16 = _mm_set_epi64x({intify_16});",
        f"let perm_{name}_32 = _mm256_set_epi64x({intify_32});",
        # Create the patterns for shifts
        f"let shift_{name}_0_lower = _mm256_set_epi64x({shift_0_lower});",
        f"let shift_{name}_0_upper = _mm256_set_epi64x({shift_0_upper});",
        f"let shift_{name}_1_lower = _mm256_set_epi64x({shift_1_lower});",
        f"let shift_{name}_1_upper = _mm256_set_epi64x({shift_1_upper});",
        f"let shift_{name}_2_lower = _mm256_set_epi64x({shift_2_lower});",
        f"let shift_{name}_2_upper = _mm256_set_epi64x({shift_2_upper});",
        # Permute the values, code starting here goes inside the loop
        f"let {name}_16 = _mm_shuffle_epi8(_mm256_castsi256_si128(longs), perm_{name}_16);",
        # Permute the values for the upper 32 bytes
        f"let {name}_32 = _mm256_shuffle_epi8(longs_perm, perm_{name}_32);",
        # Extend the 8 bit values to 16 bit values
        f"let mut {name}_0 = _mm256_cvtepu8_epi16({name}_16);",
        f"let mut {name}_1 = _mm256_cvtepu8_epi16(_mm256_castsi256_si128({name}_32));",
        f"let {name}_32_upper = _mm256_extracti128_si256({name}_32, 1);",
        f"let mut {name}_2 = _mm256_cvtepu8_epi16({name}_32_upper);",
        # Extend the 16 bit values to 32 bit values
        f"let mut {name}_0_lower = _mm256_cvtepu16_epi32(_mm256_castsi256_si128({name}_0));",
        f"let {name}_0_upper = _mm256_extracti128_si256({name}_0, 1);",
        f"let mut {name}_0_upper = _mm256_cvtepu16_epi32({name}_0_upper);",
        f"let mut {name}_1_lower = _mm256_cvtepu16_epi32(_mm256_castsi256_si128({name}_1));",
        f"let {name}_1_upper = _mm256_extracti128_si256({name}_1, 1);",
        f"let mut {name}_1_upper = _mm256_cvtepu16_epi32({name}_1_upper);",
        f"let mut {name}_2_lower = _mm256_cvtepu16_epi32(_mm256_castsi256_si128({name}_2));",
        f"let {name}_2_upper = _mm256_extracti128_si256({name}_2, 1);",
        f"let mut {name}_2_upper = _mm256_cvtepu16_epi32({name}_2_upper);",
        # Shift the values
        f"{name}_0_lower = _mm256_s{'r' if shift_right else 'l'}lv_epi32({name}_0_lower, shift_{name}_0_lower);",
        f"{name}_0_upper = _mm256_s{'r' if shift_right else 'l'}lv_epi32({name}_0_upper, shift_{name}_0_upper);",
        f"{name}_1_lower = _mm256_s{'r' if shift_right else 'l'}lv_epi32({name}_1_lower, shift_{name}_1_lower);",
        f"{name}_1_upper = _mm256_s{'r' if shift_right else 'l'}lv_epi32({name}_1_upper, shift_{name}_1_upper);",
        f"{name}_2_lower = _mm256_s{'r' if shift_right else 'l'}lv_epi32({name}_2_lower, shift_{name}_2_lower);",
        f"{name}_2_upper = _mm256_s{'r' if shift_right else 'l'}lv_epi32({name}_2_upper, shift_{name}_2_upper);",
        # Saturate the 32 bit values to 16 bit values
        f"{name}_0 = _mm256_packus_epi32({name}_0_lower, {name}_0_upper);",
        f"{name}_0 = _mm256_permute4x64_epi64({name}_0, 0b11011000);",
        f"{name}_1 = _mm256_packus_epi32({name}_1_lower, {name}_1_upper);",
        f"{name}_1 = _mm256_permute4x64_epi64({name}_1, 0b11011000);",
        f"{name}_2 = _mm256_packus_epi32({name}_2_lower, {name}_2_upper);",
        f"{name}_2 = _mm256_permute4x64_epi64({name}_2, 0b11011000);",
    ]

    return code


if __name__ == "__main__":
    perm_pattern_a = [0, 1, 0x80, 2, 3, 0x80, 4, 0x80, 5, 6, 0x80, 7]
    shift_pattern_a = [0, 3, 0, 1, 4, 0, 2, 0, 0, 3, 0, 1]
    avx512_a = generate_intrinsics_avx512(perm_pattern_a, shift_pattern_a, "a")

    perm_pattern_b = [0x80, 0, 1, 1, 2, 3, 3, 4, 0x80, 5, 6, 6]
    shift_pattern_b = [0, 5, 2, 7, 4, 1, 6, 3, 0, 5, 2, 7]
    avx512_b = generate_intrinsics_avx512(
        perm_pattern_b, shift_pattern_b, "b", shift_right=True
    )

    avx512 = avx512_a[:5] + avx512_b[:5] + avx512_a[5:] + avx512_b[5:]
    print("AVX-512")
    for line in avx512:
        print(line)

    avx2_a = generate_intrinsics_avx2(perm_pattern_a, shift_pattern_a, "a")
    avx2_b = generate_intrinsics_avx2(
        perm_pattern_b, shift_pattern_b, "b", shift_right=True
    )

    avx2 = avx2_a[:8] + avx2_b[:8] + avx2_a[8:] + avx2_b[8:]
    print("AVX2")
    for line in avx2:
        print(line)
