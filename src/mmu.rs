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
            result += (self.mem[i+p] as u64) << (8*n);
        }
        result
    }
}

