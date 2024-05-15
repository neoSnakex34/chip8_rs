use rand;

/*
 *  FONT SPRITES IN CHIP 8 - commonly stored in addresses before 0x200
 *
 *  0 0 1 0 0 0 0 0
 *  0 1 1 0 0 0 0 0
 *  0 0 1 0 0 0 0 0    that is a one; every row can be translated as 0x something like fontset below
 *  0 0 1 0 0 0 0 0
 *  0 1 1 1 0 0 0 0
 *
 *  0 is black 1 is white
 *  every sprite in chip8 is eight pixel wide
 *  a pixel row is 1 byte (8 bit long)
 */

const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;

pub struct Emu {
    pc: u16, // program counter
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_regs: [u8; NUM_REGS], // general purpose regs
    i_reg: u16,             // for ram read and write indexing
    sp: u16,                // stack pointer
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8, // delay timer, standard timer
    st: u8, // sound timer, when it's 0 emit sound
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR, // 0x200 512 decimal
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_regs: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }

    // frontend API
    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }
    // NOTE popping an empty stack would underflow panic
    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    // reset values of emu obj
    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_regs = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    /* basic loop */
    /*
     *  fetch
     *  decode and exec
     *  move pc
     *
     */
    pub fn tick(&mut self) {
        // fetch
        let op = self.fetch();
        // decode
        // exec
        self.execute(op);
    }

    fn execute(&mut self, op: u16) {
        // each hex digit would be selected and shifted if needed
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            // NOP 0000
            (0, 0, 0, 0) => return,
            // CLEAR SCREEN 00E0
            (0, 0, 0xe, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },
            // RETURN FROM SUBROUTINE 00EE
            (0, 0, 0xe, 0xe) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            },

            // JUMP 1NNN jump to nnn
            (1, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = nnn; // 12 bit addresses are NNN
            },

            // CALL SUBROUTINE 2NNN
            (2, _, _, _) => {
                let nnn = op & 0xFFF; // decode nnn removing most significant digit bitwise
                self.push(self.pc);
                self.pc = nnn;
            },

            // SKIP NEXT IF VX == NN 3XNN if vx = nn skip (if, else)
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_regs[x] == nn {
                    self.pc += 2; // skip
                }
            },

            //SKIP NEXT IF VX != NN 4XNN
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_regs[x] != nn {
                    self.pc += 2; // skip
                }
            },

            // SKIP NEXT IF VX == VY 5XY0
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_regs[x] == self.v_regs[y] {
                    self.pc += 2;
                }
            },

            // VX = NN 6XNN set v reg x to nn
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_regs[x] = nn;
            },

            // VX += NN 7XNN
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_regs[x] = self.v_regs[x].wrapping_add(nn);
            },

            // VX = VY 8XY0
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_regs[x] = self.v_regs[y]
            },

            // 8XY1 8XY2 8XY3 bitwise ops or and xor
            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_regs[x] |= self.v_regs[y]
            },

            (8, _, _, 2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_regs[x] &= self.v_regs[y]
            },

            (8, _, _, 3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_regs[x] ^= self.v_regs[y]
            },

            // VX += VY 8XY4 uses the flag reg set to 1 if overflow occurred
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_regs[x].overflowing_add(self.v_regs[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_regs[x] = new_vx;
                self.v_regs[0xF] = new_vf; // flag reg set
            },

            // VX -= VY 8XY5 uses the flag reg set to 0 if underflow occurred
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_regs[x].overflowing_sub(self.v_regs[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_regs[x] = new_vx;
                self.v_regs[0xF] = new_vf; // flag reg set
            },

            // VX>=1 8XY6 single shift right
            (8, _, _, 6) => {
                let x = digit2 as usize;
                let lsb = self.v_regs[x] & 1; // check bit that will be dropped
                self.v_regs[x] >>= 1;
                self.v_regs[0xF] = lsb
            },

            // VY -= VX 8XY7
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_regs[y].overflowing_sub(self.v_regs[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_regs[x] = new_vx;
                self.v_regs[0xF] = new_vf; // flag reg set
            },

            // VX <= 1 8XYE
            (8, _, _, 0xE) => {
                let x = digit2 as usize;
                let msb = (self.v_regs[x] >> 7) & 1;
                self.v_regs[x] <<= 1;
                self.v_regs[0xF] = msb;
            },

            // SKIP IF VX != VY 9XY0
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_regs[x] != self.v_regs[y] {
                    self.pc += 2;
                }
            },

            // ANNN I = NNN use of i reg
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            },

            // BNNN jump to v0 + NNN
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_regs[0] as u16) + nnn;
            },

            // CXNN VX = rand() & NN random generator
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = rand::random();
                self.v_regs[x] = rng & nn;
            },

            // DXYN draw sprite
            // (x, y) coordinates to write sprites from memory to screen
            // N specifies the height of pixel rows for sprite from 1 to 16 (every row is long 8)
            (0xD, _, _, _) => {
                let x_coord = self.v_regs[digit2 as usize] as u16;
                let y_coord = self.v_regs[digit3 as usize] as u16;

                let num_rows = digit4;

                let mut flipped = false; // check if any pixel is flipped

                for y_line in 0..num_rows {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    // fetch current pixel bit
                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // fit pixel into screen
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            // pixel inded for one dimensional array
                            let idx = x + SCREEN_WIDTH * y;

                            // check if pixel would be flipped and set
                            let tmp_screen = self.get_display();
                            flipped |= tmp_screen[idx];
                            self.screen[idx] = tmp_screen[idx] ^ true;
                        }
                    }
                }

                if flipped {
                    self.v_regs[0xF] = 1;
                } else {
                    self.v_regs[0xF] = 0;
                }
            },

            // SKIP IF KEY PRESSED EX9E key pression user input 0-F possible keys
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_regs[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            },

            // SKIP IF KEY NOT PRESSED EXA1
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let vx = self.v_regs[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            },

            // VX = DT FX07
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_regs[x] = self.dt;
            },

            // WAIT FOR KEY PRESSED FX0A
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_regs[x] = i as u8; // sets the key value in vx
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    // redo opcode instr
                    self.pc -= 2;
                }
            },

            // DT = VX FX15
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.dt = self.v_regs[x];
            },

            // ST = VX FX18
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.st = self.v_regs[x];
            },

            // I += VX FX1E
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_regs[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            },

            // SET I TO FONT ADDRESS FX29
            // char  ram addr | every char is 5 bytes
            //  0       0
            //  1       5
            //  2       10
            //  14(e)   70
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.v_regs[x] as u16;
                self.i_reg = c * 5; // offset
            },

            // I = BCD of VX FX33 binary coded decimal of a number in vx stored in I
            // convert hex into pseudo decimal for user experience
            // TODO read about bcd and optimize code
            (0xF, _, 3, 3) => {
                let x = digit2 as usize;
                let vx = self.v_regs[x] as f32;

                // dividind by 100 tossing, decimal part
                let hundreds = (vx / 100.0).floor() as u8;

                let tens = ((vx / 10.0) % 10.0).floor() as u8;

                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            },

            // STORE V0 to VX INTO I FX55 populate regs from v0 to specified with ram content starting from ireg
            // STORE INTO RAM
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.ram[i + idx] = self.v_regs[idx];
                }
            },

            // LOAD INTO RAM FX65
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_regs[idx] = self.ram[i + idx];
                }
            },

            (_, _, _, _) => unimplemented!("UNIMPLEMENTED OPCODE: {}", op),
        }
    }

    fn fetch(&mut self) -> u16 {
        // two bytes for instr of 16 bits (2 parts of one byte) combined
        // and taken in BIG endian notation
        // pc goes forward by the two bites just picked
        // -------- --------
        // higher b lower b
        // pc       pc + 1
        //
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // NOTE BEEP
                // TODO i should manually implement it myself
            }

            self.st -= 1;
        }
    }
}
