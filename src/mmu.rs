pub struct Mmu {
    mem: Vec<u8>,
}

impl Mmu{
    pub fn new(mem: Vec<u8>) -> Mmu {
        Mmu{mem}
    }
    pub fn read_nbytes(&self, p: u64, n: u64) -> u64 {
        let p :usize = p as usize;
        let n :usize = n as usize;
        let mut result = 0_u64;
        for i in 0..n {
            result += (self.mem[i+p] as u64) << (8*i);
        }
        result
    }
    pub fn write_byte(&mut self, p: u64, data: u8){
        let p :usize = p as usize;
        self.mem[p] = data;
    }
    pub fn write_2byte(&mut self, p: u64, data: u16){
        for i in 0..2 {
            self.mem[i+p as usize] = (data>>(i*8)) as u8;
        }
    }
    pub fn write_4byte(&mut self, p: u64, data: u32){
        for i in 0..4 {
            self.mem[i+p as usize] = (data>>(i*8)) as u8;
        }
    }
}

