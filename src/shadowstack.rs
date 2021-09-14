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
        self.sp += 1;
        return Ok(())
    }
    pub fn pop(&mut self) -> Result<u64, String>{
        if self.sp < 1{
            return Err(String::from("Error, shadowstack stack pointer will be under zero"));
        }
        let ret = self.stack[self.sp-1];
        self.sp -= 1;
        return Ok(ret)
    }
    fn get_sp(&self) -> usize{
        self.sp
    }
    fn get_stack(&self, i: usize) -> u64{
        self.stack[i]
    }
}
#[test]
fn push(){
    let mut sstack = ShadowStack::new(0,[0;255]);
    assert!(sstack.get_sp()==0);
    assert!(sstack.get_stack(0)==0);
    sstack.push(1);
    assert!(sstack.get_sp()==1);
    assert!(sstack.get_stack(0)==1);
}


#[test]
fn pushandpop(){
    let mut sstack = ShadowStack::new(0,[0;255]);
    sstack.push(1);
    match sstack.pop() {
        Ok(r) => {
            println!("{}",r);
            assert!(r==1);
        },
        Err(r) => panic!(r),
    }
}
