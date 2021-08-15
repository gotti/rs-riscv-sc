pub const LIMIT_CSR: usize = 4096;
pub struct Csr {
    register: [u64; LIMIT_CSR],
}
impl Csr {
    pub fn new(register: [u64; LIMIT_CSR]) -> Csr {
        Csr {
            register,
        }
    }
    pub fn write(&mut self, address: usize, data: u64) -> Result<(), String> {
        if address < 4096 {
            self.register[address] = data;
            Ok(())
        } else {
            Err(String::from("referring to out-of-range address"))
        }
    }
    pub fn read(&mut self, address: usize) -> Result<u64, String> {
        if address < 4096 {
            return Ok(self.register[address]);
        } else {
            Err(String::from("referring to out-of-range address"))
        }
    }
}
