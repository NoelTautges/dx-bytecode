/*!
DirectX bytecode family parser. Currently only supports DXBC 5.

DXBC 4 and DirectX Intermediate Language (DXIL) may be supported in the future.
*/

#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]

#[cfg(feature = "dxbc")]
pub mod dxbc;
