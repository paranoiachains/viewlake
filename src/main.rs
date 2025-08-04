#![no_std]
#![no_main]
#![windows_subsystem = "console"]
use viewlake::*;

use windows_sys::Win32::System::Threading::ExitProcess;
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
fn mainCRTStartup() -> ! {
    unsafe {
        w!("asdasd");
        ExitProcess(0);
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PCWSTR(pub *const u16);

pub const fn decode_utf8_char(bytes: &[u8], mut pos: usize) -> Option<(u32, usize)> {
    if bytes.len() == pos {
        return None;
    }
    let ch = bytes[pos] as u32;
    pos += 1;
    if ch <= 0x7f {
        return Some((ch, pos));
    }
    if (ch & 0xe0) == 0xc0 {
        if bytes.len() - pos < 1 {
            return None;
        }
        let ch2 = bytes[pos] as u32;
        pos += 1;
        if (ch2 & 0xc0) != 0x80 {
            return None;
        }
        let result: u32 = ((ch & 0x1f) << 6) | (ch2 & 0x3f);
        if result <= 0x7f {
            return None;
        }
        return Some((result, pos));
    }
    if (ch & 0xf0) == 0xe0 {
        if bytes.len() - pos < 2 {
            return None;
        }
        let ch2 = bytes[pos] as u32;
        pos += 1;
        let ch3 = bytes[pos] as u32;
        pos += 1;
        if (ch2 & 0xc0) != 0x80 || (ch3 & 0xc0) != 0x80 {
            return None;
        }
        let result = ((ch & 0x0f) << 12) | ((ch2 & 0x3f) << 6) | (ch3 & 0x3f);
        if result <= 0x7ff || (0xd800 <= result && result <= 0xdfff) {
            return None;
        }
        return Some((result, pos));
    }
    if (ch & 0xf8) == 0xf0 {
        if bytes.len() - pos < 3 {
            return None;
        }
        let ch2 = bytes[pos] as u32;
        pos += 1;
        let ch3 = bytes[pos] as u32;
        pos += 1;
        let ch4 = bytes[pos] as u32;
        pos += 1;
        if (ch2 & 0xc0) != 0x80 || (ch3 & 0xc0) != 0x80 || (ch4 & 0xc0) != 0x80 {
            return None;
        }
        let result =
            ((ch & 0x07) << 18) | ((ch2 & 0x3f) << 12) | ((ch3 & 0x3f) << 6) | (ch4 & 0x3f);
        if result <= 0xffff || 0x10ffff < result {
            return None;
        }
        return Some((result, pos));
    }
    None
}

pub const fn utf16_len(bytes: &[u8]) -> usize {
    let mut pos = 0;
    let mut len = 0;
    while let Some((code_point, new_pos)) = decode_utf8_char(bytes, pos) {
        pos = new_pos;
        len += if code_point <= 0xffff { 1 } else { 2 };
    }
    len
}

impl PCWSTR {
    /// Construct a new `PCWSTR` from a raw pointer
    pub const fn from_raw(ptr: *const u16) -> Self {
        Self(ptr)
    }

    /// Construct a null `PCWSTR`
    pub const fn null() -> Self {
        Self(core::ptr::null())
    }

    /// Returns a raw pointer to the `PCWSTR`
    pub const fn as_ptr(&self) -> *const u16 {
        self.0
    }

    /// Checks whether the `PCWSTR` is null
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    /// String length without the trailing 0
    ///
    /// # Safety
    ///
    /// The `PCWSTR`'s pointer needs to be valid for reads up until and including the next `\0`.
    pub unsafe fn len(&self) -> usize {
        unsafe extern "C" {
            fn wcslen(s: *const u16) -> usize;
        }
        unsafe { wcslen(self.0) }
    }

    /// Returns `true` if the string length is zero, and `false` otherwise.
    ///
    /// # Safety
    ///
    /// The `PCWSTR`'s pointer needs to be valid for reads up until and including the next `\0`.
    pub unsafe fn is_empty(&self) -> bool {
        unsafe { self.len() == 0 }
    }

    /// String data without the trailing 0
    ///
    /// # Safety
    ///
    /// The `PCWSTR`'s pointer needs to be valid for reads up until and including the next `\0`.
    pub unsafe fn as_wide(&self) -> &[u16] {
        unsafe { core::slice::from_raw_parts(self.0, self.len()) }
    }

    /// Allow this string to be displayed.
    ///
    /// # Safety
    ///
    /// See the safety information for `PCWSTR::as_wide`.
    pub unsafe fn display(&self) -> impl core::fmt::Display + '_ {
        unsafe { Decode(move || core::char::decode_utf16(self.as_wide().iter().cloned())) }
    }
}

impl Default for PCWSTR {
    fn default() -> Self {
        Self::null()
    }
}

impl AsRef<PCWSTR> for PCWSTR {
    fn as_ref(&self) -> &Self {
        self
    }
}

/// An internal helper for decoding an iterator of chars and displaying them
pub struct Decode<F>(pub F);

impl<F, R, E> core::fmt::Display for Decode<F>
where
    F: Clone + FnOnce() -> R,
    R: IntoIterator<Item = core::result::Result<char, E>>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::fmt::Write;
        let iter = self.0.clone();
        for c in iter().into_iter() {
            f.write_char(c.unwrap_or(core::char::REPLACEMENT_CHARACTER))?
        }
        Ok(())
    }
}

/// Mirror of `std::char::decode_utf16` for utf-8.
pub fn decode_utf8(
    mut buffer: &[u8],
) -> impl Iterator<Item = core::result::Result<char, core::str::Utf8Error>> + '_ {
    let mut current = "".chars();
    let mut previous_error = None;
    core::iter::from_fn(move || {
        loop {
            match (current.next(), previous_error) {
                (Some(c), _) => return Some(Ok(c)),
                // Return the previous error
                (None, Some(e)) => {
                    previous_error = None;
                    return Some(Err(e));
                }
                // We're completely done
                (None, None) if buffer.is_empty() => return None,
                (None, None) => {
                    match core::str::from_utf8(buffer) {
                        Ok(s) => {
                            current = s.chars();
                            buffer = &[];
                        }
                        Err(e) => {
                            let (valid, rest) = buffer.split_at(e.valid_up_to());
                            // Skip the invalid sequence and stop completely if we ended early
                            let invalid_sequence_length = e.error_len()?;
                            buffer = &rest[invalid_sequence_length..];

                            // Set the current iterator to the valid section and indicate previous error
                            // SAFETY: `valid` is known to be valid utf-8 from error
                            current = unsafe { core::str::from_utf8_unchecked(valid) }.chars();
                            previous_error = Some(e);
                        }
                    }
                }
            }
        }
    })
}

#[macro_export]
macro_rules! w {
    ($s:literal) => {{
        const INPUT: &[u8] = $s.as_bytes();
        const OUTPUT_LEN: usize = $crate::utf16_len(INPUT) + 1;
        const OUTPUT: &[u16; OUTPUT_LEN] = {
            let mut buffer = [0; OUTPUT_LEN];
            let mut input_pos = 0;
            let mut output_pos = 0;
            while let Some((mut code_point, new_pos)) = $crate::decode_utf8_char(INPUT, input_pos) {
                input_pos = new_pos;
                if code_point <= 0xffff {
                    buffer[output_pos] = code_point as u16;
                    output_pos += 1;
                } else {
                    code_point -= 0x10000;
                    buffer[output_pos] = 0xd800 + (code_point >> 10) as u16;
                    output_pos += 1;
                    buffer[output_pos] = 0xdc00 + (code_point & 0x3ff) as u16;
                    output_pos += 1;
                }
            }
            &{ buffer }
        };
        $crate::PCWSTR::from_raw(OUTPUT.as_ptr())
    }};
}
