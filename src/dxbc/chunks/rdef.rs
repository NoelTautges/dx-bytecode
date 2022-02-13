use bitflags::bitflags;
use nom::{number::complete::{le_u8, le_u16, le_u32}, bytes::complete::{take_until, take}, multi::count};

use crate::utils::{Res, take_string, to_err};
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
pub struct ConstantBuffer {
    pub name: String,
}

#[derive(Debug)]
pub struct ResourceBinding {

}

/// RDEF chunk data.
#[derive(Debug)]
pub struct Rdef {
    pub version: ShaderVersion,
    pub flags: ShaderFlags,
    pub creator: String,
    pub interface_slot_count: Option<u32>,
    pub constant_buffers: Vec<ConstantBuffer>,
    pub resource_bindings: Vec<ResourceBinding>,
}

/// Parses an RD11 section.
/// 
/// Only found in Shader Model 5.
fn rdef_rd11(input: &[u8]) -> Res<u32> {
    let (rest, unknown1) = le_u32(input)?;
    if unknown1 != 60 {
        return Err(to_err(rest, "Unknown 1 in SM5 RDEF RD11 section had unknown value != 60!"));
    }
    let (rest, unknown2) = le_u32(rest)?;
    if unknown2 != 24 {
        return Err(to_err(rest, "Unknown 2 in SM5 RDEF RD11 section had unknown value != 24!"));
    }
    let (rest, unknown3) = le_u32(rest)?;
    if unknown3 != 32 {
        return Err(to_err(rest, "Unknown 3 in SM5 RDEF RD11 section had unknown value != 32!"));
    }
    let (rest, unknown4) = le_u32(rest)?;
    if unknown4 != 40 {
        return Err(to_err(rest, "Unknown 4 in SM5 RDEF RD11 section had unknown value != 40!"));
    }
    let (rest, unknown5) = le_u32(rest)?;
    if unknown5 != 36 {
        return Err(to_err(rest, "Unknown 5 in SM5 RDEF RD11 section had unknown value != 36!"));
    }
    let (rest, unknown6) = le_u32(rest)?;
    if unknown6 != 12 {
        return Err(to_err(rest, "Unknown 6 in SM5 RDEF RD11 section had unknown value != 12!"));
    }
    let (rest, interface_slot_count) = le_u32(input)?;

    Ok((rest, interface_slot_count))
}

/// Parses a constant buffer.
fn constant_buffer(input: &[u8]) -> Res<ConstantBuffer> {
    println!("{:?}", input);
    let (rest, name_offset) = le_u32(input)?;
    let (name_bytes, _) = take(name_offset)(input)?;
    let (rest, var_count) = le_u32(rest)?;
    let (rest, var_offset) = le_u32(rest)?;

    println!("{}\n{:?}", name_offset, name_bytes);
    let (_, name) = take_string(name_bytes)?;

    Ok((rest, ConstantBuffer {
        name,
    }))
}

/// Parses an RDEF chunk.
pub fn rdef(input: &[u8]) -> Res<Rdef> {
    // Constant buffers
    let (rest, cb_count) = le_u32(input)?;
    let (rest, cb_offset) = le_u32(rest)?;
    // Resource bindings
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
    let (_, creator) = take_string(creator_bytes)?;

    // TODO: Lax Shader Model
    let (rd11_bytes, _) = take(4usize)(rest)?;
    let interface_slot_count = if rd11_bytes == b"RD11" {
        Some(rdef_rd11(input)?.1)
    } else {
        None
    };

    let (cb_bytes, _) = take(cb_offset)(input)?;
    let (_, constant_buffers) = count(
        constant_buffer,
        cb_count as usize,
    )(cb_bytes)?;

    let (rb_bytes, _) = take(rb_offset)(input)?;

    Ok((rest, Rdef {
        version,
        flags,
        creator,
        interface_slot_count,
        constant_buffers,
        resource_bindings: vec![],
    }))
}
