use unicorn_engine::{
    unicorn_const::{uc_error, Arch, Mode},
    Unicorn,
};

/// An environment for running CPU instructions in. Think of this as your "virtual CPU".
pub struct Environment<'a> {
    unicorn: Unicorn<'a, ()>,
}

impl<'a> Environment<'a> {
    /// Creates a new environment for the given architecture/mode. You probably want [Arch::ARM] and
    /// [Mode::LITTLE_ENDIAN].
    pub fn new(arch: Arch, mode: Mode) -> Result<Environment<'a>, uc_error> {
        let unicorn = Unicorn::new(arch, mode)?;

        Ok(Environment { unicorn })
    }
}
