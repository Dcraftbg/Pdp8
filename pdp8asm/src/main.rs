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
const fn addr_u12_to_normal(addr: usize) -> usize {
    addr * 3 / 2
}
impl Serialiser {
    fn new() -> Self {
        Self { data: Vec::new(), head: 0 }
    }
    fn push_u12(&mut self, v: u16) {
        let normhead = addr_u12_to_normal(self.head);
        if normhead + 2 > self.data.len() { self.data.resize_with(normhead+2, || 0); }
        if self.head % 2 == 0 {
            self.data[normhead]   = v as u8;
            self.data[normhead+1] &= !0b00001111;
            self.data[normhead+1] |= (v >> 8) as u8;
        } else {
            self.data[normhead+1] &= !0b11110000;
            self.data[normhead] |= (v << 4) as u8;
            self.data[normhead+1] |= (v >> 4) as u8;
        }
        self.head+=1;
    }
    fn decode_u12(&mut self) -> u16 {
        let normhead = addr_u12_to_normal(self.head);
        if self.head % 2 == 0 {
            assert!(normhead + 1 < self.data.len());
            (self.data[normhead] as u16) | (((self.data[normhead+1] as u16) & 0b1111) << 8)
        } else {
            assert!(normhead + 1 < self.data.len());
            ((self.data[normhead] as u16) >> 4) | (((self.data[normhead+1] as u16)) << 4)
        }
    }
    fn decode_basic(&mut self) -> io::Result<(u8, u8, u8)> {
        let v = self.decode_u12();
        Ok(((v & 0b111) as u8, ((v >> 3) & 0b11) as u8, (v >> 5) as u8))
    }
    fn set_ip(&mut self, ip: usize) -> usize {
        if ip > self.data.len() { self.data.resize_with(ip, || 0); }
        let old = self.head;
        self.head = ip;
        old
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
fn assemble_program(out: &mut Serialiser, src: &str) -> io::Result<()> {
    let mut lexer = Lexer::new(&src);
    let mut labels: HashMap<&str, usize> = HashMap::new();
    let mut future_labels: HashMap<&str, Vec<usize>> = HashMap::new();
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
            TokenKind::CurrentInst => {
                if let Some(tok) = lexer.next() {
                    assert!(tok.kind == TokenKind::Equal, "ERROR: expected = after $ but found {}",TDisplay(&tok));
                    let tok = lexer.next();
                    match tok {
                       Some(tok) => {
                           match tok.kind {
                               TokenKind::Integer(v) => {
                                   out.set_ip(v as usize);
                               }
                               _ => {
                                   eprintln!("ERROR: expected integer after \"$ =\" but got {}", TDisplay(&tok));
                                   panic!();
                               }
                           }
                       }
                       None => {
                           eprintln!("ERROR: expected integer after \"$ =\" but got nothing!");
                           panic!()
                       }
                    }
                } else {
                    eprintln!("ERROR: expected = after $ but found nothing");
                    panic!()
                }
            }
            TokenKind::Word(op) => {
                if let Some(tok) = lexer.peak() {
                    if tok.kind == TokenKind::DoubleDot {
                        lexer.eat();
                        let at = out.get_ip();
                        if let Some(v) = future_labels.remove(op) {
                            for x in v {
                                out.set_ip(x);
                                let (opcode, mode, _addr) = out.decode_basic()?;
                                out.encode_basic(opcode, mode, at as u8)?;
                            }
                        }
                        out.set_ip(at);
                        labels.insert(op, at);
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
                   let mut addr = 0;
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
                                       if let Some(v) = future_labels.get_mut(w) {
                                           v.push(out.get_ip() as usize);
                                       } else {
                                           future_labels.insert(w, vec![out.get_ip() as usize]);
                                       }
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
    if future_labels.len() > 0 {
        for (name, v) in future_labels.iter() {
           eprintln!("ERROR: Unknown label: {} referenced in {} places",name,v.len());
        }
        panic!()
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
        eprintln!("Expected input path!");
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
