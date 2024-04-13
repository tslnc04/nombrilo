// Use the amd64 implementation if the target architecture is x86_64 and the
// simd feature is enabled. Within the amd64 implementation, the nightly feature
// (as well as target_feature) control AVX512 support.
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
mod unpack_amd64;
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
pub(crate) use unpack_amd64::*;

// Fallback to portable simd implementation if the target architecture is not
// x86_64 but the simd and nightly features are enabled.
#[cfg(all(feature = "simd", feature = "nightly", not(target_arch = "x86_64")))]
mod unpack_portable;
#[cfg(all(feature = "simd", feature = "nightly", not(target_arch = "x86_64")))]
pub(crate) use unpack_portable::*;

// Fallback to the no-simd implementation if the simd feature is not enabled or
// the nightly feature is not enabled and the target architecture is not x86_64.
#[cfg(not(all(feature = "simd", any(feature = "nightly", target_arch = "x86_64"))))]
mod nosimd;
#[cfg(not(all(feature = "simd", any(feature = "nightly", target_arch = "x86_64"))))]
pub(crate) use nosimd::*;

mod tests;
