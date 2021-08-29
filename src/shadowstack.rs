pub struct ShadowStack {
    sp: usize,
    stack: [u64; 255],
}

impl ShadowStack {
    pub fn new(sp: usize, stack: [u64; 255]) -> Self {
        Self { sp, stack }
    }
    pub fn push(&mut self, data) -> Result<(),String> {
        self.stack[self.sp] = data;
        self.sp += 4;
        return Ok(())
    }
    pub fn pop(&mut self, data) -> Result<u64, String>{
        let ret = self.stack[self.sp];
        if sp <= 0{
            return Err(String::from("Error, shadowstack stack pointer will be under zero"));
        }
        sp -= 4;
        return ret
    }
}
