use crate::mmu::Mmu;
use std::io;

const OP_LUDI: u32 = 0b01101;
const OP_AUIPC: u32 = 0b00101;
const OP_JAL: u32 = 0b11011;
const OP_JALR: u32 = 0b11001;
const OP_BRANCH: u32 = 0b11000;
const OP_LD: u32 = 0b00000;
const OP_STORE: u32 = 0b01000;
const OP_AIMM: u32 = 0b00100;
const OP_AREG: u32 = 0b01100;

pub struct Cpu {
    pc: u64,
    next_pc: u64,
    register: [u64; 32],
    mmu: Mmu,
}

impl Cpu {
    pub fn new(pc: u64, register: [u64; 32], mmu: Mmu) -> Cpu {
        Cpu {
            pc,
            next_pc: pc + 4,
            register,
            mmu,
        }
    }
    pub fn execute(&mut self) -> io::Result<()> {
        let inst = self.fetch();
        match self.exec(inst) {
            Ok(()) => {}
            Err(()) => {}
        }
        Ok(())
    }

    fn fetch(&mut self) -> u64 {
        self.pc = self.next_pc;
        let inst = self.mmu.read_nbytes(self.pc, 4);
        let op_length = parse_inst_length(inst);
        self.next_pc += op_length;
        inst
    }
    fn exec(&mut self, inst: u64) -> Result<(), ()> {
        let op_length = parse_inst_length(inst);
        match op_length {
            2 => {
                //TODO: implement compressed op
                Ok(())
            }
            4 => {
                match inst & 0xffffffff {
                    0 => {}
                    _ => {}
                }
                Ok(())
            }
            8 => Ok(()),
            _ => Err(()),
        }
    }
    fn exec_rv32(&mut self, inst: u32) -> Result<(), ()> {
        match rv32::get_op(inst) {
            OP_LUDI => {
                self.register[rv32::get_rd(inst)] =
                    (rv32::get_bits_extended(inst, 31, 12) << 12) as u64;
            }
            OP_AUIPC => {
                self.register[rv32::get_rd(inst)] =
                    self.pc + rv32::get_bits_extended(inst, 31, 12) as u64;
            }
            OP_JAL => {
                self.register[rv32::get_rd(inst)] = self.pc + 4;
                self.pc += rv32::get_imm_jal(inst) as u64;
            }
            OP_JALR => {
                self.register[rv32::get_rd(inst)] = self.pc + 4;
                self.pc = (self.register[rv32::get_rs1(inst) as usize] + rv32::get_bits_extended(inst, 31, 20) as u64 ) &0xfffffffe;
            }
            OP_BRANCH => {
            }
            _ => {}
        }
        return Err(());
    }
}

fn parse_inst_length(inst: u64) -> u64 {
    if inst & ((1 << 7) - 1) == 0b0111111 {
        //64bit
        8
    } else if inst & ((1 << 2) - 1) == 0b11 {
        //32bit
        4
    } else {
        //16bit
        2
    }
}

mod rv32 {
    pub fn get_bits(inst: u32, msb: usize, lsb: usize) -> u32 {
        (inst >> lsb) & ((1 << (msb - lsb + 1)) - 1)
    }
    #[test]
    fn test_get_bits() {
        let t: u32 = 0b01111000;
        assert!(get_bits(t, 6, 3) == 0b1111);
        assert!(get_bits(t, 3, 0) == 0b1000);
    }
    pub fn get_bits_extended(inst: u32, msb: usize, lsb: usize) -> u32 {
        let bits = get_bits(inst, msb, lsb);
        sign_extend(bits, (msb - lsb) as u32)
    }
    pub fn get_op(inst: u32) -> u32 {
        get_bits(inst, 6, 2)
    }
    pub fn get_funct3(inst: u32) -> u32 {
        get_bits(inst, 14, 12)
    }
    pub fn get_rd(inst: u32) -> usize {
        get_bits(inst, 11, 7) as usize
    }
    pub fn get_rs1(inst: u32) -> u32 {
        get_bits(inst, 19, 15)
    }
    pub fn get_rs2(inst: u32) -> u32 {
        get_bits(inst, 24, 20)
    }
    pub fn get_imm_jal(inst: u32) -> u32 {
        let imm = (get_bits(inst, 31, 31) << 20)
            + (get_bits(inst, 19, 12) << 12)
            + (get_bits(inst, 20, 20) << 11)
            + (get_bits(inst, 30, 21) << 1);
        sign_extend(imm, 20)
    }
    #[test]
    fn test_get_imm_jal() {
        let b: u32 = 0xffffffff;
        println!("{:b}", get_imm_jal(b));
        assert!(get_imm_jal(b) == 0xfffffffe);
        let b: u32 = 0x7affffff;
        assert!(get_imm_jal(b) == 0b011111111111110101110);
    }

    pub fn sign_extend(data: u32, msb: u32) -> u32 {
        if ((data >> msb) & 1) == 1 {
            (((1 << (31 - msb)) - 1) << (msb + 1)) + data
        } else {
            data
        }
    }
    #[test]
    fn test_sign_extend() {
        let b: u32 = 0b11111111;
        assert!(sign_extend(b, 7) == 0xffffffff);
        let b: u32 = 0b111111;
        assert!(sign_extend(b, 5) == 0xffffffff);
        let b: u32 = 0b11110000;
        assert!(sign_extend(b, 7) == 0xfffffff0);
    }
}
