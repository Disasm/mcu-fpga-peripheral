#![no_std]

use heapless::spsc::Queue;
use heapless::consts::U16;

struct RingBuffer {
    queue: Queue<u8, U16>,
}

impl RingBuffer {
    pub fn new() -> Self {
        Self {
            queue: Queue::new()
        }
    }

    pub fn space_available(&self) -> usize {
        self.queue.capacity() - self.queue.len()
    }

    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let mut n = 0;
        for byte in buf.iter_mut() {
            if let Some(b) = self.queue.dequeue() {
                *byte = b;
                n += 1;
            }
        }
        n
    }

    pub fn write_byte(&mut self, byte: u8) -> Result<(), ()> {
        self.queue.enqueue(byte).map_err(|_| ())
    }
}

pub struct DecoderReader<'a> {
    bytes: &'a [u8],
    bits: u32,
    bit_count: u8,
}

impl<'a> DecoderReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            bits: 0,
            bit_count: 0,
        }
    }

    pub fn inner(&self) -> &[u8] {
        self.bytes
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty() && (self.bit_count == 0)
    }

    pub fn len_bits(&self) -> usize {
        (self.bytes.len() * 8) + (self.bit_count as usize)
    }

    pub fn read_bit(&mut self) -> Option<bool> {
        self.feed_bits();
        if self.bit_count > 0 {
            Some(self.read_bit_internal())
        } else {
            None
        }
    }

    pub fn read_int(&mut self, mut bits: usize) -> Option<u32> {
        self.feed_bits();
        if (self.bit_count as usize) >= bits {
            let mut value = 0;
            while bits > 0 {
                bits -= 1;
                if self.read_bit_internal() {
                    value |= 1 << bits;
                }
            }
            Some(value)
        } else {
            return None
        }
    }

    fn feed_bits(&mut self) {
        while self.bit_count <= 24 && !self.bytes.is_empty() {
            self.bits = (self.bits << 8) | (self.bytes[0] as u32);
            self.bit_count += 8;
            self.bytes = &self.bytes[1..];
        }
    }

    fn read_bit_internal(&mut self) -> bool {
        self.bit_count -= 1;
        (self.bits & (1 << self.bit_count)) != 0
    }
}

pub struct DecoderWriter {
    buffer: RingBuffer,
    bits: u32,
    bit_count: u8,
}

impl DecoderWriter {
    pub fn new() -> Self {
        Self {
            buffer: RingBuffer::new(),
            bits: 0,
            bit_count: 0,
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let n = self.buffer.read(buf);
        if n < buf.len() {
            self.flush();
            let n2 = self.buffer.read(&mut buf[n..]);
            n + n2
        } else {
            n
        }
    }

    pub fn space_bits(&self) -> usize {
        (32 - self.bit_count as usize) + (self.buffer.space_available() * 8)
    }

    pub fn write_zeros(&mut self, mut count: u32) -> u32 {
        self.flush();

        let mut written = 0;
        while count > 0 && self.bit_count < 32 {
            let n = core::cmp::min(count, 32 - self.bit_count as u32);
            self.bit_count += n as u8;
            written += n;
            count -= n;
            self.flush();
        }
        written
    }

    pub fn write_bit(&mut self, bit: bool) -> Result<(), ()> {
        self.flush();
        if self.bit_count < 32 {
            self.bits |= (bit as u32) << self.bit_count;
            self.bit_count += 1;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn write_trailing_zeros(&mut self) {
        let n = self.bit_count & 7;
        if n > 0 {
            self.bit_count += 7 - n;
        }
    }

    fn flush(&mut self) {
        while self.bit_count >= 8 {
            if self.buffer.write_byte((self.bits as u8).reverse_bits()).is_err() {
                return;
            }
            self.bit_count -= 8;
            self.bits = self.bits >> 8;
        }
    }
}

pub struct Decoder<'a> {
    reader: DecoderReader<'a>,
    writer: DecoderWriter,
    state: DecoderState,
}

impl<'a> Decoder<'a> {
    pub fn new(compressed: &'a [u8]) -> Self {
        Self {
            reader: DecoderReader::new(compressed),
            writer: DecoderWriter::new(),
            state: DecoderState::Initial,
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, DecoderError> {
        let mut offset = 0;
        while offset < buf.len() {
            let n = self.writer.read(&mut buf[offset..]);
            if n > 0 {
                offset += n;
            }

            if self.state == DecoderState::Finished {
                break; // Return data read or EOF
            }
            if self.state == DecoderState::Error {
                return Err(DecoderError::InvalidState);
            }

            // No more data to read, decode something
            self.decode_one()?;
        }
        Ok(offset)
    }

    fn read_bit(&mut self) -> Result<bool, DecoderError> {
        self.reader.read_bit().ok_or(DecoderError::UnexpectedEof)
    }

    fn read_int(&mut self, bits: usize) -> Result<u32, DecoderError> {
        self.reader.read_int(bits).ok_or(DecoderError::UnexpectedEof)
    }

    fn decode_one(&mut self) -> Result<(), DecoderError> {
        match self.state {
            DecoderState::Initial => {
                let compressed = self.reader.inner();

                if compressed.len() < 8 {
                    self.state = DecoderState::Error;
                    return Err(DecoderError::InvalidHeader);
                }

                if &compressed[..8] != b"ICECOMPR" {
                    self.state = DecoderState::Error;
                    return Err(DecoderError::InvalidHeader);
                }

                self.reader.bytes = &self.reader.bytes[8..];

                self.state = DecoderState::Started;
                Ok(())
            }
            DecoderState::Started => {
                match self.decode_command() {
                    Ok(state) => {
                        self.state = state;

                        // Call the function again to fill the output buffer
                        self.decode_one()
                    }
                    Err(e) => {
                        self.state = DecoderState::Error;
                        Err(e)
                    }
                }
            }
            DecoderState::WriteZeroOne(count) => {
                if count > 0 {
                    let n = self.writer.write_zeros(count);
                    if n > 0 {
                        self.state = DecoderState::WriteZeroOne(count - n);
                    }
                } else {
                    if self.writer.write_bit(true).is_ok() {
                        self.state = DecoderState::Started;
                    }
                }
                Ok(())
            }
            DecoderState::WriteZeroFinish(count) => {
                if count > 0 {
                    let n = self.writer.write_zeros(count);
                    if n > 0 {
                        self.state = DecoderState::WriteZeroFinish(count - n);
                    }
                } else {
                    self.writer.write_trailing_zeros();
                    self.state = DecoderState::Finished;
                }
                Ok(())
            }
            DecoderState::WriteDataOne(count) => {
                if self.reader.len_bits() < (count as usize) {
                    self.state = DecoderState::Error;
                    return Err(DecoderError::UnexpectedEof);
                }
                if count > 0 {
                    let mut written = 0;
                    while written < count && self.writer.space_bits() > 0 {
                        if let Some(bit) = self.reader.read_bit() {
                            assert!(self.writer.write_bit(bit).is_ok());
                            written += 1;
                        } else {
                            break;
                        }
                    }
                    if written > 0 {
                        self.state = DecoderState::WriteDataOne(count - written);
                    }
                } else {
                    if self.writer.write_bit(true).is_ok() {
                        self.state = DecoderState::Started;
                    }
                }
                Ok(())
            }
            _ => unimplemented!()
        }
    }

    fn decode_command(&mut self) -> Result<DecoderState, DecoderError> {
        let state = if self.read_bit()? {
            DecoderState::WriteZeroOne(self.read_int(2)?)
        } else if self.read_bit()? {
            DecoderState::WriteZeroOne(self.read_int(5)?)
        } else if self.read_bit()? {
            DecoderState::WriteZeroOne(self.read_int(8)?)
        } else if self.read_bit()? {
            DecoderState::WriteDataOne(self.read_int(6)?)
        } else if self.read_bit()? {
            DecoderState::WriteZeroOne(self.read_int(23)?)
        } else {
            DecoderState::WriteZeroFinish(self.read_int(23)?)
        };
        Ok(state)
    }
}

#[derive(Debug, PartialEq)]
pub enum DecoderError {
    InvalidHeader,
    UnexpectedEof,
    InvalidState
}

#[derive(PartialEq)]
enum DecoderState {
    Initial,
    Started,
    WriteZeroOne(u32),
    WriteZeroFinish(u32),
    WriteDataOne(u32),
    Finished,
    Error,
}
