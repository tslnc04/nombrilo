use std::{collections::HashMap, io::Read};

#[derive(Debug, Clone, Copy)]
pub enum TagType {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}

impl TagType {
    pub fn from(n: u8) -> TagType {
        match n {
            0 => TagType::End,
            1 => TagType::Byte,
            2 => TagType::Short,
            3 => TagType::Int,
            4 => TagType::Long,
            5 => TagType::Float,
            6 => TagType::Double,
            7 => TagType::ByteArray,
            8 => TagType::String,
            9 => TagType::List,
            10 => TagType::Compound,
            11 => TagType::IntArray,
            12 => TagType::LongArray,
            _ => panic!("Invalid tag type: {}", n),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Tag {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<Tag>),
    Compound(HashMap<String, Tag>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl Tag {
    pub fn parse_byte<F: Read>(file: &mut F) -> anyhow::Result<Tag> {
        let mut buf = [0; 1];
        file.read_exact(buf.as_mut())?;
        Ok(Tag::Byte(i8::from_be_bytes(buf)))
    }

    pub fn parse_short<F: Read>(file: &mut F) -> anyhow::Result<Tag> {
        let mut buf = [0; 2];
        file.read_exact(buf.as_mut())?;
        Ok(Tag::Short(i16::from_be_bytes(buf)))
    }

    pub fn parse_int<F: Read>(file: &mut F) -> anyhow::Result<Tag> {
        let mut buf = [0; 4];
        file.read_exact(buf.as_mut())?;
        Ok(Tag::Int(i32::from_be_bytes(buf)))
    }

    pub fn parse_long<F: Read>(file: &mut F) -> anyhow::Result<Tag> {
        let mut buf = [0; 8];
        file.read_exact(buf.as_mut())?;
        Ok(Tag::Long(i64::from_be_bytes(buf)))
    }

    pub fn parse_float<F: Read>(file: &mut F) -> anyhow::Result<Tag> {
        let mut buf = [0; 4];
        file.read_exact(buf.as_mut())?;
        Ok(Tag::Float(f32::from_be_bytes(buf)))
    }

    pub fn parse_double<F: Read>(file: &mut F) -> anyhow::Result<Tag> {
        let mut buf = [0; 8];
        file.read_exact(buf.as_mut())?;
        Ok(Tag::Double(f64::from_be_bytes(buf)))
    }

    pub fn parse_byte_array<F: Read>(file: &mut F) -> anyhow::Result<Tag> {
        let mut buf = [0; 4];
        file.read_exact(buf.as_mut())?;
        let len = usize::try_from(i32::from_be_bytes(buf))?;
        let mut buf = vec![0; len];
        file.read_exact(buf.as_mut())?;
        Ok(Tag::ByteArray(buf.iter().map(|&x| i8::from_be_bytes([x])).collect()))
    }

    pub fn parse_string<F: Read>(file: &mut F) -> anyhow::Result<Tag> {
        let mut buf = [0; 2];
        file.read_exact(buf.as_mut())?;
        let len = u16::from_be_bytes(buf) as usize;
        let mut buf = vec![0; len];
        file.read_exact(buf.as_mut())?;
        Ok(Tag::String(String::from_utf8(buf)?))
    }

    pub fn parse_list<F: Read>(file: &mut F) -> anyhow::Result<Tag> {
        let mut buf = [0; 1];
        file.read_exact(buf.as_mut())?;
        let tag_type = TagType::from(buf[0]);
        let mut buf = [0; 4];
        file.read_exact(buf.as_mut())?;
        let len = usize::try_from(i32::from_be_bytes(buf))?;

        let mut tags = Vec::new();
        for _ in 0..len {
            tags.push(Tag::parse_unnamed(file, tag_type)?);
        }
        Ok(Tag::List(tags))
    }

    pub fn parse_compound<F: Read>(file: &mut F) -> anyhow::Result<Tag> {
        let mut tags = HashMap::new();

        while let Ok((name, tag)) = Tag::parse_fully_formed(file) {
            if let Tag::End = tag {
                break;
            }
            tags.insert(name, tag);
        }
        
        Ok(Tag::Compound(tags))
    }

    pub fn parse_int_array<F: Read>(file: &mut F) -> anyhow::Result<Tag> {
        let mut buf = [0; 4];
        file.read_exact(buf.as_mut())?;
        let len = usize::try_from(i32::from_be_bytes(buf))?;
        let mut buf = vec![0; len * 4];
        file.read_exact(buf.as_mut())?;
        let ints = buf.chunks_exact(4).map(|x| i32::from_be_bytes([x[0], x[1], x[2], x[3]])).collect::<Vec<_>>();
        
        Ok(Tag::IntArray(ints))
    }

    pub fn parse_long_array<F: Read>(file: &mut F) -> anyhow::Result<Tag> {
        let mut buf = [0; 4];
        file.read_exact(buf.as_mut())?;
        let len = usize::try_from(i32::from_be_bytes(buf))?;
        let mut buf = vec![0; len * 8];
        file.read_exact(buf.as_mut())?;
        let longs = buf.chunks_exact(8).map(|x| i64::from_be_bytes([x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7]])).collect::<Vec<_>>();
        
        Ok(Tag::LongArray(longs))
    }

    pub fn parse_unnamed<F: Read>(file: &mut F, tag_type: TagType) -> anyhow::Result<Tag> {
        match tag_type {
            TagType::End => Ok(Tag::End),
            TagType::Byte => Tag::parse_byte(file),
            TagType::Short => Tag::parse_short(file),
            TagType::Int => Tag::parse_int(file),
            TagType::Long => Tag::parse_long(file),
            TagType::Float => Tag::parse_float(file),
            TagType::Double => Tag::parse_double(file),
            TagType::ByteArray => Tag::parse_byte_array(file),
            TagType::String => Tag::parse_string(file),
            TagType::List => Tag::parse_list(file),
            TagType::Compound => Tag::parse_compound(file),
            TagType::IntArray => Tag::parse_int_array(file),
            TagType::LongArray => Tag::parse_long_array(file),
        }
    }

    pub fn parse_fully_formed<F: Read>(file: &mut F) -> anyhow::Result<(String, Tag)> {
        let mut buf = [0; 1];
        file.read_exact(buf.as_mut())?;
        let tag_type = TagType::from(buf[0]);
        if let TagType::End = tag_type {
            return Ok((String::new(), Tag::End));
        }

        let mut buf = [0; 2];
        file.read_exact(buf.as_mut())?;
        let len = u16::from_be_bytes(buf) as usize;
        let mut buf = vec![0; len];
        file.read_exact(buf.as_mut())?;
        let name = String::from_utf8(buf)?;

        let tag = Tag::parse_unnamed(file, tag_type)?;
        Ok((name, tag))
    }
}
