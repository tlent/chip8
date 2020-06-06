use std::convert::TryInto;

use rand::prelude::*;

use crate::display::Display;
use crate::input::Input;

pub struct CPU {
    memory: [u8; 4096],
    registers: [u8; 16],
    address_register: u16, // "Register I"
    instruction_pointer: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
}

impl CPU {
    pub fn new(program: &[u8]) -> Self {
        let mut memory = [0; 4096];
        let character_sprite_data = CHARACTER_SPRITES
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        memory[0..character_sprite_data.len()].copy_from_slice(&character_sprite_data);
        memory[0x200..0x200 + program.len()].copy_from_slice(program);
        Self {
            memory,
            registers: [0; 16],
            address_register: 0,
            instruction_pointer: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            stack: Vec::with_capacity(16),
        }
    }

    pub fn cycle(&mut self, display: &mut Display, input: &Input) {
        let i = self.instruction_pointer as usize;
        let opcode = u16::from_be_bytes(self.memory[i..i + 2].try_into().unwrap());
        let instruction = Instruction::from_opcode(opcode);
        self.execute_instruction(instruction, display, input);
    }

    pub fn tick(&mut self) {
        self.delay_timer = self.delay_timer.saturating_sub(1);
        self.sound_timer = self.sound_timer.saturating_sub(1);
    }

    pub fn should_play_sound(&self) -> bool {
        self.sound_timer > 0
    }

    fn execute_instruction(
        &mut self,
        instruction: Instruction,
        display: &mut Display,
        input: &Input,
    ) {
        let mut jump = false;
        match instruction {
            Instruction::RCA1802 { .. } => {}
            Instruction::ClearScreen => {
                display.clear();
            }
            Instruction::Return => {
                self.instruction_pointer =
                    self.stack.pop().expect("Return called with empty stack");
                jump = true;
            }
            Instruction::Jump { address } => {
                self.instruction_pointer = address;
                jump = true;
            }
            Instruction::Subroutine { address } => {
                self.stack.push(self.instruction_pointer + 2);
                self.instruction_pointer = address;
                jump = true;
            }
            Instruction::IfEqualConst { register, value } => {
                if self.registers[register as usize] == value {
                    self.instruction_pointer += 4;
                    jump = true;
                }
            }
            Instruction::IfNotEqualConst { register, value } => {
                if self.registers[register as usize] != value {
                    self.instruction_pointer += 4;
                    jump = true;
                }
            }
            Instruction::IfEqualRegister { a, b } => {
                if self.registers[a as usize] == self.registers[b as usize] {
                    self.instruction_pointer += 4;
                    jump = true;
                }
            }
            Instruction::SetConst { register, value } => self.registers[register as usize] = value,
            Instruction::AddConst { register, value } => self.registers[register as usize] += value,
            Instruction::SetRegister { dest, src } => {
                self.registers[dest as usize] = self.registers[src as usize]
            }
            Instruction::Or { a, b } => self.registers[a as usize] |= self.registers[b as usize],
            Instruction::And { a, b } => self.registers[a as usize] &= self.registers[b as usize],
            Instruction::Xor { a, b } => self.registers[a as usize] ^= self.registers[b as usize],
            Instruction::Add { a, b } => {
                let a_val = self.registers[a as usize];
                let b_val = self.registers[b as usize];
                let (sum, overflow) = a_val.overflowing_add(b_val);
                self.registers[a as usize] = sum;
                self.registers[15] = if overflow { 1 } else { 0 };
            }
            Instruction::Sub { a, b } => {
                let a_val = self.registers[a as usize];
                let b_val = self.registers[b as usize];
                let (diff, overflow) = a_val.overflowing_sub(b_val);
                self.registers[a as usize] = diff;
                // 1 if NOT overflow
                self.registers[15] = if !overflow { 1 } else { 0 };
            }
            Instruction::ShiftRight { register } => {
                let val = self.registers[register as usize];
                self.registers[15] = val & 1;
                self.registers[register as usize] >>= 1;
            }
            Instruction::NegSub { a, b } => {
                let a_val = self.registers[a as usize];
                let b_val = self.registers[b as usize];
                let (diff, overflow) = b_val.overflowing_sub(a_val);
                self.registers[a as usize] = diff;
                // 1 if NOT overflow
                self.registers[15] = if !overflow { 1 } else { 0 };
            }
            Instruction::ShiftLeft { register } => {
                let val = self.registers[register as usize];
                self.registers[15] = val >> 7;
                self.registers[register as usize] <<= 1;
            }
            Instruction::IfNotEqualRegister { a, b } => {
                if self.registers[a as usize] != self.registers[b as usize] {
                    self.instruction_pointer += 4;
                    jump = true;
                }
            }
            Instruction::SetI { address } => self.address_register = address,
            Instruction::JumpOffset { address } => {
                self.instruction_pointer = address + self.registers[0] as u16;
                jump = true;
            }
            Instruction::Rand { register, value } => {
                let mut rng = rand::thread_rng();
                self.registers[register as usize] = rng.gen::<u8>() & value;
            }
            Instruction::DrawSprite {
                x: x_register,
                y: y_register,
                height,
            } => {
                let i = self.address_register as usize;
                let start_x = self.registers[x_register as usize];
                let start_y = self.registers[y_register as usize];
                let n = height as usize;
                let sprite = &self.memory[i..i + n];
                let (display_width, display_height) = display.dimensions();
                let mut collision = false;
                for (i, &byte) in sprite.iter().enumerate() {
                    let mut mask = 1 << 7;
                    for j in 0..8 {
                        let x = (start_x + j) % display_width;
                        let y = (start_y + i as u8) % display_height;
                        let value = mask & byte > 0;
                        let prev_value = display.get_pixel(x, y);
                        if value && prev_value {
                            collision = true;
                        }
                        display.set_pixel(x, y, value ^ prev_value);
                        mask >>= 1;
                    }
                }
                self.registers[15] = collision as u8;
            }
            Instruction::IfPressed { register } => {
                let key = self.registers[register as usize];
                if input.is_key_pressed(key) {
                    self.instruction_pointer += 4;
                    jump = true;
                }
            }
            Instruction::IfNotPressed { register } => {
                let key = self.registers[register as usize];
                if !input.is_key_pressed(key) {
                    self.instruction_pointer += 4;
                    jump = true;
                }
            }
            Instruction::GetTimer { register } => {
                self.registers[register as usize] = self.delay_timer
            }
            Instruction::AwaitInput { register } => {
                jump = true;
                if let Some(key) = input.get_pressed_key() {
                    self.registers[register as usize] = key;
                    self.instruction_pointer += 2;
                }
            }
            Instruction::SetTimer { register } => {
                self.delay_timer = self.registers[register as usize]
            }
            Instruction::SetSound { register } => {
                self.sound_timer = self.registers[register as usize]
            }
            Instruction::AddToI { register } => {
                self.address_register += self.registers[register as usize] as u16
            }
            Instruction::SetIToFontChar { register } => {
                let val = self.registers[register as usize];
                assert!(val <= 16);
                self.address_register = val as u16 * 5;
            }
            Instruction::BinaryCodedDecimal { register } => {
                let val = self.registers[register as usize];
                let i = self.address_register as usize;
                self.memory[i] = val / 100;
                self.memory[i + 1] = val % 100 / 10;
                self.memory[i + 2] = val % 10;
            }
            Instruction::RegisterDump { register } => {
                let start = self.address_register as usize;
                let n = register as usize;
                for i in 0..=n {
                    self.memory[start + i] = self.registers[i];
                }
            }
            Instruction::RegisterLoad { register } => {
                let start = self.address_register as usize;
                let n = register as usize;
                for i in 0..=n {
                    self.registers[i] = self.memory[start + i];
                }
            }
        };
        if !jump {
            self.instruction_pointer += 2;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Opcode {
    control: u8,
    address: u16,
    constant: u8,
    a: u8,
    b: u8,
    c: u8,
}

impl Opcode {
    fn new(opcode: u16) -> Self {
        Self {
            control: ((opcode & 0xF000) >> 12) as u8,
            address: opcode & 0x0FFF,
            constant: (opcode & 0x00FF) as u8,
            a: ((opcode & 0x0F00) >> 8) as u8,
            b: ((opcode & 0x00F0) >> 4) as u8,
            c: (opcode & 0x000F) as u8,
        }
    }
}

// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#3.1
// https://en.wikipedia.org/wiki/CHIP-8#Opcode_table
#[derive(Debug, PartialEq, Clone, Copy)]
enum Instruction {
    RCA1802 { address: u16 },                    //0NNN
    ClearScreen,                                 //00E0
    Return,                                      //00EE
    Jump { address: u16 },                       //1NNN
    Subroutine { address: u16 },                 //2NNN
    IfEqualConst { register: u8, value: u8 },    //3XNN
    IfNotEqualConst { register: u8, value: u8 }, //4XNN
    IfEqualRegister { a: u8, b: u8 },            //5XY0
    SetConst { register: u8, value: u8 },        //6XNN
    AddConst { register: u8, value: u8 },        //7XNN
    SetRegister { dest: u8, src: u8 },           //8XY0
    Or { a: u8, b: u8 },                         //8XY1
    And { a: u8, b: u8 },                        //8XY2
    Xor { a: u8, b: u8 },                        //8XY3
    Add { a: u8, b: u8 },                        //8XY4
    Sub { a: u8, b: u8 },                        //8XY5
    ShiftRight { register: u8 },                 //8XY6
    NegSub { a: u8, b: u8 },                     //8XY7
    ShiftLeft { register: u8 },                  //8XYE
    IfNotEqualRegister { a: u8, b: u8 },         //9XY0
    SetI { address: u16 },                       //ANNN
    JumpOffset { address: u16 },                 //BNNN
    Rand { register: u8, value: u8 },            //CXNN
    DrawSprite { x: u8, y: u8, height: u8 },     //DXYN
    IfPressed { register: u8 },                  //EX9E
    IfNotPressed { register: u8 },               //EXA1
    GetTimer { register: u8 },                   //FX07
    AwaitInput { register: u8 },                 //FX0A
    SetTimer { register: u8 },                   //FX15
    SetSound { register: u8 },                   //FX18
    AddToI { register: u8 },                     //FX1E
    SetIToFontChar { register: u8 },             //FX29
    BinaryCodedDecimal { register: u8 },         //FX33
    RegisterDump { register: u8 },               //FX55
    RegisterLoad { register: u8 },               //FX65
}

impl Instruction {
    fn from_opcode(opcode: u16) -> Self {
        let opcode = Opcode::new(opcode);
        match opcode {
            Opcode {
                control: 0,
                a: 0,
                constant: 0xE0,
                ..
            } => Self::ClearScreen,
            Opcode {
                control: 0,
                a: 0,
                constant: 0xEE,
                ..
            } => Self::Return,
            Opcode { control: 0, .. } => Self::RCA1802 {
                address: opcode.address,
            },
            Opcode { control: 1, .. } => Self::Jump {
                address: opcode.address,
            },
            Opcode { control: 2, .. } => Self::Subroutine {
                address: opcode.address,
            },
            Opcode { control: 3, .. } => Self::IfEqualConst {
                register: opcode.a,
                value: opcode.constant,
            },
            Opcode { control: 4, .. } => Self::IfNotEqualConst {
                register: opcode.a,
                value: opcode.constant,
            },
            Opcode {
                control: 5, c: 0, ..
            } => Self::IfEqualRegister {
                a: opcode.a,
                b: opcode.b,
            },
            Opcode { control: 6, .. } => Self::SetConst {
                register: opcode.a,
                value: opcode.constant,
            },
            Opcode { control: 7, .. } => Self::AddConst {
                register: opcode.a,
                value: opcode.constant,
            },
            Opcode {
                control: 8, c: 0, ..
            } => Self::SetRegister {
                dest: opcode.a,
                src: opcode.b,
            },
            Opcode {
                control: 8, c: 1, ..
            } => Self::Or {
                a: opcode.a,
                b: opcode.b,
            },
            Opcode {
                control: 8, c: 2, ..
            } => Self::And {
                a: opcode.a,
                b: opcode.b,
            },
            Opcode {
                control: 8, c: 3, ..
            } => Self::Xor {
                a: opcode.a,
                b: opcode.b,
            },
            Opcode {
                control: 8, c: 4, ..
            } => Self::Add {
                a: opcode.a,
                b: opcode.b,
            },
            Opcode {
                control: 8, c: 5, ..
            } => Self::Sub {
                a: opcode.a,
                b: opcode.b,
            },
            Opcode {
                control: 8, c: 6, ..
            } => Self::ShiftRight { register: opcode.a },
            Opcode {
                control: 8, c: 7, ..
            } => Self::NegSub {
                a: opcode.a,
                b: opcode.b,
            },
            Opcode {
                control: 8, c: 0xE, ..
            } => Self::ShiftLeft { register: opcode.a },
            Opcode {
                control: 9, c: 0, ..
            } => Self::IfNotEqualRegister {
                a: opcode.a,
                b: opcode.b,
            },
            Opcode { control: 0xA, .. } => Self::SetI {
                address: opcode.address,
            },
            Opcode { control: 0xB, .. } => Self::JumpOffset {
                address: opcode.address,
            },
            Opcode { control: 0xC, .. } => Self::Rand {
                register: opcode.a,
                value: opcode.constant,
            },
            Opcode { control: 0xD, .. } => Self::DrawSprite {
                x: opcode.a,
                y: opcode.b,
                height: opcode.c,
            },
            Opcode {
                control: 0xE,
                constant: 0x9E,
                ..
            } => Self::IfPressed { register: opcode.a },
            Opcode {
                control: 0xE,
                constant: 0xA1,
                ..
            } => Self::IfNotPressed { register: opcode.a },
            Opcode {
                control: 0xF,
                constant: 0x07,
                ..
            } => Self::GetTimer { register: opcode.a },
            Opcode {
                control: 0xF,
                constant: 0x0A,
                ..
            } => Self::AwaitInput { register: opcode.a },
            Opcode {
                control: 0xF,
                constant: 0x15,
                ..
            } => Self::SetTimer { register: opcode.a },
            Opcode {
                control: 0xF,
                constant: 0x18,
                ..
            } => Self::SetSound { register: opcode.a },
            Opcode {
                control: 0xF,
                constant: 0x1E,
                ..
            } => Self::AddToI { register: opcode.a },
            Opcode {
                control: 0xF,
                constant: 0x29,
                ..
            } => Self::SetIToFontChar { register: opcode.a },
            Opcode {
                control: 0xF,
                constant: 0x33,
                ..
            } => Self::BinaryCodedDecimal { register: opcode.a },
            Opcode {
                control: 0xF,
                constant: 0x55,
                ..
            } => Self::RegisterDump { register: opcode.a },
            Opcode {
                control: 0xF,
                constant: 0x65,
                ..
            } => Self::RegisterLoad { register: opcode.a },
            _ => panic!("invalid opcode"),
        }
    }
}

const CHARACTER_SPRITES: [[u8; 5]; 16] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0], // 0
    [0x20, 0x60, 0x20, 0x20, 0x70], // 1
    [0xF0, 0x10, 0xF0, 0x80, 0xF0], // 2
    [0xF0, 0x10, 0xF0, 0x10, 0xF0], // 3
    [0x90, 0x90, 0xF0, 0x10, 0x10], // 4
    [0xF0, 0x80, 0xF0, 0x10, 0xF0], // 5
    [0xF0, 0x80, 0xF0, 0x90, 0xF0], // 6
    [0xF0, 0x10, 0x20, 0x40, 0x40], // 7
    [0xF0, 0x90, 0xF0, 0x90, 0xF0], // 8
    [0xF0, 0x90, 0xF0, 0x10, 0xF0], // 9
    [0xF0, 0x90, 0xF0, 0x90, 0x90], // A
    [0xE0, 0x90, 0xE0, 0x90, 0xE0], // B
    [0xF0, 0x80, 0x80, 0x80, 0xF0], // C
    [0xE0, 0x90, 0x90, 0x90, 0xE0], // D
    [0xF0, 0x80, 0xF0, 0x80, 0xF0], // E
    [0xF0, 0x80, 0xF0, 0x80, 0x80], // F
];
