use std::{env, io};
use std::io::Read;
use std::fs::File;
use std::process::exit;

use crate::cpu::Cpu;
use crate::mmu::Mmu;

mod cpu;
mod mmu;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let l = args.len();
    if l <= 1 {
        println!("Lack of argument, binary name");
        exit(1);
    }
    let f = &args[1];
    let mut f = File::open(f)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let mut memory: Vec<u8> = Vec::with_capacity(1000);
    let mut mmu = Mmu::new(buf);
    let mut cpu = Cpu::new(0, [0;4096], [0;32], mmu);
    cpu.execute()?;
    Ok(())
}
