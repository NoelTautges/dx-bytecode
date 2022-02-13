/*!
[DirectX Bytecode][dxbc] (DXBC) parser.

[dxbc]: https://docs.microsoft.com/en-us/windows/win32/direct3dhlsl/shader-model-5-assembly--directx-hlsl-
*/

mod chunks;

use nom::bytes::complete::{tag, take};
use nom::multi::{count, length_count};
use nom::number::complete::le_u32;
use nom::sequence::preceded;

use crate::utils::{Res, to_err};
use chunks::ChunkVariant;

/// DXBC chunk.
#[derive(Debug)]
pub struct Chunk {
    /// Chunk length.
    pub len: u32,
    /// Chunk variant.
    pub variant: ChunkVariant,
}

/// Parsed bytecode object, including the header, chunks, and assembly.
pub struct Bytecode {
    /// [Custom MD5 checksum of all chunks in the shader.](https://github.com/GPUOpen-Archive/common-src-ShaderUtils/blob/master/DX10/DXBCChecksum.cpp)
    pub checksum: [u8; 16],
}

/// Parses a DXBC chunk from bytes.
fn chunk(input: &[u8]) -> Res<Chunk> {
    let (rest, four_cc) = take(4usize)(input)?;
    let (rest, len) = le_u32(rest)?;
    let (rest, data) = take(len)(rest)?;
    let variant = match four_cc {
        b"ISGN" => ChunkVariant::ISGN,
        b"ISG1" => ChunkVariant::ISG1,
        b"OSGN" => ChunkVariant::OSGN,
        b"OSG1" => ChunkVariant::OSG1,
        b"OSG5" => ChunkVariant::OSG5,
        b"PCSG" => ChunkVariant::PCSG,
        b"IFCE" => ChunkVariant::IFCE,
        b"RDEF" => ChunkVariant::RDEF(chunks::rdef(data)?.1),
        b"SFI0" => ChunkVariant::SFI0,
        b"Aon9" => ChunkVariant::Aon9,
        b"SHDR" => ChunkVariant::SHDR,
        b"SHEX" => ChunkVariant::SHEX,
        b"STAT" => ChunkVariant::STAT,
        b"SDGB" => ChunkVariant::SDGB,
        b"SPDB" => ChunkVariant::SPDB,
        _ => return Err(to_err(data, "Unknown chunk type!")),
    };

    Ok((rest, Chunk { len, variant }))
}

/// Parses a bytecode object from bytes.
pub fn parse_dxbc(input: &[u8]) -> Res<Bytecode> {
    tag("DXBC")(input)?;
    let (rest, checksum) = preceded(tag("DXBC"), take(16u8))(input)?;
    let (rest, _) = tag("\x01\x00\x00\x00")(rest)?;
    let (rest, len) = le_u32(rest)?;
    let len = len as usize;
    if len != input.len() {
        return Err(to_err(rest, "Wrong shader length!"));
    }
    let (rest, offsets) = length_count(le_u32, le_u32)(rest)?;
    let (rest, _chunks) = count(chunk, offsets.len())(rest)?;

    Ok((
        rest,
        Bytecode {
            checksum: checksum.try_into().unwrap(),
        },
    ))
}
