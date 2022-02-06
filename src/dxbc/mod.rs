/*!
[DirectX Bytecode][dxbc] (DXBC) parser.

[dxbc]: https://docs.microsoft.com/en-us/windows/win32/direct3dhlsl/shader-model-5-assembly--directx-hlsl-
*/

mod chunks;

use std::str::Utf8Error;

use nom::Err;
use nom::bytes::complete::{tag, take};
use nom::error::{VerboseError, VerboseErrorKind};
use nom::multi::{count, length_count};
use nom::number::complete::le_u32;
use nom::sequence::preceded;

use crate::utils::Res;
use chunks::ChunkVariant;

/// DXBC chunk.
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
fn chunk(input: &[u8]) -> Res<&[u8], Chunk> {
    let (rest, four_cc) = take(4usize)(input)?;
    let four_cc = match std::str::from_utf8(four_cc) {
        Ok(s) => s,
        Err(Utf8Error { .. }) => {
            return Err(Err::Failure(VerboseError {
                errors: vec![(
                    rest,
                    VerboseErrorKind::Context("UTF-8 error decoding chunk FourCC!"),
                )],
            }))
        }
    };
    let (rest, len) = le_u32(rest)?;
    let mut rest = rest;
    let variant = match four_cc {
        "ISGN" => ChunkVariant::ISGN,
        "ISG1" => ChunkVariant::ISG1,
        "OSGN" => ChunkVariant::OSGN,
        "OSG1" => ChunkVariant::OSG1,
        "OSG5" => ChunkVariant::OSG5,
        "PCSG" => ChunkVariant::PCSG,
        "IFCE" => ChunkVariant::IFCE,
        "RDEF" => {
            let chunk = chunks::rdef(rest)?;
            rest = chunk.0;
            ChunkVariant::RDEF(chunk.1)
        },
        "SFI0" => ChunkVariant::SFI0,
        "Aon9" => ChunkVariant::Aon9,
        "SHDR" => ChunkVariant::SHDR,
        "SHEX" => ChunkVariant::SHEX,
        "STAT" => ChunkVariant::STAT,
        "SDGB" => ChunkVariant::SDGB,
        "SPDB" => ChunkVariant::SPDB,
        _ => {
            return Err(Err::Failure(VerboseError {
                errors: vec![(rest, VerboseErrorKind::Context("Unknown chunk type!"))],
            }))
        }
    };

    Ok((rest, Chunk { len, variant }))
}

/// Parses a bytecode object from bytes.
pub fn parse_dxbc(input: &[u8]) -> Res<&[u8], Bytecode> {
    tag("DXBC")(input)?;
    let (rest, checksum) = preceded(tag("DXBC"), take(16u8))(input)?;
    let (rest, _) = tag("\x01\x00\x00\x00")(rest)?;
    let (rest, len) = le_u32(rest)?;
    let len = len as usize;
    if len != input.len() {
        return Err(Err::Failure(VerboseError {
            errors: vec![(rest, VerboseErrorKind::Context("Wrong shader length!"))],
        }));
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
