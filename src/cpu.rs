use crate::mmu::Mmu;
use std::io;

mod op {
    pub const LUDI: u32 = 0b01101;
    pub const AUIPC: u32 = 0b00101;
    pub const JAL: u32 = 0b11011;
    pub const JALR: u32 = 0b11001;
    pub const BRANCH: u32 = 0b11000;
    pub const LD: u32 = 0b00000;
    pub const STORE: u32 = 0b01000;
    pub const AIMM: u32 = 0b00100;
    pub const AREG: u32 = 0b01100;
    pub const CSR: u32 = 0b11100;
}

//funct3 for branch
mod f3b {
    pub const BEQ: u32 = 0b000;
    pub const BNE: u32 = 0b001;
    pub const BLT: u32 = 0b100;
    pub const BGE: u32 = 0b101;
    pub const BLTU: u32 = 0b110;
    pub const BGEU: u32 = 0b111;
}

//funct3 for load
mod f3l {
    pub const LB: u32 = 0b000;
    pub const LH: u32 = 0b001;
    pub const LW: u32 = 0b010;
    pub const LBU: u32 = 0b100;
    pub const LHU: u32 = 0b101;
}

//funct3 for store
mod f3s {
    pub const SB: u32 = 0b000;
    pub const SH: u32 = 0b001;
    pub const SW: u32 = 0b010;
}

//funct3 for arithmatic immediate
mod f3i {
    pub const ADDI: u32 = 0b000;
    pub const SLTI: u32 = 0b010;
    pub const SLTIU: u32 = 0b011;
    pub const XORI: u32 = 0b100;
    pub const ORI: u32 = 0b110;
    pub const ANDI: u32 = 0b111;
}

//funct3 for arithmatic register
mod f3r {
    pub const ADD_SUB: u32 = 0b000;
    pub const SLL: u32 = 0b001;
    pub const SLT: u32 = 0b010;
    pub const SLTU: u32 = 0b011;
    pub const XOR: u32 = 0b100;
    pub const SRL_SRA: u32 = 0b101;
    pub const OR: u32 = 0b110;
    pub const AND: u32 = 0b111;
}

mod f3c {
    pub const CSRRW: u32 = 0b001;
    pub const CSRRS: u32 = 0b010;
    pub const CSRRC: u32 = 0b011;
    pub const CSRRWI: u32 = 0b101;
    pub const CSRRSI: u32 = 0b110;
    pub const CSRRCI: u32 = 0b111;
}

pub struct Cpu {
    pc: u64,
    csr: [u64; 4096],
    register: [u64; 32],
    mmu: Mmu,
}

impl Cpu {
    pub fn new(pc: u64, csr: [u64; 4096], register: [u64; 32], mmu: Mmu) -> Cpu {
        Cpu {
            pc,
            csr,
            register,
            mmu,
        }
    }
    pub fn execute(&mut self) -> io::Result<()> {
        loop {
            let old_pc = self.pc;
            let (inst, op_len) = self.fetch();
            match self.exec(inst) {
                Ok(()) => {}
                Err(()) => {
                    println!("Error");
                }
            }
            if old_pc == self.pc {
                self.pc += op_len;
            }
        }
        Ok(())
    }

    fn fetch(&mut self) -> (u64, u64) {
        let inst = self.mmu.read_nbytes(self.pc, 4);
        let op_length = parse_inst_length(inst);
        println!("inst: {:#x}", inst);
        println!("pc  : {:#x}", self.pc);
        (inst, op_length)
    }
    fn exec(&mut self, inst: u64) -> Result<(), ()> {
        let op_length = parse_inst_length(inst);
        match op_length {
            2 => {
                //TODO: implement compressed op
                Err(())
            }
            4 => match self.exec_rv32(inst as u32) {
                Ok(()) => Ok(()),
                Err(()) => Err(()),
            },
            8 => Ok(()),
            _ => Err(()),
        }
    }
    fn exec_rv32(&mut self, inst: u32) -> Result<(), ()> {
        println!("{:x}", rv32::get_op(inst));
        match rv32::get_op(inst) {
            op::LUDI => {
                self.register[rv32::get_rd(inst)] =
                    (rv32::get_bits_extended(inst, 31, 12) << 12) as u64;
            }
            op::AUIPC => {
                self.register[rv32::get_rd(inst)] =
                    self.pc + rv32::get_bits_extended(inst, 31, 12) as u64;
            }
            op::JAL => {
                self.register[rv32::get_rd(inst)] = self.pc + 4;
                self.pc += rv32::get_imm_jal(inst) as u64;
                println!(
                    "JAL x{:x}, 0x{:x}",
                    rv32::get_rd(inst),
                    rv32::get_imm_jal(inst)
                )
            }
            op::JALR => {
                self.register[rv32::get_rd(inst)] = self.pc + 4;
                self.pc = (self.register[rv32::get_rs1(inst)]
                    + rv32::get_bits_extended(inst, 31, 20) as u64)
                    & 0xfffffffe;
            }
            op::BRANCH => match rv32::get_funct3(inst) {
                f3b::BEQ => {
                    if self.register[rv32::get_rs1(inst)] == self.register[rv32::get_rs2(inst)] {
                        self.pc += rv32::sign_extend(rv32::get_imm_branch(inst), 12) as u64;
                    }
                }
                f3b::BNE => {
                    if self.register[rv32::get_rs1(inst)] != self.register[rv32::get_rs2(inst)] {
                        self.pc += rv32::sign_extend(rv32::get_imm_branch(inst), 12) as u64;
                    }
                }
                f3b::BLT => {
                    if (self.register[rv32::get_rs1(inst)] as i32)
                        < (self.register[rv32::get_rs2(inst)] as i32)
                    {
                        self.pc += rv32::sign_extend(rv32::get_imm_branch(inst), 12) as u64;
                    }
                }
                f3b::BGE => {
                    if (self.register[rv32::get_rs1(inst)] as i32)
                        >= (self.register[rv32::get_rs2(inst)] as i32)
                    {
                        self.pc += rv32::sign_extend(rv32::get_imm_branch(inst), 12) as u64;
                    }
                }
                f3b::BLTU => {
                    if self.register[rv32::get_rs1(inst)] < self.register[rv32::get_rs2(inst)] {
                        self.pc += rv32::sign_extend(rv32::get_imm_branch(inst), 12) as u64;
                    }
                }
                f3b::BGEU => {
                    if self.register[rv32::get_rs1(inst)] >= self.register[rv32::get_rs2(inst)] {
                        self.pc += rv32::sign_extend(rv32::get_imm_branch(inst), 12) as u64;
                    }
                }
                _ => {
                    println!("not found");
                    return Err(());
                }
            },
            op::LD => match rv32::get_funct3(inst) {
                f3l::LB => {
                    let offset = rv32::get_bits_extended(inst, 31, 20);
                    let address = (self.register[rv32::get_rs1(inst)] as u32) + offset;
                    let data = rv32::sign_extend(self.mmu.read_nbytes(address as u64, 1) as u32, 7);
                    self.register[rv32::get_rs1(inst)] = data as u64;
                }
                f3l::LH => {
                    let offset = rv32::get_bits_extended(inst, 31, 20);
                    let address = (self.register[rv32::get_rs1(inst)] as u32) + offset;
                    let data =
                        rv32::sign_extend(self.mmu.read_nbytes(address as u64, 2) as u32, 15);
                    self.register[rv32::get_rs1(inst)] = data as u64;
                }
                f3l::LW => {
                    let offset = rv32::get_bits_extended(inst, 31, 20);
                    let address = (self.register[rv32::get_rs1(inst)] as u32) + offset;
                    let data =
                        rv32::sign_extend(self.mmu.read_nbytes(address as u64, 2) as u32, 31);
                    self.register[rv32::get_rs1(inst)] = data as u64;
                }
                f3l::LBU => {
                    let offset = rv32::get_bits_extended(inst, 31, 20);
                    let address = (self.register[rv32::get_rs1(inst)] as u32) + offset;
                    let data = self.mmu.read_nbytes(address as u64, 1) as u32;
                    self.register[rv32::get_rs1(inst)] = data as u64;
                }
                f3l::LHU => {
                    let offset = rv32::get_bits_extended(inst, 31, 20);
                    let address = (self.register[rv32::get_rs1(inst)] as u32) + offset;
                    let data = self.mmu.read_nbytes(address as u64, 2) as u32;
                    self.register[rv32::get_rs1(inst)] = data as u64;
                }
                _ => {
                    println!("not found");
                    return Err(());
                }
            },
            op::STORE => match rv32::get_funct3(inst) {
                f3s::SB => {
                    self.mmu.write_byte(
                        self.register[rv32::get_rs1(inst)]
                            + rv32::sign_extend(rv32::get_imm_st(inst), 11) as u64,
                        self.register[rv32::get_rs2(inst)] as u8,
                    );
                }
                f3s::SH => {
                    self.mmu.write_2byte(
                        self.register[rv32::get_rs1(inst)]
                            + rv32::sign_extend(rv32::get_imm_st(inst), 11) as u64,
                        self.register[rv32::get_rs2(inst)] as u16,
                    );
                }
                f3s::SW => {
                    self.mmu.write_4byte(
                        self.register[rv32::get_rs1(inst)]
                            + rv32::sign_extend(rv32::get_imm_st(inst), 11) as u64,
                        self.register[rv32::get_rs2(inst)] as u32,
                    );
                }
                _ => {
                    return Err(());
                }
            },
            op::AIMM => match rv32::get_funct3(inst) {
                f3i::ADDI => {
                    // TODO: 32bitと64bit
                    println!(
                        "ADDI x{}, x{}, 0x{:x}",
                        rv32::get_rd(inst),
                        rv32::get_rs1(inst),
                        rv32::get_bits_extended(inst, 31, 20)
                    );
                    self.register[rv32::get_rd(inst)] = self.register[rv32::get_rs1(inst)]
                        + rv32::get_bits_extended(inst, 31, 20) as u64;
                }
                f3i::SLTI => {
                    // TODO: 32bitと64bit
                    self.register[rv32::get_rd(inst)] = if self.register[rv32::get_rs1(inst)]
                        < rv32::get_bits_extended(inst, 31, 20) as u64
                    {
                        1
                    } else {
                        0
                    };
                }
                f3i::SLTIU => {
                    self.register[rv32::get_rd(inst)] = if self.register[rv32::get_rs1(inst)]
                        < rv32::get_bits(inst, 31, 20) as u64
                    {
                        1
                    } else {
                        0
                    };
                }
                f3i::XORI => {
                    self.register[rv32::get_rd(inst)] =
                        ((self.register[rv32::get_rs1(inst)] as u32)
                            ^ (rv32::get_bits(inst, 31, 20))) as u64;
                }
                f3i::ORI => {
                    self.register[rv32::get_rd(inst)] =
                        ((self.register[rv32::get_rs1(inst)] as u32)
                            | (rv32::get_bits(inst, 31, 20))) as u64;
                }
                f3i::ANDI => {
                    self.register[rv32::get_rd(inst)] =
                        ((self.register[rv32::get_rs1(inst)] as u32)
                            & (rv32::get_bits(inst, 31, 20))) as u64;
                }
                _ => {
                    println!("not found");
                    return Err(());
                }
            },
            op::AREG => match rv32::get_funct3(inst) {
                f3r::ADD_SUB => {}
                _ => {
                    println!("not found");
                    return Err(());
                }
            },
            op::CSR => match rv32::get_funct3(inst) {
                f3c::CSRRW => {
                    let csr = rv32::get_bits(inst, 31, 20) as usize;
                    let t = self.csr[csr];
                    self.csr[csr] = self.register[rv32::get_rs1(inst)];
                    self.register[rv32::get_rd(inst)] = t;
                }
                f3c::CSRRS => {
                    let csr = rv32::get_bits(inst, 31, 20) as usize;
                    let t = self.csr[csr];
                    self.csr[csr] = t | self.register[rv32::get_rs1(inst)];
                    self.register[rv32::get_rd(inst)] = t;
                }
                f3c::CSRRC => {
                    let csr = rv32::get_bits(inst, 31, 20) as usize;
                    let t = self.csr[csr];
                    self.csr[csr] = t & (!self.register[rv32::get_rs1(inst)]);
                    self.register[rv32::get_rd(inst)] = t;
                }
                f3c::CSRRWI => {
                    let csr = rv32::get_bits(inst, 31, 20) as usize;
                    self.register[rv32::get_rd(inst)] = self.csr[csr];
                    self.csr[csr] = rv32::get_bits(inst, 19,15) as u64;
                }
                f3c::CSRRSI => {
                    let csr = rv32::get_bits(inst, 31, 20) as usize;
                    let t = self.csr[csr];
                    self.csr[csr] = t | (rv32::get_bits(inst, 19, 15) as u64);
                    self.register[rv32::get_rd(inst)] = t;
                }
                f3c::CSRRCI => {
                    let csr = rv32::get_bits(inst, 31, 20) as usize;
                    let t = self.csr[csr];
                    self.csr[csr] = t & (!(rv32::get_bits(inst, 19, 15) as u64));
                    self.register[rv32::get_rd(inst)] = t;
                }
                _ => {
                    println!("not found");
                    return Err(());
                }
            },
            _ => {
                println!("not found op");
                return Err(());
            }
        }
        return Ok(());
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
    pub fn get_rs1(inst: u32) -> usize {
        get_bits(inst, 19, 15) as usize
    }
    pub fn get_rs2(inst: u32) -> usize {
        get_bits(inst, 24, 20) as usize
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

    pub fn get_imm_branch(inst: u32) -> u32 {
        let imm = (get_bits(inst, 31, 31) << 12)
            + (get_bits(inst, 30, 25) << 5)
            + (get_bits(inst, 7, 7) << 11)
            + (get_bits(inst, 11, 8) << 1);
        return imm;
    }
    pub fn get_imm_ld(inst: u32) -> u32 {
        return get_bits(inst, 31, 20);
    }
    pub fn get_imm_st(inst: u32) -> u32 {
        return (get_bits(inst, 31, 25) << 5) + get_bits(inst, 11, 7);
    }
}
