use nom::{Err, IResult};
use nom::error::{VerboseError, VerboseErrorKind};

pub type Res<T, U> = IResult<T, U, VerboseError<T>>;

pub fn to_err<'a>(rest: &'a [u8], error: &'static str) -> Err<VerboseError<&'a [u8]>> {
    Err::Failure(VerboseError {
        errors: vec![(
            rest,
            VerboseErrorKind::Context(error),
        )],
    })
}
