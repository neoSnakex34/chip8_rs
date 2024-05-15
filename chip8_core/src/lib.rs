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
        let digit4 = (op & 0x000F);

        match (digit1, digit2, digit3, digit4) {
            // NOP 0000
            (0, 0, 0, 0) => return,
            // CLEAR SCREEN 00E0
            (0, 0, 0xE, 0) => { self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT]; }
            // TODO pf 19
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
