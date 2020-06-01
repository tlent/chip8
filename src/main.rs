fn main() {
    println!("Hello, world!");
}

mod display {
    struct ScreenBuffer {
        buffer: [u8; 256],
    }
}

mod input {}

mod chip8 {
    struct Chip8 {
        memory: [u8; 4096],
        registers: [u8; 16],
        address_register: u16,
        instruction_pointer: usize,
        delay_timer: u8,
        sound_timer: u8,
        stack: Vec<u16>,
    }

    impl Chip8 {
        pub fn new() -> Self {
            Self {
                memory: [0; 4096],
                registers: [0; 16],
                address_register: 0,
                instruction_pointer: 0,
                delay_timer: 0,
                sound_timer: 0,
                stack: vec![],
            }
        }

        pub fn run(&mut self) {}
    }

    // https://en.wikipedia.org/wiki/CHIP-8#Opcode_table
    enum Instruction {
        RCA1802 { address: u16 },                    //0NNN
        ClearScreen,                                 //00E0
        Return,                                      //00EE
        GoTo { address: u16 },                       //1NNN
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
        Jump { address: u16 },                       //BNNN
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
                Opcode { control: 1, .. } => Self::GoTo {
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
                Opcode { control: 0xB, .. } => Self::Jump {
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
}
