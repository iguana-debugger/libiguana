use std::{
    io,
    process::{Child, Command, Stdio},
};

pub struct Environment {
    jimulator_process: Child,
}

impl Environment {
    pub fn new() -> Result<Self, io::Error> {
        let jimulator_process = Command::new("jimulator").stdin(Stdio::piped()).spawn()?;

        Ok(Self { jimulator_process })
    }
}

impl Drop for Environment {
    fn drop(&mut self) {
        // We should probably check for errors here?
        let _ = self.jimulator_process.kill();
    }
}
