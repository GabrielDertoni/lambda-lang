pub enum TryFold<T> {
    Continue(T),
    Done(T),
}

impl<T> std::ops::Try for TryFold<T> {
    type Ok = T;
    type Error = T;

    fn into_result(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Continue(val) => Ok(val),
            Self::Done(val) => Err(val),
        }
    }

    fn from_ok(v: Self::Ok) -> Self {
        Self::Continue(v)
    }

    fn from_error(v: Self::Error) -> Self {
        Self::Done(v)
    }
}
