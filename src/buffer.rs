/*
* Copyright (C) 2019-2021, Miklos Maroti
*
* This program is free software: you can redistribute it and/or modify
* it under the terms of the GNU General Public License as published by
* the Free Software Foundation, either version 3 of the License, or
* (at your option) any later version.
*
* This program is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
* GNU General Public License for more details.
*
* You should have received a copy of the GNU General Public License
* along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

#[derive(Debug)]
pub struct Buffer1 {
    data: Box<[u32]>,
    len: usize,
}

impl Buffer1 {
    pub fn new(len: usize) -> Self {
        let data = vec![0; (len + 31) / 32].into_boxed_slice();
        Self { data, len }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get(&self, pos: usize) -> u32 {
        debug_assert!(pos < self.len);
        let data = self.data[pos / 32];
        let data = data >> (pos % 32);
        data & 1
    }

    pub fn set(&mut self, pos: usize, val: u32) {
        debug_assert!(pos < self.len && val <= 1);
        let mut data = self.data[pos / 32];
        data &= !(1 << (pos % 32));
        data |= val << (pos % 32);
        self.data[pos / 32] = data;
    }

    pub fn fill(&mut self, val: u32) {
        debug_assert!(val <= 1);
        const TABLE: [u32; 2] = [0x00000000, 0xffffffff];
        self.data.fill(TABLE[val as usize]);
    }
}

#[derive(Debug)]
pub struct Buffer2 {
    data: Box<[u32]>,
    len: usize,
}

impl Buffer2 {
    pub fn new(len: usize) -> Self {
        let data = vec![0; (len + 15) / 16].into_boxed_slice();
        Self { data, len }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get(&self, pos: usize) -> u32 {
        debug_assert!(pos < self.len);
        let data = self.data[pos / 16];
        let data = data >> (2 * (pos % 16));
        data & 3
    }

    pub fn set(&mut self, pos: usize, val: u32) {
        debug_assert!(pos < self.len && val <= 3);
        let mut data = self.data[pos / 16];
        data &= !(3 << (2 * (pos % 16)));
        data |= val << (2 * (pos % 16));
        self.data[pos / 16] = data;
    }

    pub fn fill(&mut self, val: u32) {
        debug_assert!(val <= 3);
        const TABLE: [u32; 4] = [0x00000000, 0x55555555, 0xaaaaaaaa, 0xffffffff];
        self.data.fill(TABLE[val as usize]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn random(len: usize) -> Vec<u32> {
        let mut num = 0xffffffff; // something nonzero
        let mut vec: Vec<u32> = Default::default();
        while vec.len() < len {
            let msb = (num as i32) < 0;
            num <<= 1;
            if msb {
                num ^= 0x04c11db7;
            }
            vec.push(num);
        }
        vec
    }

    #[test]
    fn buffer() {
        let vec = random(11111);
        let mut buf1 = Buffer1::new(vec.len());
        let mut buf2 = Buffer2::new(vec.len());
        for (i, a) in vec.iter().enumerate() {
            buf1.set(i, a & 1);
            buf2.set(i, a & 3);
        }
        for (i, a) in vec.iter().enumerate() {
            assert_eq!(buf1.get(i), a & 1);
            assert_eq!(buf2.get(i), a & 3);
        }

        buf1.fill(1);
        buf2.fill(1);
        for i in 0..vec.len() {
            assert_eq!(buf1.get(i), 1);
            assert_eq!(buf2.get(i), 1);
        }
    }
}
