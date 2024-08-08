use std::{collections::HashMap, env, fs, io, process::ExitCode};
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
            self.data.push((v >> 8) as u8);
        } else {
            let last = self.data.len()-1;
            self.data[last] |= (v << 4) as u8;
            self.data.push((v >> 4) as u8);
        }
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
    fn encode_word(&mut self, word: u16) -> io::Result<()> {
        self.push_u12(word);
        Ok(())
    }
    fn get_ip(&self) -> usize {
        self.head
    }
}
fn expect_u8(t: Option<Token>, what: &str) -> u8 {
    let v = expect_int(t, what);
    if v >= 256 {
        panic!("Expected integer with size <256 but got {}",v);
    }
    v as u8
}
fn expect_int(t: Option<Token>, what: &str) -> u16 {
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
    fn encode_word(&mut self, word: u16) -> io::Result<()>;
    #[allow(dead_code)]
    fn get_ip(&self) -> usize;
}
const WORD_LIMIT: u16 = 1<<12;
fn assemble_program(out: &mut dyn Output, src: &str) -> io::Result<()> {
    let mut lexer = Lexer::new(&src);
    let mut labels: HashMap<&str, usize> = HashMap::new();
    while let Some(t) = lexer.next() {
        match t.kind {
            TokenKind::DotWord(word) => {
                match word {
                    "w" => {
                        let word = expect_int(lexer.next(), "word after .w");
                        if word >= WORD_LIMIT {
                            panic!("Integer exceeds word limit of <{} but got {}",WORD_LIMIT,word);
                        }
                        out.encode_word(word)?;
                    }
                    _ => panic!("Unknown DotWord: {:?}",word)
                }
            }
            TokenKind::Word(op) => {
                if let Some(tok) = lexer.peak() {
                    if tok.kind == TokenKind::DoubleDot {
                        lexer.eat();
                        labels.insert(op, out.get_ip());
                        continue;
                    }
                } 
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
                    let device = expect_u8(lexer.next(), "device after IOT");
                    let function = expect_u8(lexer.next(), "function after IOT");
                    out.encode_iot(device, function)?;
                } else {
                   let mut mode = 0;
                   let addr;
                   if let Some(t) = lexer.peak() {
                       mode = match t.kind {
                           TokenKind::OpenSquare => 0b1,
                           TokenKind::Word("Z") => 0b10,
                           _ => 0,
                       }
                   } 
                   if mode != 0 { lexer.eat(); }
                   match lexer.next() {
                       Some(t) => {
                           match t.kind {
                               TokenKind::Integer(v) => {
                                   addr = v as u8;
                               }
                               TokenKind::CurrentInst => {
                                   // NOTE: I know its technically not correct but 
                                   // it will do for now
                                   addr = out.get_ip() as u8;
                               }
                               TokenKind::Word(w) => {
                                   if let Some(label) = labels.get(w) {
                                       addr = *label as u8;
                                   } else {
                                       panic!("Unknown label: {}",w);
                                   }
                               }
                               _ => panic!("Expected Integer or [ after instruction but got {}",TDisplay(&t))
                           }
                       }
                       None => panic!("Expected value for instruction {} but found nothing",op),
                   }
                   if addr > 127 {
                       panic!("addr too big: {}",addr);
                   }
                   if mode == 0b1 {
                       if let Some(v) = lexer.next() {
                           if v.kind != TokenKind::CloseSquare {
                               panic!("Expected closing ']' but got {}",TDisplay(&v));
                           }
                       } else {
                           panic!("Expected closing ']'");
                       }
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
