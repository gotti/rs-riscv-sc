use std::ops;

#[derive(Clone, Copy)]
pub struct Bits {
    data: u64,
    len: usize,
}
#[macro_export]
macro_rules! bitcat {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_bits = Bits::blank();
            $(
                temp_bits = temp_bits.shiftadd($x);
            )*
            temp_bits
        }
    };
}
impl Bits {
    pub fn blank() -> Self {
        return Self { data: 0, len: 0 };
    }
    pub fn new(data: u64, len: usize) -> Self {
        return Self { data, len };
    }
    pub fn shiftadd(&mut self, s: Self) -> Self {
        Self{data:(self.data<<s.len)+s.data,len:self.len+s.len}
    }
    pub fn cut_new(data:u32, msb: usize, lsb: usize) -> Self{
        Bits{data: ((data >> lsb) & ((1 << (msb - lsb + 1)) - 1)) as u64, len: msb-lsb+1}
    }
    pub fn cut(&self, msb: usize, lsb: usize) -> Self{
        Bits{data: ((self.data >> lsb) & ((1 << (msb - lsb + 1)) - 1)) as u64, len: msb-lsb+1}
    }
    pub fn to_u32(&self) -> u32{
        self.data as u32
    }
    pub fn extend(&self) -> u32{
        let msb = self.len -1;
        if ((self.data >> msb) & 1) == 1 {
            ((((1 << (31 - msb)) - 1) << (msb + 1)) + self.data) as u32
        } else {
            self.data as u32
        }
    }
    pub fn expand(&self, len: usize) -> Self{
        Self{data: self.data, len}
    }
}

#[test]
fn test_new(){
    let b = Bits::new(7,3);
    assert!(b.to_u32()==7);
}
#[test]
fn test_shiftadd(){
    let mut b = Bits::new(7, 3);
    b = b.shiftadd(Bits::new(7,3));
    println!("{:x}",b.to_u32());
    assert!(b.to_u32()==63);
}

impl ops::Add<Bits> for Bits {
    type Output = Bits;
    fn add(self, _rhs: Bits) -> Bits {
        let len = if self.len > _rhs.len {
            self.len
        } else {
            _rhs.len
        };
        let data = (self.data + _rhs.data) & (1 << len - 1);
        return Bits { data, len };
    }
}
