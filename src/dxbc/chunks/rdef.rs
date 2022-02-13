use bitflags::bitflags;
use nom::{number::complete::{le_u8, le_u16, le_u32}, bytes::complete::{take_until, take}};

use crate::utils::{Res, to_err};
use super::common::{ShaderVersion, ProgramType};

bitflags! {
    #[derive(Default)]
    pub struct ShaderFlags: u32 {
        const DEBUG                             = 0b0000000000000000001;
        const SKIP_VALIDATION                   = 0b0000000000000000010;
        const SKIP_OPTIMIZATION                 = 0b0000000000000000100;
        const PACK_MATRIX_ROW_MAJOR             = 0b0000000000000001000;
        const PACK_MATRIX_COLUMN_MAJOR          = 0b0000000000000010000;
        const PARTIAL_PRECISION                 = 0b0000000000000100000;
        const FORCE_VS_SOFTWARE_NO_OPT          = 0b0000000000001000000;
        const FORCE_PS_SOFTWARE_NO_OPT          = 0b0000000000010000000;
        const NO_PRESHADER                      = 0b0000000000100000000;
        const AVOID_FLOW_CONTROL                = 0b0000000001000000000;
        const PREFER_FLOW_CONTROL               = 0b0000000010000000000;
        const ENABLE_STRICTNESS                 = 0b0000000100000000000;
        const ENABLE_BACKWARDS_COMPATIBILITY    = 0b0000001000000000000;
        const IEEE_STRICTNESS                   = 0b0000010000000000000;
        const OPTIMIZATION_LEVEL_0              = 0b0000100000000000000;
        const OPTIMIZATION_LEVEL_1              = 0b0000000000000000000;
        const OPTIMIZATION_LEVEL_2              = 0b0001100000000000000;
        const OPTIMIZATION_LEVEL_3              = 0b0001000000000000000;
        const RESERVED_16                       = 0b0010000000000000000;
        const RESERVED_17                       = 0b0100000000000000000;
        const WARNINGS_ARE_ERRORS               = 0b1000000000000000000;
    }
}

#[derive(Debug)]
pub struct Rdef {
    pub version: ShaderVersion,
    pub flags: ShaderFlags,
    pub creator: String,
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
    let flags = ShaderFlags::from_bits_truncate(flags);
    let (rest, creator_offset) =  le_u32(rest)?;
    let (creator_bytes, _) = take(creator_offset)(input)?;
    let (_, creator) = take_until("\0")(creator_bytes)?;
    let creator = match String::from_utf8(creator.to_vec()) {
        Ok(s) => s,
        Err(_) => return Err(to_err(creator_bytes, "Couldn't convert creator string to UTF-8!")),
    };

    Ok((rest, Rdef {
        version,
        flags,
        creator,
    }))
}