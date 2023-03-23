// The RAM for the CPU memory map is only 0x800 in size, however
// the addresses 0x0 - 0x1FFF can be addressed. Only the 11 least significant
// digits are used for access.
// Ref: https://www.nesdev.org/wiki/CPU_memory_map

const INDEX_MASK: u16 = (0b1 << 11) -1;

pub struct RAM {
    memory_map: [u8; 0x800]
}

impl RAM {
    pub fn new() -> Self {
        RAM {
            memory_map: [0; 0x800],
        }
    }

    pub fn write(&mut self, index: u16, value: u8) {
        if index > 0x1FFF {
            panic!("Attempted write to RAM out of bounds at {}", index);
        }
        self.memory_map[(index & INDEX_MASK) as usize] = value
    }

    pub fn read(&self, index: u16) -> u8{
        if index > 0x1FFF {
            panic!("Attemped read from RAM out of bounds at {}", index);
        }
        self.memory_map[(index & INDEX_MASK) as usize]
    }
}


#[cfg(test)]
mod tests {
    use std::panic::catch_unwind;

    use super::*;

    #[test]
    fn test_all_index_valid() {
        let mut ram = RAM::new();
        // populate with some data
        for i in 0..0x800u16 {
            ram.write(i, (i / 8) as u8);
        }

        for i in 0..0x800u16 {
            assert_eq!((i / 8) as u8, ram.read(i));
            assert_eq!((i / 8) as u8, ram.read(i + 0x800));
            assert_eq!((i / 8) as u8, ram.read(i + 0x1000));
            assert_eq!((i / 8) as u8, ram.read(i + 0x1800));
        }
    }

    #[test]
    fn test_index_out_of_bounds() {
        let mut ram = RAM::new();
        assert_eq!(0, ram.read(0x1FFF));
        assert!(catch_unwind(|| ram.read(0x2000)).is_err());
    }

}