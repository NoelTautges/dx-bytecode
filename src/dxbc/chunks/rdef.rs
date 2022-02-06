use nom::number::complete::{le_u8, le_u16, le_u32};

use crate::utils::{Res, to_err};
use super::common::{ShaderVersion, ProgramType};

#[derive(Debug)]
pub struct Rdef {
    pub version: ShaderVersion,
}

pub fn rdef(input: &[u8]) -> Res<&[u8], Rdef> {
    let (rest, cb_count) = le_u32(input)?;
    let (rest, cb_offset) = le_u32(rest)?;
    let (rest, rb_count) = le_u32(rest)?;
    let (rest, rb_offset) = le_u32(rest)?;

    let (rest, minor) = le_u8(rest)?;
    let (rest, major) = le_u8(rest)?;
    let (rest, program_type) = le_u16(rest)?;
    let program_type = match ProgramType::try_from(program_type) {
        Ok(program_type) => program_type,
        Err(e) => return Err(to_err(rest, e)),
    };
    let version = ShaderVersion {
        major,
        minor,
        program_type,
    };
    
    let (rest, flags) = le_u32(rest)?;

    Ok((rest, Rdef {
        version,
    }))
}