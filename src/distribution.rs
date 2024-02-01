use counter::Counter;

use crate::{chunk_format::BlockStates, Chunk};

/// The distribution of block states in the section. Returns a vector with
/// the same length of palette, with each element being the number of blocks
/// with that state.
fn distribution(block_states: &BlockStates) -> Vec<u64> {
    if block_states.data.is_none() {
        return vec![16 * 16 * 16];
    }

    let mut distribution = vec![0; block_states.palette.len()];
    for index in block_states.unpack_data() {
        distribution[index as usize] += 1;
    }
    distribution
}

pub fn chunk(chunk: Chunk) -> Counter<String, u64> {
    let mut chunk_distribution = Counter::new();
    for section in chunk.sections {
        if let Some(block_states) = section.block_states {
            for (count, palette) in distribution(&block_states).iter().zip(block_states.palette) {
                chunk_distribution
                    .entry(palette.name)
                    .and_modify(|e| *e += count)
                    .or_insert(*count);
            }
        }
    }
    chunk_distribution
}
