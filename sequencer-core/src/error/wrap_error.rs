pub trait WrapError {
    type Output;

    fn wrap() -> Self::Output;
}
