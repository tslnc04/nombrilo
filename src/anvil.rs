use std::io::{Read, Seek, SeekFrom};

use anyhow::bail;
use flate2::read::{GzDecoder, ZlibDecoder};

use crate::{
    chunk_format::Chunk,
    de::{from_reader, from_slice},
};

const SECTOR_SIZE: usize = 4 * 1024;

fn parse_chunk<R>(reader: &mut R) -> anyhow::Result<Chunk>
where
    R: Read,
{
    let mut buf = [0; 5];
    reader.read_exact(&mut buf)?;
    let length = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    let compression_type = buf[4];
    let mut buf = vec![0; length];
    reader.read_exact(&mut buf)?;

    let chunk = match compression_type {
        1 => {
            let mut gz = GzDecoder::new(&buf[..]);
            from_reader(&mut gz)
        }
        2 => {
            let mut zlib = ZlibDecoder::new(&buf[..]);
            from_reader(&mut zlib)
        }
        3 => from_slice(&buf),
        4 => {
            let mut lz4 = lz4_flex::frame::FrameDecoder::new(&buf[..]);
            from_reader(&mut lz4)
        }
        _ => bail!(
            "unknown compression type for Anvil file: {}",
            compression_type
        ),
    };

    Ok(chunk?)
}

/// Parses the chunk at the given x and z region-relative chunk coordinates from
/// the region file. x and z should be in the range 0..32.
pub fn parse_chunk_at<R>(reader: &mut R, x: u8, z: u8) -> anyhow::Result<Chunk>
where
    R: Read + Seek,
{
    let location_offset = (z as u64 * 32 + x as u64) * 4;
    reader.seek(SeekFrom::Start(location_offset))?;
    let mut buf = [0; 3];
    reader.read_exact(&mut buf)?;
    let location = u32::from_be_bytes([0, buf[0], buf[1], buf[2]]);

    if location == 0 {
        bail!("chunk not present in region file");
    }
    reader.seek(SeekFrom::Start(location as u64 * SECTOR_SIZE as u64))?;
    parse_chunk(reader)
}

pub fn parse_region<R>(reader: &mut R) -> anyhow::Result<Vec<Chunk>>
where
    R: Read + Seek,
{
    let mut locations = [0; SECTOR_SIZE];
    reader.read_exact(&mut locations)?;
    let mut timestamps = [0; SECTOR_SIZE];
    reader.read_exact(&mut timestamps)?;

    let mut chunk_locations = Vec::new();
    for z in 0..32usize {
        for x in 0..32usize {
            let offset = (z * 32 + x) * 4;
            // First three bytes are big endian offset in sectors into file
            let location = u32::from_be_bytes([
                0,
                locations[offset],
                locations[offset + 1],
                locations[offset + 2],
            ]);

            if location != 0 {
                chunk_locations.push(location);
            }
        }
    }

    let mut chunks = Vec::<Chunk>::with_capacity(chunk_locations.len());
    for location in chunk_locations {
        reader.seek(SeekFrom::Start(location as u64 * SECTOR_SIZE as u64))?;
        chunks.push(parse_chunk(reader)?);
    }

    Ok(chunks)
}
