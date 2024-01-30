use std::io::{self, Write};

use crate::nbt::TagType;

pub(super) trait Formatter {
    fn write_bool<W>(&mut self, writer: &mut W, value: bool) -> io::Result<()>
    where
        W: Write;
    fn write_byte<W>(&mut self, writer: &mut W, value: i8) -> io::Result<()>
    where
        W: Write;
    fn write_short<W>(&mut self, writer: &mut W, value: i16) -> io::Result<()>
    where
        W: Write;
    fn write_int<W>(&mut self, writer: &mut W, value: i32) -> io::Result<()>
    where
        W: Write;
    fn write_long<W>(&mut self, writer: &mut W, value: i64) -> io::Result<()>
    where
        W: Write;
    fn write_float<W>(&mut self, writer: &mut W, value: f32) -> io::Result<()>
    where
        W: Write;
    fn write_double<W>(&mut self, writer: &mut W, value: f64) -> io::Result<()>
    where
        W: Write;
    fn write_byte_array<W>(&mut self, writer: &mut W, value: &[u8]) -> io::Result<()>
    where
        W: Write;
    fn write_string<W>(&mut self, writer: &mut W, value: &str) -> io::Result<()>
    where
        W: Write;

    fn start_byte_array<W>(&mut self, writer: &mut W, len: i32) -> io::Result<()>
    where
        W: Write;
    fn start_int_array<W>(&mut self, writer: &mut W, len: i32) -> io::Result<()>
    where
        W: Write;
    fn start_long_array<W>(&mut self, writer: &mut W, len: i32) -> io::Result<()>
    where
        W: Write;
    fn start_list<W>(&mut self, writer: &mut W, len: i32, element_type: TagType) -> io::Result<()>
    where
        W: Write;
    fn start_element<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: Write;
    fn end_sequence<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write;

    fn start_compound<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write;
    fn end_compound<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write;

    fn start_entry<W>(&mut self, writer: &mut W, key: &[u8], value_type: TagType) -> io::Result<()>
    where
        W: Write;
    fn end_entry<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write;
}

pub(super) struct StringifiedFormatter {
    first: Vec<bool>,
}

impl StringifiedFormatter {
    pub fn new() -> Self {
        Self { first: Vec::new() }
    }
}

impl Formatter for StringifiedFormatter {
    fn write_bool<W>(&mut self, writer: &mut W, value: bool) -> io::Result<()>
    where
        W: Write,
    {
        write!(writer, "{}", if value { "true" } else { "false" })
    }

    fn write_byte<W>(&mut self, writer: &mut W, value: i8) -> io::Result<()>
    where
        W: Write,
    {
        write!(writer, "{}b", value)
    }

    fn write_short<W>(&mut self, writer: &mut W, value: i16) -> io::Result<()>
    where
        W: Write,
    {
        write!(writer, "{}s", value)
    }

    fn write_int<W>(&mut self, writer: &mut W, value: i32) -> io::Result<()>
    where
        W: Write,
    {
        write!(writer, "{}", value)
    }

    fn write_long<W>(&mut self, writer: &mut W, value: i64) -> io::Result<()>
    where
        W: Write,
    {
        write!(writer, "{}l", value)
    }

    fn write_float<W>(&mut self, writer: &mut W, value: f32) -> io::Result<()>
    where
        W: Write,
    {
        write!(writer, "{}f", value)
    }

    fn write_double<W>(&mut self, writer: &mut W, value: f64) -> io::Result<()>
    where
        W: Write,
    {
        write!(writer, "{}d", value)
    }

    fn write_byte_array<W: Write>(&mut self, writer: &mut W, value: &[u8]) -> io::Result<()> {
        write!(writer, "[B;")?;
        for (i, byte) in value.iter().enumerate() {
            if i != 0 {
                write!(writer, ",")?;
            }
            write!(writer, "{}b", byte)?;
        }
        write!(writer, "]")
    }

    fn write_string<W>(&mut self, writer: &mut W, value: &str) -> io::Result<()>
    where
        W: Write,
    {
        write!(writer, "\"{}\"", value.escape_debug())
    }

    fn start_byte_array<W>(&mut self, writer: &mut W, _len: i32) -> io::Result<()>
    where
        W: Write,
    {
        self.first.push(true);
        write!(writer, "[B;")
    }

    fn start_int_array<W>(&mut self, writer: &mut W, _len: i32) -> io::Result<()>
    where
        W: Write,
    {
        self.first.push(true);
        write!(writer, "[I;")
    }

    fn start_long_array<W>(&mut self, writer: &mut W, _len: i32) -> io::Result<()>
    where
        W: Write,
    {
        self.first.push(true);
        write!(writer, "[L;")
    }

    fn start_list<W>(&mut self, writer: &mut W, _len: i32, _element_type: TagType) -> io::Result<()>
    where
        W: Write,
    {
        self.first.push(true);
        write!(writer, "[")
    }

    fn start_element<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: Write,
    {
        if !first {
            write!(writer, ",")
        } else {
            Ok(())
        }
    }

    fn end_sequence<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.first.pop();
        write!(writer, "]")
    }

    fn start_compound<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.first.push(true);
        write!(writer, "{{")
    }
    fn end_compound<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.first.pop();
        write!(writer, "}}")
    }

    fn start_entry<W>(&mut self, writer: &mut W, key: &[u8], _value_type: TagType) -> io::Result<()>
    where
        W: Write,
    {
        if let Some(first) = self.first.last_mut() {
            if !*first {
                write!(writer, ",")?;
            } else {
                *first = false;
            }
        }
        writer.write_all(key)?;
        write!(writer, ":")
    }

    fn end_entry<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        Ok(())
    }
}

pub(super) struct BinaryFormatter {
    top_level: bool,
}

impl BinaryFormatter {
    pub fn new() -> Self {
        Self { top_level: true }
    }
}

impl Formatter for BinaryFormatter {
    fn write_bool<W>(&mut self, writer: &mut W, value: bool) -> io::Result<()>
    where
        W: Write,
    {
        writer.write_all(&[if value { 1 } else { 0 }])
    }

    fn write_byte<W>(&mut self, writer: &mut W, value: i8) -> io::Result<()>
    where
        W: Write,
    {
        writer.write_all(&[value as u8])
    }

    fn write_short<W>(&mut self, writer: &mut W, value: i16) -> io::Result<()>
    where
        W: Write,
    {
        writer.write_all(&value.to_be_bytes())
    }

    fn write_int<W>(&mut self, writer: &mut W, value: i32) -> io::Result<()>
    where
        W: Write,
    {
        writer.write_all(&value.to_be_bytes())
    }

    fn write_long<W>(&mut self, writer: &mut W, value: i64) -> io::Result<()>
    where
        W: Write,
    {
        writer.write_all(&value.to_be_bytes())
    }

    fn write_float<W>(&mut self, writer: &mut W, value: f32) -> io::Result<()>
    where
        W: Write,
    {
        writer.write_all(&value.to_be_bytes())
    }

    fn write_double<W>(&mut self, writer: &mut W, value: f64) -> io::Result<()>
    where
        W: Write,
    {
        writer.write_all(&value.to_be_bytes())
    }

    fn write_byte_array<W>(&mut self, writer: &mut W, value: &[u8]) -> io::Result<()>
    where
        W: Write,
    {
        writer.write_all(&(value.len() as i32).to_be_bytes())?;
        writer.write_all(value)
    }

    fn write_string<W>(&mut self, writer: &mut W, value: &str) -> io::Result<()>
    where
        W: Write,
    {
        // the serializer already checks that the length fits into u16
        writer.write_all(&(value.len() as u16).to_be_bytes())?;
        writer.write_all(cesu8::to_java_cesu8(value).as_ref())
    }

    fn start_byte_array<W>(&mut self, writer: &mut W, len: i32) -> io::Result<()>
    where
        W: Write,
    {
        writer.write_all(&len.to_be_bytes())
    }

    fn start_int_array<W>(&mut self, writer: &mut W, len: i32) -> io::Result<()>
    where
        W: Write,
    {
        writer.write_all(&len.to_be_bytes())
    }

    fn start_long_array<W>(&mut self, writer: &mut W, len: i32) -> io::Result<()>
    where
        W: Write,
    {
        writer.write_all(&len.to_be_bytes())
    }

    fn start_list<W>(&mut self, writer: &mut W, len: i32, element_type: TagType) -> io::Result<()>
    where
        W: Write,
    {
        if self.top_level {
            // Minecraft only generates files with a Compound or List at the top
            // level, so assume this is a List and specify the tag type and name
            // it the empty string.
            self.top_level = false;
            writer.write_all(&[TagType::List as u8, 0, 0])?;
        }

        writer.write_all(&[element_type as u8])?;
        writer.write_all(&len.to_be_bytes())
    }

    fn start_element<W>(&mut self, _writer: &mut W, _first: bool) -> io::Result<()>
    where
        W: Write,
    {
        Ok(())
    }

    fn end_sequence<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        Ok(())
    }

    fn start_compound<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        if self.top_level {
            // Minecraft only generates files with a Compound or List at the top
            // level, so assume this is a Compound and specify the tag type and
            // name it the empty string.
            self.top_level = false;
            writer.write_all(&[TagType::Compound as u8, 0, 0])
        } else {
            Ok(())
        }
    }

    fn end_compound<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        writer.write_all(&[TagType::End as u8])
    }

    fn start_entry<W>(&mut self, writer: &mut W, key: &[u8], value_type: TagType) -> io::Result<()>
    where
        W: Write,
    {
        writer.write_all(&[value_type as u8])?;
        writer.write_all(key)
    }

    fn end_entry<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        Ok(())
    }
}
