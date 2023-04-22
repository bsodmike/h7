const DEFAULT_CHAR: char = 0u8 as char;

#[derive(Debug)]
pub enum InputBufferError {
    BufferFull,
}

#[derive(Debug)]
pub struct InputBuffer<const CHAR_BUFFER_SZ: usize>
where
    [(); CHAR_BUFFER_SZ * 4]:,
{
    char_idx: usize,
    char_buffer: [char; CHAR_BUFFER_SZ],
    byte_idx: usize,
    byte_buffer: [u8; CHAR_BUFFER_SZ * 4],
}

impl<const CHAR_BUFFER_SZ: usize> InputBuffer<CHAR_BUFFER_SZ>
where
    [(); CHAR_BUFFER_SZ * 4]:,
{
    pub const fn new() -> Self {
        Self {
            char_idx: 0,
            char_buffer: [DEFAULT_CHAR; CHAR_BUFFER_SZ],
            byte_idx: 0,
            byte_buffer: [0u8; CHAR_BUFFER_SZ * 4],
        }
    }

    pub fn push(&mut self, c: char) -> Result<(), InputBufferError> {
        if self.char_idx < self.char_buffer.len() {
            self.char_buffer[self.char_idx] = c;
            self.char_idx += 1;

            c.encode_utf8(&mut self.byte_buffer[self.byte_idx..]);
            self.byte_idx += c.len_utf8();

            Ok(())
        } else {
            Err(InputBufferError::BufferFull)
        }
    }

    pub fn push_str(&mut self, s: &str) -> Result<(), InputBufferError> {
        let chars = s.chars();
        for c in chars {
            self.push(c)?;
        }

        Ok(())
    }

    pub fn pop(&mut self) -> Option<char> {
        if self.char_idx > 0 {
            self.char_idx = self.char_idx.saturating_sub(1);
            let c = self.char_buffer[self.char_idx];
            self.byte_idx -= c.len_utf8();
            Some(c)
        } else {
            None
        }
    }

    pub fn pop_n(&mut self, count: usize) -> &[char] {
        let upper_bound = self.char_idx;
        let lower_bound = self.char_idx.saturating_sub(count);

        let n_bytes: usize = self.char_buffer[lower_bound..upper_bound]
            .iter()
            .map(|c| c.len_utf8())
            .sum();

        self.char_idx = lower_bound;
        self.byte_idx -= n_bytes;

        &self.char_buffer[lower_bound..upper_bound]
    }

    pub fn as_str(&self) -> &str {
        // SAFETY:
        //
        // Since we only can add valid chars and string slices, the byte buffer always
        // contain valid UTF-8
        unsafe { core::str::from_utf8_unchecked(&self.byte_buffer[..self.byte_idx]) }
    }

    pub const fn clear(&mut self) {
        self.char_idx = 0;
        self.byte_idx = 0;
    }
}

impl<const CHAR_BUFFER_SZ: usize> core::ops::Deref for InputBuffer<CHAR_BUFFER_SZ>
where
    [(); CHAR_BUFFER_SZ * 4]:,
{
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<const CHAR_BUFFER_SZ: usize> core::fmt::Display for InputBuffer<CHAR_BUFFER_SZ>
where
    [(); CHAR_BUFFER_SZ * 4]:,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
