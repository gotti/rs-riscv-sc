use clap::{App, Arg};
use std::fs::File;
use std::io::Read;
use std::{env, io};

use crate::cpu::Cpu;
use crate::csr::Csr;
use crate::mmu::Mmu;
use crate::register::Register;
mod crate::shadowstack::ShadowStack;

mod cpu;
mod csr;
mod mmu;
mod register;
mod shadowstack;

fn main() -> io::Result<()> {
    let matches = App::new("rs-riscv-sc, a risc-v emulator written in rust.")
        .version("0.0")
        .arg(Arg::with_name("INPUT_FILE").help("Path to raw riscv binary starting 0, not elf."))
        .arg(Arg::with_name("test-mode").short("t").long("test-mode").help("Run riscv-tests"))
        .get_matches();
    let f = matches.value_of("INPUT_FILE").unwrap();
    let test_mode = matches.is_present("test-mode");
    let mut f = File::open(f)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let mut memory: Vec<u8> = Vec::with_capacity(1000);
    let mut mmu = Mmu::new(buf, test_mode);
    let mut csr = Csr::new([0; 4096]);
    let mut reg = Register::new([0; 32]);
    let mut cpu = Cpu::new(0, csr, reg, 0b11, mmu);
    cpu.execute()?;
    Ok(())
}
