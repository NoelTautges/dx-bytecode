/// Program type.
#[derive(Debug)]
pub enum ProgramType {
    /// [Vertex shader](https://docs.microsoft.com/en-us/windows/win32/direct3d11/vertex-shader-stage).
    VertexShader,
    /// [Pixel shader](https://docs.microsoft.com/en-us/windows/win32/direct3d11/pixel-shader-stage).
    PixelShader,
    /// [Geometry shader](https://docs.microsoft.com/en-us/windows/win32/direct3d11/geometry-shader-stage).
    GeometryShader,
    /// [Hull shader](https://docs.microsoft.com/en-us/windows/win32/direct3d11/direct3d-11-advanced-stages-tessellation#hull-shader-stage).
    HullShader,
    /// [Domain shader](https://docs.microsoft.com/en-us/windows/win32/direct3d11/direct3d-11-advanced-stages-tessellation#domain-shader-stage).
    DomainShader,
    /// [Compute shader](https://docs.microsoft.com/en-us/windows/win32/direct3d11/direct3d-11-advanced-stages-compute-shader).
    ComputeShader,
}

impl TryFrom<u16> for ProgramType {
    type Error = &'static str;

    fn try_from(num: u16) -> Result<Self, Self::Error> {
        match num {
            0xFFFE => Ok(ProgramType::VertexShader),
            0xFFFF => Ok(ProgramType::PixelShader),
            0x4753 => Ok(ProgramType::GeometryShader),
            0x4853 => Ok(ProgramType::HullShader),
            0x4453 => Ok(ProgramType::DomainShader),
            0x4353 => Ok(ProgramType::ComputeShader),
            _ => Err("Unknown program type!"),
        }
    }
}

/// Shader version.
#[derive(Debug)]
pub struct ShaderVersion {
    /// Major version.
    pub major: u8,
    /// Minor version.
    pub minor: u8,
    /// Program type.
    pub program_type: ProgramType,
}
