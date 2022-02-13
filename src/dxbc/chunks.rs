mod common;
mod rdef;

pub use rdef::*;

/// DXBC chunks.
#[derive(Debug)]
pub enum ChunkVariant {
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
    RDEF(Rdef),
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
