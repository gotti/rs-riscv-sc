pub struct ShadowStack {
    sp: usize,
    stack: [u64; 255],
}

impl ShadowStack {
    pub fn new(sp: usize, stack: [u64; 255]) -> Self {
        Self { sp, stack }
    }
    pub fn push(&mut self, data: u64) -> Result<(),String> {
        self.stack[self.sp] = data;
        self.sp += 4;
        return Ok(())
    }
    pub fn pop(&mut self) -> Result<u64, String>{
        let ret = self.stack[self.sp];
        if self.sp <= 3{
            return Err(String::from("Error, shadowstack stack pointer will be under zero"));
        }
        self.sp -= 4;
        return Ok(ret)
    }
}
