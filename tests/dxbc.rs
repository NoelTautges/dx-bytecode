use dx_bytecode::dxbc::parse_dxbc;
use std::fs;
use test_generator::test_resources;

#[test_resources("shaders/**/*.dxbc")]
fn parse_shader(shader_path: &str) {
    let bytes = fs::read(shader_path);
    assert!(bytes.is_ok(), "Couldn't read shader!");
    let bytes = bytes.unwrap();
    let dxbc = parse_dxbc(&bytes);
    assert!(dxbc.is_ok(), "Couldn't parse shader!");
}
