use unicorn_engine::{unicorn_const::uc_error, Unicorn};

pub trait MemoryExtensions {
    fn read_string(&self, address: u64) -> Result<Vec<u8>, uc_error>;
}

impl MemoryExtensions for Unicorn<'_, ()> {
    /// Reads memory from `address` to `address` + `size`, flipping each word in the process since
    /// Unicorn reads the wrong way around for some reason
    fn read_string(&self, address: u64) -> Result<Vec<u8>, uc_error> {
        // Ok(self
        //     .mem_read_as_vec(address, size)?
        //     .chunks(4)
        //     .map(|word| word.into_iter().rev())
        //     .flatten()
        //     .map(|borrow| *borrow)
        //     .collect())

        let mut string = vec![];
        let mut offset = 0;

        loop {
            let mut byte_buf = [0];

            self.mem_read(address + offset, &mut byte_buf)?;

            if byte_buf[0] == 0 {
                return Ok(string);
            } else {
                string.push(byte_buf[0]);
                offset += 1;
            }
        }
    }
}
