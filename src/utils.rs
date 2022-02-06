use nom::IResult;
use nom::error::VerboseError;

pub type Res<T, U> = IResult<T, U, VerboseError<T>>;
