/*!
[DirectX Bytecode][dxbc] (DXBC) parser. Currently only supports DXBC 5.

DXBC 4 may be supported in the future.

[dxbc]: https://docs.microsoft.com/en-us/windows/win32/direct3dhlsl/shader-model-5-assembly--directx-hlsl-
*/

use std::str::Utf8Error;

use nom::bytes::complete::{tag, take};
use nom::error::{VerboseError, VerboseErrorKind};
use nom::multi::{count, length_count};
use nom::number::complete::le_u32;
use nom::sequence::preceded;
use nom::{Err, IResult};

type Res<T, U> = IResult<T, U, VerboseError<T>>;

/// Chunk type, marked by the FourCC code at the beginning of each chunk.
pub enum ChunkType {
    /// Input signature.
    ISGN,
    /// [Input signature when the shader uses min16float types.](https://twitter.com/aras_p/status/639106535889760257)
    ISG1,
    /// Output signature.
    OSGN,
    /// [Output signature when the shader uses min16float types.](https://twitter.com/aras_p/status/639106535889760257)
    OSG1,
    /// Output signature (SM5).
    OSG5,
    /// Patch constant signature.
    PCSG,
    /// [Interface and class definitions.](https://docs.microsoft.com/en-us/windows/win32/direct3dhlsl/overviews-direct3d-11-hlsl-dynamic-linking-class)
    ///
    /// [SlimShader docs](https://github.com/tgjones/slimshader/blob/master/src/SlimShader/Chunks/Ifce/InterfacesChunk.cs)
    IFCE,
    /// Resource definitions.
    RDEF,
    /// Enables features like half- or double-precision floating points and [structured buffers][buffers].
    ///
    /// [SlimShader docs](https://github.com/tgjones/slimshader/blob/master/src/SlimShader/Chunks/Sfi0/Sfi0Chunk.cs)
    ///
    /// [buffers]: https://docs.microsoft.com/en-us/windows/win32/direct3d11/direct3d-11-advanced-stages-cs-resources
    SFI0,
    /// Shader assembly (DX9).
    Aon9,
    /// Shader assembly (SM4).
    SHDR,
    /// Shader assembly (SM5).
    SHEX,
    /// Shader statistics.
    STAT,
    /// Shader debug info (old).
    SDGB,
    /// Shader debug info (new).
    SPDB,
}

/// DXBC chunk.
pub struct Chunk {
    /// Chunk type, determined by FourCC code.
    pub ty: ChunkType,
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
    let ty = match four_cc {
        "ISGN" => ChunkType::ISGN,
        "ISG1" => ChunkType::ISG1,
        "OSGN" => ChunkType::OSGN,
        "OSG1" => ChunkType::OSG1,
        "OSG5" => ChunkType::OSG5,
        "PCSG" => ChunkType::PCSG,
        "IFCE" => ChunkType::IFCE,
        "RDEF" => ChunkType::RDEF,
        "SFI0" => ChunkType::SFI0,
        "Aon9" => ChunkType::Aon9,
        "SHDR" => ChunkType::SHDR,
        "SHEX" => ChunkType::SHEX,
        "STAT" => ChunkType::STAT,
        "SDGB" => ChunkType::SDGB,
        "SPDB" => ChunkType::SPDB,
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
