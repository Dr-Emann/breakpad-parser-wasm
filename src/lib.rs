#![no_std]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use core::mem;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    type Api;

    #[wasm_bindgen(method, js_name = onLine)]
    fn on_line(api: &Api, addr: u32, size: u32, line: u32, file_index: u32);
    #[wasm_bindgen(method, js_name = onFunc)]
    fn on_func(api: &Api, addr: u32, size: u32, params: u32);
    #[wasm_bindgen(method, js_name = onFile)]
    fn on_file(api: &Api, index: u32);
    #[wasm_bindgen(method, js_name = onPublic)]
    fn on_public(api: &Api, addr: u32, params: u32);
    #[wasm_bindgen(method, js_name = onStrValue)]
    fn on_str_value(api: &Api, value: &[u8]);
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
enum State {
    Start = 0,

    LineHexAddr = 1,
    LineHexSize = 2,
    LineDecLine = 3,
    LineDecFile = 4,
    LineEnd = 5,

    Skip = 6,

    FuncOrFile = 7,

    Func = 8,
    FuncHexAddr = 9,
    FuncHexSize = 10,
    FuncHexParams = 11,
    FuncStrName = 12,
    FuncEnd = 13,

    File = 14,
    FileDecIndex = 15,
    FileStrName = 16,
    FileEnd = 17,

    Public = 18,
    PublicHexAddr = 19,
    PublicHexParams = 20,
    PublicStrName = 21,
    PublicEnd = 22,
}

static HEX_TABLE: [u8; 256] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // 0x00
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x10
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x20
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
    0x07, // 0x30
    0x08, 0x09, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x40
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x50
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0xff, // 0x60
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x70
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x80
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x90
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0xa0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0xb0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0xc0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0xd0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0xe0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0xf0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
];

static DEC_TABLE: [u8; 256] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // 0x00
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x10
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x20
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
    0x07, // 0x30
    0x08, 0x09, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x40
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x50
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x60
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x70
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x80
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x90
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0xa0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0xb0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0xc0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0xd0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0xe0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0xf0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
];

#[wasm_bindgen]
struct Parser {
    state: State,
    int_value: u32,
    row: [u32; 4],
    row_pos: usize,
    api: Api,
}

#[wasm_bindgen]
impl Parser {
    #[wasm_bindgen(constructor)]
    pub fn new(api: Api) -> Self {
        Self {
            state: State::Start,
            int_value: 0,
            row: [0; 4],
            row_pos: 0,
            api,
        }
    }

    #[wasm_bindgen]
    pub fn parse(&mut self, chunk: &[u8]) {
        let mut offset: usize = 0;
        while offset < chunk.len() {
            offset = match self.state {
                State::Start => self.parse_start(chunk, offset),
                State::FuncOrFile => self.parse_func_or_file(chunk, offset),

                State::LineHexAddr => self.parse_hex(chunk, offset),
                State::LineHexSize => self.parse_hex(chunk, offset),
                State::FuncHexAddr => self.parse_hex(chunk, offset),
                State::FuncHexSize => self.parse_hex(chunk, offset),
                State::FuncHexParams => self.parse_hex(chunk, offset),
                State::PublicHexAddr => self.parse_hex(chunk, offset),
                State::PublicHexParams => self.parse_hex(chunk, offset),

                State::LineDecLine => self.parse_dec(chunk, offset),
                State::LineDecFile => self.parse_dec(chunk, offset),
                State::FileDecIndex => self.parse_dec(chunk, offset),

                State::FuncStrName => self.parse_str(chunk, offset),
                State::FileStrName => self.parse_str(chunk, offset),
                State::PublicStrName => self.parse_str(chunk, offset),

                State::Func => self.skip_until_digit(chunk, offset),
                State::File => self.skip_until_digit(chunk, offset),
                State::Public => self.skip_until_digit(chunk, offset),

                State::LineEnd => {
                    self.on_line_end();
                    offset
                }
                State::FuncEnd => {
                    self.on_func_end();
                    offset
                }
                State::FileEnd => {
                    self.on_file_end();
                    offset
                }
                State::PublicEnd => {
                    self.on_public_end();
                    offset
                }

                State::Skip => self.skip_past_newline(chunk, offset),
            }
        }
    }

    #[wasm_bindgen]
    pub fn finish(&mut self) {
        match self.state {
            State::LineEnd => self.on_line_end(),
            State::FuncEnd => self.on_func_end(),
            State::FileEnd => self.on_file_end(),
            State::PublicEnd => self.on_public_end(),
            _ => (),
        }
    }

    fn parse_start(&mut self, chunk: &[u8], offset: usize) -> usize {
        let ch = chunk[offset];
        if HEX_TABLE[ch as usize] != 0xff {
            self.state = State::LineHexAddr;

            // First character is significant
            return offset;
        }

        self.state = match chunk[offset] as char {
            'F' => State::FuncOrFile,
            'P' => State::Public,

            // Likely STACK
            _ => State::Skip,
        };

        offset + 1
    }

    fn parse_func_or_file(&mut self, chunk: &[u8], offset: usize) -> usize {
        self.state = match chunk[offset] as char {
            'U' => State::Func,
            'I' => State::File,
            _ => State::Skip,
        };
        offset + 1
    }

    fn parse_hex(&mut self, chunk: &[u8], offset: usize) -> usize {
        let mut int_value = self.int_value;
        for i in offset..chunk.len() {
            let d = HEX_TABLE[chunk[i] as usize];
            if d == 0xff {
                self.row[self.row_pos] = int_value;
                self.row_pos += 1;
                self.int_value = 0;
                self.bump_state();
                return i + 1;
            }

            int_value = (int_value << 4) | (d as u32);
        }
        self.int_value = int_value;
        chunk.len()
    }

    fn parse_dec(&mut self, chunk: &[u8], offset: usize) -> usize {
        let mut int_value = self.int_value;
        for i in offset..chunk.len() {
            let d = DEC_TABLE[chunk[i] as usize];
            if d == 0xff {
                self.row[self.row_pos] = int_value;
                self.row_pos += 1;
                self.int_value = 0;
                self.bump_state();
                return i + 1;
            }

            int_value = (int_value * 10) + d as u32;
        }
        self.int_value = int_value;
        chunk.len()
    }

    fn parse_str(&mut self, chunk: &[u8], offset: usize) -> usize {
        for i in offset..chunk.len() {
            if chunk[i] as char == '\n' {
                self.bump_state();
                self.api.on_str_value(&chunk[offset..i]);
                return i + 1;
            }
        }
        self.api.on_str_value(&chunk[offset..chunk.len()]);
        return chunk.len();
    }

    fn skip_until_digit(&mut self, chunk: &[u8], offset: usize) -> usize {
        for i in offset..chunk.len() {
            if HEX_TABLE[chunk[i] as usize] != 0xff {
                self.bump_state();
                return i;
            }
        }
        return chunk.len();
    }

    fn skip_past_newline(&mut self, chunk: &[u8], offset: usize) -> usize {
        for i in offset..chunk.len() {
            if chunk[i] as char == '\n' {
                self.state = State::Start;
                return i + 1;
            }
        }
        return chunk.len();
    }

    fn bump_state(&mut self) {
        self.state = unsafe { mem::transmute(self.state as u8 + 1) }
    }

    fn on_line_end(&mut self) {
        self.on_end();
        self.api
            .on_line(self.row[0], self.row[1], self.row[2], self.row[3]);
    }

    fn on_func_end(&mut self) {
        self.on_end();
        self.api.on_func(self.row[0], self.row[1], self.row[2]);
    }

    fn on_file_end(&mut self) {
        self.on_end();
        self.api.on_file(self.row[0]);
    }

    fn on_public_end(&mut self) {
        self.on_end();
        self.api.on_public(self.row[0], self.row[1]);
    }

    fn on_end(&mut self) {
        self.row_pos = 0;
        self.state = State::Start;
    }
}