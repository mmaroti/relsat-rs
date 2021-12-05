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

use std::fmt;

#[derive(Debug)]
pub struct Buffer1 {
    data: Box<[u32]>,
    len: usize,
}

impl Buffer1 {
    const FILL: [u32; 2] = [0x00000000, 0xffffffff];
    const FORMAT: [char; 2] = ['0', '1'];

    pub fn new(len: usize, val: u32) -> Self {
        assert!(val <= 1);
        let val = Buffer1::FILL[val as usize];
        let data = vec![val; (len + 31) / 32].into_boxed_slice();
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
        self.data.fill(Buffer1::FILL[val as usize]);
    }
}

impl fmt::Display for Buffer1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"")?;
        for idx in 0..self.len() {
            let val = self.get(idx);
            write!(f, "{}", Buffer1::FORMAT[val as usize])?;
        }
        write!(f, "\"")
    }
}

#[derive(Debug)]
pub struct Buffer2 {
    data: Box<[u32]>,
    len: usize,
}

impl Buffer2 {
    const FILL: [u32; 4] = [0x00000000, 0x55555555, 0xaaaaaaaa, 0xffffffff];
    const FORMAT: [char; 4] = ['0', '1', '2', '3'];

    pub fn new(len: usize, val: u32) -> Self {
        assert!(val <= 3);
        let val = Buffer2::FILL[val as usize];
        let data = vec![val; (len + 15) / 16].into_boxed_slice();
        Self { data, len }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn get(&self, pos: usize) -> u32 {
        debug_assert!(pos < self.len);
        let data = self.data[pos / 16];
        let data = data >> (2 * (pos % 16));
        data & 3
    }

    #[inline(always)]
    pub fn set(&mut self, pos: usize, val: u32) {
        debug_assert!(pos < self.len && val <= 3);
        let mut data = self.data[pos / 16];
        data &= !(3 << (2 * (pos % 16)));
        data |= val << (2 * (pos % 16));
        self.data[pos / 16] = data;
    }

    #[inline(always)]
    pub fn fill(&mut self, val: u32) {
        debug_assert!(val <= 3);
        self.data.fill(Buffer2::FILL[val as usize]);
    }

    pub fn update<ITER>(&mut self, pattern: u32, other: &Self, iter: &mut ITER)
    where
        ITER: Iterator<Item = usize>,
    {
        let mut last = 0;
        for (pos1, pos2) in iter.enumerate() {
            let val = (self.get(pos1) << 3) | (other.get(pos2) << 1);
            let val = (pattern >> val) & 3;
            self.set(pos1, val);
            last = pos1 + 1;
        }
        debug_assert!(last == self.len);
    }

    /// Calculates the operation pattern. Each tuple corresponds to a
    /// possible input output combination. The first element is the
    /// original value, the second is the other value, and the third
    /// is the new value replacing the original one.
    pub const fn pattern(cases: &[(u32, u32, u32)]) -> u32 {
        let mut val = 0;
        let mut idx = 0;
        while idx < cases.len() {
            let (a, b, c) = cases[idx];
            assert!(a <= 3 && b <= 3 && c <= 3);
            let pos = (a << 3) | (b << 1);
            assert!(val & (3 << pos) == 0);
            val |= c << pos;
            idx += 1;
        }
        val
    }
}

struct Reader2<'a, ITER>
where
    ITER: Iterator<Item = usize>,
{
    iter: &'a mut ITER,
    buffer: &'a Buffer2,
}

impl<'a, ITER> Iterator for Reader2<'a, ITER>
where
    ITER: Iterator<Item = usize>,
{
    type Item = u32;

    #[inline(always)]
    #[allow(clippy::while_let_on_iterator)]
    fn next(&mut self) -> Option<u32> {
        match self.iter.next() {
            None => None,
            Some(pos) => {
                let mut val: u32 = self.buffer.get(pos);
                let mut idx = 2;
                while let Some(pos) = self.iter.next() {
                    val |= self.buffer.get(pos) << idx;
                    idx += 2;
                    if idx >= 32 {
                        break;
                    }
                }
                Some(val)
            }
        }
    }
}

impl fmt::Display for Buffer2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"")?;
        for idx in 0..self.len() {
            let val = self.get(idx);
            write!(f, "{}", Buffer2::FORMAT[val as usize])?;
        }
        write!(f, "\"")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn random(mut seed: u32, len: usize) -> Vec<u32> {
        assert!(seed != 0);
        let mut vec: Vec<u32> = Default::default();
        while vec.len() < len {
            let msb = (seed as i32) < 0;
            seed <<= 1;
            if msb {
                seed ^= 0x04c11db7;
            }
            vec.push(seed);
        }
        vec
    }

    #[test]
    fn buffer() {
        let vec = random(0x12345678, 11111);
        let mut buf1 = Buffer1::new(vec.len(), 0);
        let mut buf2 = Buffer2::new(vec.len(), 0);
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

    #[test]
    fn reader() {
        let data = random(0xf1234567, 200);
        let mut buf2 = Buffer2::new(data.len(), 0);
        for i in 0..data.len() {
            buf2.set(i, data[i] & 3);
        }
        for len in 0..data.len() {
            let mut iter = 0..len;
            let reader = Reader2 {
                iter: &mut iter,
                buffer: &buf2,
            };
            let out: Vec<u32> = reader.collect();
            assert_eq!(out.len(), (len + 15) / 16);
            for i in 0..(len + 15) / 16 {
                let mut val = 0;
                for j in 0..16 {
                    if i * 16 + j < len {
                        val |= buf2.get(i * 16 + j) << (2 * j);
                    }
                }
                assert_eq!(val, out[i]);
            }
        }
    }
}
