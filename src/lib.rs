#![cfg_attr(
    all(feature = "simd", feature = "nightly", not(target_arch = "x86_64")),
    feature(portable_simd),
    feature(generic_const_exprs)
)]
#![cfg_attr(
    all(
        feature = "nightly",
        target_arch = "x86_64",
        target_feature = "avx512bw"
    ),
    feature(stdarch_x86_avx512)
)]

pub mod anvil;
pub mod chunk_format;
pub mod de;
pub mod distribution;
pub mod nbt;
pub mod ser;

pub mod unpack;

pub use anvil::parse_chunk_at;
pub use anvil::parse_region;
pub use chunk_format::Chunk;
