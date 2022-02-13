use nom::bytes::complete::take_until;
use nom::{Err, IResult};
use nom::error::{VerboseError, VerboseErrorKind};

pub type Res<'a, U> = IResult<&'a [u8], U, VerboseError<&'a [u8]>>;

pub fn to_err<'a>(rest: &'a [u8], error: &'static str) -> Err<VerboseError<&'a [u8]>> {
    Err::Failure(VerboseError {
        errors: vec![(
            rest,
            VerboseErrorKind::Context(error),
        )],
    })
}

pub fn take_string(rest: &[u8]) -> IResult<&[u8], String, VerboseError<&[u8]>> {
    let (rest, str_bytes) = take_until("\0")(rest)?;
    match String::from_utf8(str_bytes.to_vec()) {
        Ok(s) => Ok((rest, s)),
        Err(_) => Err(to_err(str_bytes, "Couldn't convert creator string to UTF-8!")),
    }
}
