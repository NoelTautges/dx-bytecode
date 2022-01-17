/*!
[DirectX Bytecode][dxbc] (DXBC) parser. Currently only supports DXBC 5.

DXBC 4 may be supported in the future.

[dxbc]: https://docs.microsoft.com/en-us/windows/win32/direct3dhlsl/shader-model-5-assembly--directx-hlsl-
*/

use nom::bytes::complete::{tag, take};
use nom::error::{VerboseError, VerboseErrorKind};
use nom::multi::{count, length_count};
use nom::number::complete::le_u32;
use nom::sequence::preceded;
use nom::{Err, IResult};

type Res<T, U> = IResult<T, U, VerboseError<T>>;

/// Chunk type, marked by the FourCC code at the beginning of each chunk.
pub enum ChunkType {
    IFCE,
    ISGN,
    OSGN,
    OSG5,
    PCSG,
    RDEF,
    SFI0,
    SHDR,
    SHEX,
    SPDB,
    STAT,
}

/// DXBC chunk.
pub struct Chunk {
    pub ty: ChunkType,
}

/// Parsed bytecode object, including the header, chunks, and assembly.
pub struct Bytecode {
    pub checksum: [u8; 16],
}

/// Parses a DXBC chunk from bytes.
fn chunk(input: &[u8]) -> Res<&[u8], Chunk> {
    let (rest, four_cc) = take(4usize)(input)?;
    let ty = match four_cc {
        b"\x49\x46\x43\x45" => ChunkType::IFCE,
        b"\x49\x53\x47\x4E" => ChunkType::ISGN,
        b"\x4F\x53\x47\x4E" => ChunkType::OSGN,
        b"\x4F\x53\x47\x35" => ChunkType::OSG5,
        b"\x50\x43\x53\x47" => ChunkType::PCSG,
        b"\x52\x44\x45\x46" => ChunkType::RDEF,
        b"\x53\x46\x49\x30" => ChunkType::SFI0,
        b"\x53\x48\x44\x52" => ChunkType::SHDR,
        b"\x53\x48\x45\x58" => ChunkType::SHEX,
        b"\x53\x50\x44\x42" => ChunkType::SPDB,
        b"\x53\x54\x41\x54" => ChunkType::STAT,
        _ => {
            return Err(Err::Failure(VerboseError {
                errors: vec![(rest, VerboseErrorKind::Context("Unknown chunk type!"))],
            }))
        }
    };
    let (rest, len) = le_u32(rest)?;
    let (rest, _data) = take(len)(rest)?;

    Ok((rest, Chunk { ty }))
}

/// Parses a bytecode object from bytes.
pub fn get_dxbc(input: &[u8]) -> Res<&[u8], Bytecode> {
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
