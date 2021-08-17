pub struct Register {
    registers: [u64; 32],
}

impl Register {
    pub fn new(registers: [u64; 32]) -> Register {
        Register { registers }
    }
    pub fn read(&self, n: usize, len: u8) -> Result<u64, String> {
        if n >= 32 {
            return Err(String::from("Too large register number"));
        }
        if n==0 {
            return Ok(0);
        }
        match len {
            1 => return Ok(self.registers[n] as u8 as u64),
            2 => return Ok(self.registers[n] as u16 as u64),
            4 => return Ok(self.registers[n] as u32 as u64),
            8 => return Ok(self.registers[n] as u64 as u64),
            _ => {
                return Err(String::from("No such length"));
            }
        }
    }
    pub fn write(&mut self, n: usize, d: u64, len: u8) -> Result<(),String>{
        if n >= 32 {
            return Err(String::from("Too large register number"));
        }
        match len {
            1 => self.registers[n] = d as u8 as u64,
            2 => self.registers[n] = d as u16 as u64,
            3 => self.registers[n] = d as u32 as u64,
            4 => self.registers[n] = d as u64 as u64,
            _ => {
                return Err(String::from("No such length"));
            }
        }
        return Ok(());
    }
}
