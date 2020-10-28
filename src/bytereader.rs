use std::str;
use std::convert::{TryInto};

pub struct ByteReader {
    response: Vec<u8>,
    pub iterator: usize
}

impl ByteReader {
    pub fn new(data: Vec<u8>) -> Self {
        ByteReader {
            response: data,
            iterator: 0, 
        }
    }

    pub fn remaining(&mut self) -> usize {
        self.response.len() - self.iterator
    }

    pub fn get_bytes(&mut self, length: usize) -> &[u8] {
        self.check_byte_size(length);

        let value = &self.response[self.iterator..self.iterator+length];
        
        self.iterator += length;
        
        value
    }

    pub fn get_byte(&mut self) -> u8 {
        let byte_size = 1;
        
        self.check_byte_size(byte_size);

        let value = self.response[self.iterator];
        
        self.iterator += byte_size;
        
        value
    }

    pub fn get_float(&mut self) -> f32 {
        let byte_size = 4;
        
        self.check_byte_size(byte_size);
        
        let value = f32::from_ne_bytes(
            (&self.response[self.iterator..self.iterator+byte_size])
                .try_into()
                .expect("Slice with incorrect length")
        );
        
        self.iterator += byte_size;
        
        value
    }

    pub fn get_short(&mut self) -> i16 {
        let byte_size = 2;      
        
        self.check_byte_size(byte_size);
        
        let value = i16::from_ne_bytes(
            (&self.response[self.iterator..self.iterator+byte_size])
                .try_into()
                .expect("Slice with incorrect length")
        );
        
        self.iterator += byte_size;
        
        value
    }

    pub fn get_int(&mut self) -> i32 {
        let byte_size = 4;
        
        self.check_byte_size(byte_size);
        
        let value = i32::from_ne_bytes(
            (&self.response[self.iterator..self.iterator+byte_size])
                .try_into()
                .expect("Slice with incorrect length")
        );
        
        self.iterator += byte_size;
        
        value
    }

    pub fn get_long(&mut self) -> u32 {
        let byte_size = 4;
        
        self.check_byte_size(byte_size);
        
        let value = u32::from_ne_bytes(
            (&self.response[self.iterator..self.iterator+byte_size])
                .try_into()
                .expect("Slice with incorrect length")
        );
        
        self.iterator += byte_size;
        
        value
    }

    pub fn get_string(&mut self) -> String {
        let byte_size = 1;
        
        self.check_byte_size(byte_size);
        
        let pos = self.get_next_null_char_pos();                
        let val = str::from_utf8(&self.response[self.iterator..pos]).unwrap();
        
        self.iterator = pos + 1;
        
        String::from(val)
    }

    pub fn peek_remaining_bytes(&mut self) -> &[u8] {
        if self.iterator >= self.response.len() {
            panic!("out of range")
        } else {
            &self.response[self.iterator..]
        }        
    }

    fn get_next_null_char_pos(&self) -> usize {
        let data = &self.response[..];
        let mut found: bool = false;
        let mut pos: usize = 0;

        for i in self.iterator..data.len() {
            if data[i] == 0x00 {
                pos = i;
                found = true;
                break;
            }
        }
        
        if !found {
            panic!("Didn't find a null char in the remaining bytes");
        } else {
            pos
        }
    }

    fn check_byte_size(&mut self, byte_size: usize) {
        let remaining = self.remaining();
        
        if remaining < byte_size {
            panic!("out of range, tried to access {} bytes, but only {} bytes remaining", byte_size, remaining);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_byte() {
        let data = vec![0x6a];

        let mut reader = ByteReader::new(data);

        assert_eq!(0x6a, reader.get_byte());
    }

    #[test]
    #[should_panic]
    fn test_get_byte_out_or_range() {
        let data = vec![0x6a];
        let mut reader = ByteReader::new(data);

        reader.get_byte();
        reader.get_byte();
    }

    #[test]
    fn test_get_bytes() {
        let data = vec![0x6a, 0x6a, 0x6a];

        let mut reader = ByteReader::new(data);

        assert_eq!([0x6a, 0x6a, 0x6a], reader.get_bytes(3));
    }

    #[test]
    #[should_panic]
    fn test_get_bytes_out_of_range() {
        let data = vec![0x6a, 0x6a, 0x6a];

        let mut reader = ByteReader::new(data);

        reader.get_bytes(4);
    }

    #[test]
    fn test_get_short() {
        let data = [i16::to_ne_bytes(6230), i16::to_ne_bytes(1234)].concat();
        
        data.iter().for_each(|b| println!("{}", b));

        let mut reader = ByteReader::new(data);

        assert_eq!(6230, reader.get_short());
        assert_eq!(1234, reader.get_short());
    }

    #[test]
    fn test_get_int() {
        let data = i32::to_ne_bytes(655556).to_vec();

        let mut reader = ByteReader::new(data);

        assert_eq!(655556, reader.get_int());        
    }

    #[test]
    fn test_get_long() {
        let data = u64::to_ne_bytes(655556000).to_vec();

        let mut reader = ByteReader::new(data);

        assert_eq!(655556000u32, reader.get_long());
    }

    #[test]
    fn test_get_float() {
        let data = f32::to_ne_bytes(132.34).to_vec();

        data.iter().for_each(|x| println!("{}", x));
        
        let mut reader = ByteReader::new(data);

        assert_eq!(132.34, reader.get_float());        
    }

    #[test]
    fn test_get_string() {
        let mut reader = ByteReader::new(
            vec![
                0x6a, 0x69, 0x6d, 0x6d, 
                0x79, 0x00, 0x73, 0x6f, 
                0x72, 0x74, 0x69, 0x6e, 
                0x67, 0x00, 0x67, 0x6f, 
                0x72, 0x61, 0x6e, 0x00 ]);
        
        assert_eq!("jimmy", reader.get_string());
        assert_eq!("sorting", reader.get_string());
        assert_eq!("goran", reader.get_string());
    }

    #[test]
    #[should_panic]
    fn test_get_string_out_of_range_should_panic() {
        let mut reader = ByteReader::new(
            vec![
                0x6a, 0x69, 0x6d, 0x6d, 
                0x79, 0x00, 0x73, 0x6f, 
                0x72, 0x74, 0x69, 0x6e, 
                0x67, 0x00, 0x67, 0x6f, 
                0x72, 0x61, 0x6e, 0x00 ]);
        
        reader.get_string();
        reader.get_string();
        reader.get_string();
        reader.get_string(); // out of range
    }

    #[test]
    #[should_panic]
    fn test_get_string_with_missing_null_char_should_panic() {
        let mut reader = ByteReader::new(
            vec![
                0x6a, 0x69, 0x6d, 0x6d, 
                0x79, 0x73, 0x6f, 
                0x72, 0x74, 0x69, 
                0x6e, 0x67, 0x67, 
                0x6f, 0x72, 0x61, 
                0x6e ]);
        
        reader.get_string();
        reader.get_string();
        reader.get_string();
        reader.get_string(); // out of range
    }

    #[test]
    fn test_peek_remaining_bytes() {
        let mut reader = ByteReader::new(
            vec![
                0x6a, 0x69, 0x6d, 0x6d, 
                0x79, 0x00, 0x73, 0x6f, 
                0x72, 0x74, 0x69, 0x6e, 
                0x67, 0x00, 0x67, 0x6f, 
                0x72, 0x61, 0x6e, 0x00,
                0x72, 0x61, 0x6e, 0x00,
                0x72, 0x61, 0x6e, 0x00 ]);
        
        reader.get_string(); // 6
        reader.get_short(); // 6 + 2 = 8
        reader.get_int(); // 8 + 4 = 12
        reader.get_long(); // 12 + 4 = 16
        reader.get_byte(); // 16 + 1 = 17
        reader.get_bytes(3); // 17 + 3 = 20 - 8 bytes remaining

        assert_eq!([0x72, 0x61, 0x6e, 0x00, 0x72, 0x61, 0x6e, 0x00], reader.peek_remaining_bytes());
    }

    #[test]
    #[should_panic]
    fn test_peek_remaining_bytes_out_of_range_should_panic() {
        let mut reader = ByteReader::new(
            vec![
                0x6a, 0x69, 0x6d, 0x6d, 
                0x79, 0x00, 0x73, 0x6f, 
                0x72, 0x74, 0x69, 0x6e, 
                0x67, 0x00, 0x67, 0x6f, 
                0x72, 0x61, 0x6e, 0x00,
                0x72, 0x61, 0x6e, 0x00,
                0x72, 0x61, 0x6e, 0x00 ]);
        
        reader.iterator = 28;

        reader.peek_remaining_bytes();
    }
}