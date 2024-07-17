use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    nbt::owned::{ByteArray, LongArray, Tag},
    unpack,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    #[serde(rename = "DataVersion")]
    pub data_version: i32,
    #[serde(rename = "xPos")]
    pub x_pos: i32,
    #[serde(rename = "zPos")]
    pub z_pos: i32,
    #[serde(rename = "yPos")]
    pub y_pos: i32,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "LastUpdate")]
    pub last_update: i64,
    pub sections: Vec<Section>,
    pub block_entities: Vec<BlockEntity>,
    #[serde(rename = "HeightMaps")]
    pub height_maps: Option<HeightMaps>,
    #[serde(rename = "InhabitedTime")]
    pub inhabited_time: i64,
    pub blending_data: Option<BlendingData>,
    /// List of lists for each section. Each section list contains shorts which
    /// are packed with three nybbles for chunk relative coordinates of the
    /// block to be ticked. The packing is done as 0ZYX where 0 is the most
    /// signficant nybble.
    #[serde(rename = "PostProcessing")]
    pub post_processing: Vec<Vec<i16>>,
    pub structures: Structures,
    /// Determines whether to load the light data for the chunk. May be omitted
    /// apparently.
    #[serde(rename = "isLightOn")]
    pub is_light_on: Option<bool>,

    /// Omitted if there are no block ticks in the chunk.
    pub block_ticks: Option<Vec<TileTick>>,
    /// Omitted if there are no liquid ticks in the chunk.
    pub fluid_ticks: Option<Vec<TileTick>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    /// Y coordinate of the chunk, ranging from -4 to 20, where the lowest block
    /// in the section is at `y * 16`.
    #[serde(rename = "Y")]
    pub y: i8,
    /// Block states seem to be present in all sections, except for when y = 20,
    /// which represents the section above the build limit.
    pub block_states: Option<BlockStates>,
    pub biomes: Option<Biomes>,
    /// Block light is stored as a packed array of bytes. Each nybble represents
    /// a single block. Omitted if there is no block light data.
    #[serde(rename = "BlockLight")]
    pub block_light: Option<ByteArray>,
    /// Sky light is stored as a packed array of bytes. Each nybble represents a
    /// single block. Omitted if all values 0x00 or 0xff.
    #[serde(rename = "SkyLight")]
    pub sky_light: Option<ByteArray>,
}

/// Block states are stored as a packed array of longs that represent indices
/// into the palette. If there is only one block state in the section then the
/// array is omitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockStates {
    pub palette: Vec<BlockStatePalette>,
    pub data: Option<LongArray>,
}

impl BlockStates {
    /// Gets the block state for a block a the given section-relative coordinates.
    pub fn block(&self, x: usize, y: usize, z: usize) -> &BlockStatePalette {
        if let Some(data) = self.data.as_ref() {
            let packed_index = y * 16 * 16 + z * 16 + x;
            let bits_per_block = self.bits_per_block();
            let blocks_per_long = 64 / bits_per_block;

            let data_index = packed_index / blocks_per_long;
            let long_index = packed_index % blocks_per_long;
            let palette_index = (data.get(data_index).unwrap() as u64
                >> (long_index * bits_per_block))
                & ((1 << bits_per_block) - 1);

            &self.palette[palette_index as usize]
        } else {
            &self.palette[0]
        }
    }

    /// Unpacks the block state data into a flat array of block state indices.
    pub fn unpack_data(&mut self) -> Vec<u16> {
        let bits_per_block = self.bits_per_block();
        if let Some(data) = self.data.as_mut() {
            if self.palette.len() <= 32 {
                // Assumes little endian
                if self.palette.len() <= 16 {
                    return unpack::unpack4(data.as_raw_slice(), data.big_endian());
                }
                return unpack::unpack5(data.as_raw_slice(), data.big_endian());
            }

            let blocks_per_long = 64 / bits_per_block;
            let mut unpacked = Vec::with_capacity(data.len() * blocks_per_long);
            for long in data.as_slice() {
                for i in 0..blocks_per_long {
                    unpacked.push(
                        ((long >> (i * bits_per_block)) & ((1 << bits_per_block) - 1)) as u16,
                    );
                }
            }
            unpacked
        } else {
            vec![0; 16 * 16 * 16]
        }
    }

    /// The number of bits used to store the block state indices. Minimum of 4
    /// and maximum of 12 since palette length is limited to 4096.
    fn bits_per_block(&self) -> usize {
        // Equivalent to ceil(log_2(palette.len())) clamped to 4 and 12.
        (usize::BITS - (self.palette.len() - 1).leading_zeros()).clamp(4, 12) as usize
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockStatePalette {
    #[serde(rename = "Name")]
    pub name: String,
    /// Properties are stored as a map of property name to value. If there are
    /// no properties then the map is omitted.
    pub properties: Option<HashMap<String, String>>,
}

/// Biomes are stored as a packed array of longs that represent indices into the
/// palette. If there is only one biome in the section then the array is
/// omitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Biomes {
    pub palette: Vec<String>,
    pub data: Option<LongArray>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BlockEntity {
    pub id: String,
    #[serde(rename = "keepPacked")]
    keep_packed: Option<bool>,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    /// The rest of the fields are specific to the block entity type.
    #[serde(flatten)]
    pub specific: HashMap<String, Tag>,
}

/// Height maps are stored as a map of name to array of longs. The arrays each
/// hold 256 9-bit values packed into longs.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct HeightMaps {
    pub motion_blocking: LongArray,
    pub motion_blocking_no_leaves: LongArray,
    pub ocean_floor: LongArray,
    pub ocean_floor_wg: LongArray,
    pub world_surface: LongArray,
    pub world_surface_wg: LongArray,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BlendingData {
    pub min_section: i32,
    pub max_section: i32,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Structures {
    /// Structures that have yet to be generated.
    pub starts: HashMap<String, Tag>,
    /// Chunk coordinates of the structure starts. The lower 32 bits are the X
    /// coordinate and the upper 32 bits are the Z coordinate.
    #[serde(rename = "References")]
    pub references: HashMap<String, LongArray>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileTick {
    pub i: String,
    pub p: i32,
    pub t: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
