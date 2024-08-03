use std::{env, fs, io, process::ExitCode};
mod lexer;
mod token;
use token::*;
use lexer::*;
struct Serialiser {
    data: Vec<u8>,
    head: usize, // in 12 bit instructions
}
const IOT: u8 = 0b110;
impl Serialiser {
    fn new() -> Self {
        Self { data: Vec::new(), head: 0 }
    }
    fn push_u12(&mut self, v: u16) {
        if self.head % 2 == 0 {
            self.data.push(v as u8);
        } else {
            let last = self.data.len()-1;
            self.data[last] = (v << 4) as u8;
        }
        self.data.push((v >> 8) as u8);
        self.head+=1;
    }
}
impl Output for Serialiser {
    fn encode_iot(&mut self, device: u8, function: u8) -> io::Result<()> {
        let a = ((IOT) as u16) | ((device as u16) << 3) | ((function as u16) << 9);
        self.push_u12(a);
        Ok(())
    }
    fn encode_basic(&mut self, opcode: u8, mode: u8, addr: u8) -> io::Result<()> {
        let a = ((opcode as u16)) | ((mode as u16) << 3) | ((addr as u16) << 5);
        self.push_u12(a);
        Ok(())
    }
    fn get_ip(&self) -> usize {
        self.head
    }
}
fn expect_int(t: Option<Token>, what: &str) -> u8 {
    if t.is_none() { panic!("Expected {} but found EOF",what); }
    let t = t.as_ref().unwrap();
    match t.kind {
        TokenKind::Integer(v) => v,
        _ =>  panic!("Expected {} but found {}",what, TDisplay(t)), 
    }
}
trait Output {
    fn encode_iot(&mut self, device: u8, function: u8) -> io::Result<()>;
    fn encode_basic(&mut self, opcode: u8, mode: u8, addr: u8) -> io::Result<()>;
    #[allow(dead_code)]
    fn get_ip(&self) -> usize;
}
fn assemble_program(out: &mut dyn Output, src: &str) -> io::Result<()> {
    let mut lexer = Lexer::new(&src);
    while let Some(t) = lexer.next() {
        match t.kind {
            TokenKind::Word(op) => {
                let opcode: u8 = match op {
                    "and" | "AND" => 0b000,
                    "tad" | "TAD" => 0b001,
                    "isz" | "ISZ" => 0b010,
                    "dca" | "DCA" => 0b011,
                    "call"| "CALL"=> 0b100,
                    "jmp" | "JMP" => 0b101,
                    "iot" | "IOT" => 0b110,
                    "opr" | "OPR" => 0b111,
                    _ => panic!("Unknown instruction {}",op)
                };
                if opcode == IOT {
                    let device = expect_int(lexer.next(), "device after IOT");
                    let function = expect_int(lexer.next(), "function after IOT");
                    out.encode_iot(device, function)?;
                } else {
                   let mut mode = 0;
                   let addr;
                   if let Some(t) = lexer.peak() {
                       mode = match t.kind {
                           TokenKind::Word("I") => 0b1,
                           TokenKind::Word("Z") => 0b10,
                           TokenKind::Word("IZ") => 0b11,
                           _ => 0,
                       }
                   } 
                   if mode != 0 { lexer.eat(); }
                   match lexer.next() {
                       Some(t) => {
                           match t.kind {
                               TokenKind::Integer(v) => {
                                   addr = v
                               }
                               _ => panic!("Expected Integer or [ after instruction but got {}",TDisplay(&t))
                           }
                       }
                       None => panic!("Expected value for instruction {} but found nothing",op),
                   }
                   if addr > 127 {
                       panic!("addr too big: {}",addr);
                   } 
                   out.encode_basic(opcode, mode, addr)?;
                }
            }
            _ => panic!("Unexpected token: {:?}",t.kind)
        }
    }
    Ok(())
}
fn main() -> ExitCode {
    let mut args = env::args();
    let _exe = args.next().expect("exe as first argument");
    let mut path: Option<String> = None;
    let mut opath: Option<String> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" => {
                if opath.is_none() {
                    opath = 
                    if let Some(v) = args.next() {
                        Some(v)
                    } else {
                        eprintln!("Expected output path after -o");
                        return ExitCode::FAILURE;
                    };
                }
            }
            _ => {
                if path.is_none() {
                   path = Some(arg);
                } else {
                   eprintln!("Unknown argument {}",arg);
                   return ExitCode::FAILURE;
                }
            }
        }
    }
    if path.is_none() {
        eprintln!("Expected input path!\n");
        return ExitCode::FAILURE;
    }
    let path = path.as_ref().unwrap().as_str();
    let opath = opath.as_ref().map(|x| x.as_str()).unwrap_or("out.bin");
    let src = match fs::read_to_string(&path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to read {}: {}",path,e);
            return ExitCode::FAILURE;
        }
    };
    let mut serial = Serialiser::new();
    if let Err(e) = assemble_program(&mut serial, &src) {
        eprintln!("Failed to serialise {}",e);
        return ExitCode::FAILURE;
    }
    if let Err(e) = fs::write(opath, &serial.data) {
        eprintln!("Failed to output to {}: {}",opath,e);
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
