use crate::{
    bitcat,
    bitutils::{self, Bits},
    cpu::rv32::get_bits,
    csr::Csr,
    mmu::Mmu,
    register::Register,
    shadowstack::ShadowStack,
};
use std::{io, str::FromStr};

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
    pub const FENCE: u32 = 0b00011;
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
    pub const SLLI: u32 = 0b001;
    pub const SRLI_SRAI: u32 = 0b101;
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
    pub const EXCEPT: u32 = 0b000;
    pub const CSRRW: u32 = 0b001;
    pub const CSRRS: u32 = 0b010;
    pub const CSRRC: u32 = 0b011;
    pub const CSRRWI: u32 = 0b101;
    pub const CSRRSI: u32 = 0b110;
    pub const CSRRCI: u32 = 0b111;
}

mod privilege {
    pub const user: u8 = 0b00;
    pub const supervisor: u8 = 0b01;
    pub const hypervisor: u8 = 0b10;
    pub const machine: u8 = 0b11;
}

mod exception {
    pub const ecall: u32 = 0;
    pub const mret: u32 = 0b001100000010;
}

mod f3co_2 {
    pub const slli: u32 = 0b000;
    pub const jal: u32 = 0b001;
    pub const lwsp: u32 = 0b010;
    pub const mv: u32 = 0b100;
    pub const ldsp: u32 = 0b011;
    pub const ja: u32 = 0b100;
    pub const swsp: u32 = 0b110;
    pub const sdsp: u32 = 0b111;
}

pub trait R2R {
    fn exec_register(&self, state: State, reg: &Register) -> Result<Register, String>;
}

struct RegisterInst {
    exec: fn(&Self, State, &Register) -> Result<Register, String>,
}
impl R2R for RegisterInst {
    fn exec_register(&self, state: State, reg: &Register) -> Result<Register, String> {
        (self.exec)(&self, state, reg)
    }
}
const add: RegisterInst = RegisterInst {
    exec: |sel, state, reg| {
        let mut r = reg.clone();
        if rv32::get_bits(state.imm as u32, 10, 10) != 1 {
            r.write(
                state.rd,
                reg.read(state.rs1, 4)? + reg.read(state.rs2, 4)?,
                4,
            )?;
        } else {
            r.write(
                state.rd,
                reg.read(state.rs1, 4)? - reg.read(state.rs2, 4)?,
                4,
            )?;
        }
        Ok(r)
    },
};
pub struct State {
    rd: usize,
    rs1: usize,
    rs2: usize,
    imm: u64,
}
impl State {
    pub fn new(rd: usize, rs1: usize, rs2: usize, imm: u64) -> Self {
        Self { rd, rs1, rs2, imm }
    }
    pub fn read_rd(&self) -> usize {
        self.rd
    }
    pub fn read_rs1(&self) -> usize {
        self.rs1
    }
    pub fn read_rs2(&self) -> usize {
        self.rs2
    }
    pub fn read_rs3(&self) -> u64 {
        self.imm
    }
}

pub struct Cpu {
    pc: u64,
    len: u8,
    csr: Csr,
    register: Register,
    privilege: u8,
    mmu: Mmu,
    sstack: ShadowStack,
}

impl Cpu {
    pub fn new(
        pc: u64,
        csr: Csr,
        register: Register,
        privilege: u8,
        mmu: Mmu,
        sstack: ShadowStack,
    ) -> Cpu {
        Cpu {
            pc,
            len: 4,
            csr,
            register,
            privilege,
            mmu,
            sstack,
        }
    }
    pub fn execute(&mut self) -> io::Result<()> {
        loop {
            let old_pc = self.pc;
            let (inst, op_len) = self.fetch();
            let mut backup_register: [u64; 32] = [0; 32];
            for i in 0..32 {
                backup_register[i] = match self.register.read(i, self.len) {
                    Ok(t) => t,
                    Err(s) => {
                        println!("{:?}", s);
                        0
                    }
                };
            }
            match self.exec(inst) {
                Ok(()) => {}
                Err(e) => {
                    println!("{}", e);
                    break;
                }
            }
            for i in 0..32 {
                let k = match self.register.read(i, self.len) {
                    Ok(t) => t,
                    Err(s) => {
                        println!("{:?}", s);
                        0
                    }
                };
                if k != backup_register[i] {
                    print!("\x1b[31m{:x?}\x1b[0m\t", k);
                } else {
                    print!("{:x?}\t", k);
                }
                if i % 8 == 3 {
                    println!("");
                }
            }
            if old_pc == self.pc {
                self.pc += op_len;
            }
        }
        Ok(())
    }

    fn fetch(&mut self) -> (u64, u64) {
        let inst = match self.len {
            4 => self.mmu.read_nbytes(self.pc as u32 as u64, 4),
            _ => 999999999999,
        };
        let op_length = parse_inst_length(inst);
        println!("inst: {:#x}", inst);
        println!("pc  : {:#x}", self.pc);
        (inst, op_length)
    }
    fn exec(&mut self, inst: u64) -> Result<(), String> {
        let op_length = parse_inst_length(inst);
        match op_length {
            2 => {
                //TODO: implement compressed op
                Err(String::from("Not implemented compressed op"))
            }
            4 => self.exec_rv32(inst as u32),
            8 => Err(String::from("Not implemented rv64")),
            _ => Err(String::from("Invalid xlen")),
        }
    }
    fn uncompress(inst: u32) -> Result<u32, String> {
        match rv32::get_bits(inst, 1, 0) {
            0 => {}
            1 => {}
            2 => match rv32::get_bits(inst, 15, 13) {
                f3co_2::slli => {
                    let rd = bitutils::Bits::cut_new(inst, 11, 7);
                    let shamt = bitutils::Bits::cut_new(inst, 12, 12)
                        .shiftadd(bitutils::Bits::cut_new(inst, 6, 2));
                    let bits = bitcat!(
                        shamt,
                        rd,
                        Bits::new(0b001, 3),
                        rd,
                        Bits::new(op::AIMM as u64, 5),
                        Bits::new(0b11, 2)
                    );
                    return Ok(bits.to_u32());
                }
                f3co_2::jal => {
                    let imm = Bits::new(
                        bitcat!(
                            Bits::cut_new(inst, 12, 12),
                            Bits::cut_new(inst, 8, 8),
                            Bits::cut_new(inst, 10, 9),
                            Bits::cut_new(inst, 6, 6),
                            Bits::cut_new(inst, 7, 7),
                            Bits::cut_new(inst, 2, 2),
                            Bits::cut_new(inst, 11, 11),
                            Bits::cut_new(inst, 5, 3)
                        )
                        .extend() as u64,
                        32,
                    );
                    let ret = bitcat!(
                        imm.cut(20, 20),
                        imm.cut(10, 1),
                        imm.cut(11,11),
                        imm.cut(19, 12),
                        Bits::new(1, 5),
                        Bits::new(op::JAL as u64, 5),
                        Bits::new(0b11, 2)
                    );
                    return Ok(ret.to_u32())
                }
                f3co_2::lwsp => {
                    let rd = bitutils::Bits::cut_new(inst, 11, 7);
                    let imm = bitcat!(
                        Bits::cut_new(inst, 3, 2),
                        Bits::cut_new(inst, 12, 12),
                        Bits::cut_new(inst, 6, 5),
                        Bits::new(0, 2)
                    );
                    let ret = bitcat!(
                        imm,
                        Bits::new(2, 5),
                        Bits::new(2, 3),
                        rd,
                        Bits::new(op::LD as u64, 5),
                        Bits::new(0b11, 2)
                    );
                    return Ok(ret.to_u32());
                }
                f3co_2::mv => match rv32::get_bits(inst, 12, 12) {
                    0 => {
                        let rd = Bits::cut_new(inst, 11, 7);
                        let rs2 = Bits::cut_new(inst, 6, 2);
                        let ret = bitcat!(
                            rs2,
                            Bits::new(0, 5),
                            Bits::new(f3r::ADD_SUB as u64, 3),
                            rd,
                            Bits::new(op::AREG as u64, 5),
                            Bits::new(0b11, 2)
                        );
                        return Ok(ret.to_u32());
                    }
                    _ => {
                        return Err(String::from("No such compressed op"));
                    }
                },
                f3co_2::ldsp => {
                    return Err(String::from("C.LDSP,RV64C is not implemeted."));
                },
                f3co_2::ja => {
                },
            },
            _ => {}
        }

        return Err(String::from("uncompress error"));
    }
    fn exec_rv32(&mut self, inst: u32) -> Result<(), String> {
        match rv32::get_op(inst) {
            op::LUDI => {
                self.register.write(
                    rv32::get_rd(inst),
                    (rv32::get_bits_extended(inst, 31, 12) << 12) as u64,
                    self.len,
                )?;
            }
            op::AUIPC => {
                self.register.write(
                    rv32::get_rd(inst),
                    self.pc + rv32::get_bits_extended(inst, 31, 12) as u64,
                    self.len,
                )?;
            }
            op::JAL => {
                self.register
                    .write(rv32::get_rd(inst), self.pc + 4, self.len)?;
                self.pc += rv32::get_imm_jal(inst) as u64;
                if rv32::get_rd(inst) == 1 {
                    //this is subroutine call
                    self.sstack.push(self.pc)?;
                }
                println!(
                    "JAL x{:x}, 0x{:x}",
                    rv32::get_rd(inst),
                    rv32::get_imm_jal(inst)
                )
            }
            op::JALR => {
                self.register
                    .write(rv32::get_rd(inst), self.pc + 4, self.len)?;
                let target = (self.register.read(rv32::get_rs1(inst), self.len)?
                    + rv32::get_bits_extended(inst, 31, 20) as u64)
                    & !1;
                if rv32::get_rd(inst) == 0
                    && rv32::get_rs1(inst) == 1
                    && rv32::get_bits(inst, 31, 20) == 0
                {
                    //ret
                    if target == self.sstack.pop()? {
                        self.pc = (self.register.read(rv32::get_rs1(inst), self.len)?
                            + rv32::get_bits_extended(inst, 31, 20) as u64)
                            & !1;
                    } else {
                        return Err(String::from("@@@ shadow stack mismatch @@@"));
                    }
                }
            }
            op::BRANCH => match rv32::get_funct3(inst) {
                f3b::BEQ => {
                    if self.register.read(rv32::get_rs1(inst), self.len)?
                        == self.register.read(rv32::get_rs2(inst), self.len)?
                    {
                        self.pc += rv32::sign_extend(rv32::get_imm_branch(inst), 12) as u64;
                    }
                }
                f3b::BNE => {
                    if self.register.read(rv32::get_rs1(inst), self.len)?
                        != self.register.read(rv32::get_rs2(inst), self.len)?
                    {
                        println!(
                            "bne_pc={0}",
                            rv32::sign_extend(rv32::get_imm_branch(inst), 12)
                        );
                        self.pc += rv32::sign_extend(rv32::get_imm_branch(inst), 12) as u64;
                    }
                }
                f3b::BLT => {
                    if (self.register.read(rv32::get_rs1(inst), self.len)? as i32)
                        < (self.register.read(rv32::get_rs2(inst), self.len)? as i32)
                    {
                        self.pc += rv32::sign_extend(rv32::get_imm_branch(inst), 12) as u64;
                    }
                }
                f3b::BGE => {
                    if (self.register.read(rv32::get_rs1(inst), self.len)? as i32)
                        >= (self.register.read(rv32::get_rs2(inst), self.len)? as i32)
                    {
                        self.pc += rv32::sign_extend(rv32::get_imm_branch(inst), 12) as u64;
                    }
                }
                f3b::BLTU => {
                    if self.register.read(rv32::get_rs1(inst), self.len)?
                        < self.register.read(rv32::get_rs2(inst), self.len)?
                    {
                        self.pc += rv32::sign_extend(rv32::get_imm_branch(inst), 12) as u64;
                    }
                }
                f3b::BGEU => {
                    if self.register.read(rv32::get_rs1(inst), self.len)?
                        >= self.register.read(rv32::get_rs2(inst), self.len)?
                    {
                        self.pc += rv32::sign_extend(rv32::get_imm_branch(inst), 12) as u64;
                    }
                }
                _ => {
                    return Err(String::from("No inst on branch"));
                }
            },
            op::LD => match rv32::get_funct3(inst) {
                f3l::LB => {
                    let offset = rv32::get_bits_extended(inst, 31, 20);
                    let address =
                        (self.register.read(rv32::get_rs1(inst), self.len)? as u32) + offset;
                    let data = rv32::sign_extend(self.mmu.read_nbytes(address as u64, 1) as u32, 7);
                    self.register
                        .write(rv32::get_rs1(inst), data as u64, self.len)?;
                }
                f3l::LH => {
                    let offset = rv32::get_bits_extended(inst, 31, 20);
                    let address =
                        (self.register.read(rv32::get_rs1(inst), self.len)? as u32) + offset;
                    let data =
                        rv32::sign_extend(self.mmu.read_nbytes(address as u64, 2) as u32, 15);
                    self.register
                        .write(rv32::get_rs1(inst), data as u64, self.len)?;
                }
                f3l::LW => {
                    let offset = rv32::get_bits_extended(inst, 31, 20);
                    let address =
                        (self.register.read(rv32::get_rs1(inst), self.len)? as u32) + offset;
                    let data =
                        rv32::sign_extend(self.mmu.read_nbytes(address as u64, 2) as u32, 31);
                    self.register
                        .write(rv32::get_rs1(inst), data as u64, self.len)?;
                }
                f3l::LBU => {
                    let offset = rv32::get_bits_extended(inst, 31, 20);
                    let address =
                        (self.register.read(rv32::get_rs1(inst), self.len)? as u32) + offset;
                    let data = self.mmu.read_nbytes(address as u64, 1) as u32;
                    self.register
                        .write(rv32::get_rs1(inst), data as u64, self.len)?;
                }
                f3l::LHU => {
                    let offset = rv32::get_bits_extended(inst, 31, 20);
                    let address =
                        (self.register.read(rv32::get_rs1(inst), self.len)? as u32) + offset;
                    let data = self.mmu.read_nbytes(address as u64, 2) as u32;
                    self.register
                        .write(rv32::get_rs1(inst), data as u64, self.len)?;
                }
                _ => {
                    return Err(String::from("No inst on load"));
                }
            },
            op::STORE => match rv32::get_funct3(inst) {
                f3s::SB => {
                    self.mmu.write_byte(
                        (self.register.read(rv32::get_rs1(inst), self.len)?
                            + rv32::sign_extend(rv32::get_imm_st(inst), 11) as u64)
                            as u32 as u64,
                        self.register.read(rv32::get_rs2(inst), self.len)? as u8,
                    );
                }
                f3s::SH => {
                    self.mmu.write_2byte(
                        (self.register.read(rv32::get_rs1(inst), self.len)?
                            + rv32::sign_extend(rv32::get_imm_st(inst), 11) as u64)
                            as u32 as u64,
                        self.register.read(rv32::get_rs2(inst), self.len)? as u16,
                    );
                }
                f3s::SW => {
                    self.mmu.write_4byte(
                        (self.register.read(rv32::get_rs1(inst), self.len)?
                            + rv32::sign_extend(rv32::get_imm_st(inst), 11) as u64)
                            as u32 as u64,
                        self.register.read(rv32::get_rs2(inst), self.len)? as u32,
                    );
                }
                _ => {
                    return Err(String::from("No inst on store"));
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
                    self.register.write(
                        rv32::get_rd(inst),
                        self.register.read(rv32::get_rs1(inst), self.len)?
                            + rv32::get_bits_extended(inst, 31, 20) as u64,
                        self.len,
                    )?;
                }
                f3i::SLTI => {
                    // TODO: 32bitと64bit
                    self.register.write(
                        rv32::get_rd(inst),
                        if self.register.read(rv32::get_rs1(inst), self.len)?
                            < rv32::get_bits_extended(inst, 31, 20) as u64
                        {
                            1
                        } else {
                            0
                        },
                        self.len,
                    )?;
                }
                f3i::SLTIU => {
                    self.register.write(
                        rv32::get_rd(inst),
                        if self.register.read(rv32::get_rs1(inst), self.len)?
                            < rv32::get_bits(inst, 31, 20) as u64
                        {
                            1
                        } else {
                            0
                        },
                        self.len,
                    )?;
                }
                f3i::XORI => {
                    self.register.write(
                        rv32::get_rd(inst),
                        ((self.register.read(rv32::get_rs1(inst), self.len)? as u32)
                            ^ (rv32::get_bits(inst, 31, 20))) as u64,
                        self.len,
                    )?;
                }
                f3i::ORI => {
                    self.register.write(
                        rv32::get_rd(inst),
                        ((self.register.read(rv32::get_rs1(inst), self.len)? as u32)
                            | (rv32::get_bits(inst, 31, 20))) as u64,
                        self.len,
                    )?;
                }
                f3i::ANDI => {
                    self.register.write(
                        rv32::get_rd(inst),
                        ((self.register.read(rv32::get_rs1(inst), self.len)? as u32)
                            & (rv32::get_bits(inst, 31, 20))) as u64,
                        self.len,
                    )?;
                }
                f3i::SLLI => {
                    self.register.write(
                        rv32::get_rd(inst),
                        ((self.register.read(rv32::get_rs1(inst), self.len)? as u32)
                            << (rv32::get_bits(inst, 31, 20))) as u64,
                        self.len,
                    )?;
                }
                f3i::SRLI_SRAI => match rv32::get_bits(inst, 30, 30) {
                    0 => {
                        self.register.write(
                            rv32::get_rd(inst),
                            ((self.register.read(rv32::get_rs1(inst), self.len)? as u32)
                                >> (rv32::get_bits(inst, 31, 20)))
                                as u64,
                            self.len,
                        )?;
                    }
                    1 => {
                        self.register.write(
                            rv32::get_rd(inst),
                            if rv32::get_bits(inst, 31, 31) == 1 {
                                self.register.read(rv32::get_rs1(inst), self.len)?
                                    >> get_bits(inst, 25, 20)
                            } else {
                                rv32::get_bits_extended(
                                    (self.register.read(rv32::get_rs1(inst), self.len)?
                                        >> get_bits(inst, 25, 20))
                                        as u32,
                                    31 - get_bits(inst, 25, 20) as usize,
                                    0,
                                ) as u64
                            },
                            self.len,
                        )?;
                    }
                    _ => {
                        return Err(String::from("No inst on SRLI_SRAI"));
                    }
                },
                _ => {
                    return Err(String::from("No inst on arithmatic immediate"));
                }
            },
            op::AREG => match rv32::get_funct3(inst) {
                f3r::ADD_SUB => {
                    println!("ADD_SUB");
                    let state = State {
                        rd: rv32::get_rd(inst),
                        rs1: rv32::get_rs1(inst),
                        rs2: rv32::get_rs2(inst),
                        imm: rv32::get_bits(inst, 31, 25) as u64,
                    };
                    let areg = add.exec_register(state, &self.register)?;
                    for i in 0..32 {
                        self.register.write(i, areg.read(i, 4)?, 4)?;
                    }
                    /*
                    self.register.write(
                        rv32::get_rd(inst),
                        self.register.read(rv32::get_rs1(inst), self.len)?
                            + self.register.read(rv32::get_rs2(inst), self.len)?,
                        self.len,
                    )?;
                    */
                }
                _ => {
                    return Err(String::from("No inst on arithmatic register"));
                }
            },
            op::CSR => match rv32::get_funct3(inst) {
                f3c::EXCEPT => {
                    let exception = rv32::get_bits(inst, 31, 20);
                    match exception {
                        exception::ecall => {
                            let current_pc = self.pc;
                            self.csr.write(0x341, current_pc)?;
                            let current_priv = self.privilege;
                            match current_priv {
                                privilege::user => {
                                    //ほんとう？
                                    self.csr.write(0x142, 0)?;
                                    self.privilege = privilege::supervisor;
                                }
                                privilege::machine => {
                                    println!("ECALL {:?}", self.csr.read(0x305)? as u32);
                                    self.csr.write(0x142, 0)?;
                                    self.csr.write(0x341, self.pc)?;
                                    self.csr.write(0x342, 11)?;
                                    self.privilege = privilege::machine;
                                    self.pc = self.csr.read(0x305)? as u32 as u64;
                                }
                                _ => {
                                    return Err(String::from("Unknown privilege level"));
                                }
                            }
                        }
                        exception::mret => {
                            self.pc = self.csr.read(0x341)?;
                        }
                        _ => {
                            return Err(String::from("No inst on CSR EXCEPTION"));
                        }
                    }
                }
                f3c::CSRRW => {
                    println!("x5={:?}", self.register.read(5, self.len)?);
                    println!(
                        "CSRRW x{:?}, 0x{:x}, x{:?}",
                        rv32::get_rd(inst),
                        rv32::get_bits(inst, 31, 20,),
                        rv32::get_rs1(inst)
                    );
                    let csr = rv32::get_bits(inst, 31, 20) as usize;
                    let t = self.csr.read(csr)?;
                    self.csr
                        .write(csr, self.register.read(rv32::get_rs1(inst), self.len)?)?;
                    self.register.write(rv32::get_rd(inst), t, self.len)?;
                }
                f3c::CSRRS => {
                    let csr = rv32::get_bits(inst, 31, 20) as usize;
                    let t = self.csr.read(csr)?;
                    self.csr
                        .write(csr, t | self.register.read(rv32::get_rs1(inst), self.len)?)?;
                    self.register.write(rv32::get_rd(inst), t, self.len)?;
                }
                f3c::CSRRC => {
                    let csr = rv32::get_bits(inst, 31, 20) as usize;
                    let t = self.csr.read(csr)?;
                    self.csr
                        .write(csr, t & !self.register.read(rv32::get_rs1(inst), self.len)?)?;
                    self.register.write(rv32::get_rd(inst), t, self.len)?;
                }
                f3c::CSRRWI => {
                    let csr = rv32::get_bits(inst, 31, 20) as usize;
                    self.register
                        .write(rv32::get_rd(inst), self.csr.read(csr)?, self.len)?;
                    self.csr.write(csr, rv32::get_bits(inst, 19, 15) as u64)?;
                }
                f3c::CSRRSI => {
                    let csr = rv32::get_bits(inst, 31, 20) as usize;
                    let t = self.csr.read(csr)?;
                    self.csr
                        .write(csr, t | rv32::get_bits(inst, 19, 15) as u64)?;
                    self.register.write(rv32::get_rd(inst), t, self.len)?;
                }
                f3c::CSRRCI => {
                    let csr = rv32::get_bits(inst, 31, 20) as usize;
                    let t = self.csr.read(csr)?;
                    self.csr
                        .write(csr, t & (!rv32::get_bits(inst, 19, 15)) as u64)?;
                    self.register.write(rv32::get_rd(inst), t, self.len)?;
                }
                _ => {
                    return Err(String::from("No inst on CSR"));
                }
            },
            op::FENCE => {
                return Ok(());
            }
            _ => {
                return Err(String::from("No instruction"));
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
