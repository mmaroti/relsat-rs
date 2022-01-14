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

//! Structures for working with 1-bit and 2-bit vectors.

use std::ops::Range;

use super::bitops::{Bit1, Bit2, Op222};

/// A vector for holding single bits represented as 0 or 1.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Buffer1 {
    data: Vec<u32>,
    len: usize,
}

impl Buffer1 {
    const FILL: [u32; 2] = [0x00000000, 0xffffffff];

    pub fn new(len: usize, val: Bit1) -> Self {
        let fill = Buffer1::FILL[val.idx() as usize];
        let data = vec![fill; (len + 31) / 32];
        Self { data, len }
    }

    pub fn append(&mut self, len: usize, val: Bit1) {
        let fill = Buffer1::FILL[val.idx() as usize];
        if self.len % 32 != 0 {
            let mask = (1 << (self.len % 32)) - 1;
            let val = self.data.last_mut().unwrap();
            *val = (*val & mask) | (fill & !mask);
        }
        self.len += len;
        self.data.resize((self.len + 31) / 32, fill);
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn get(&self, pos: usize) -> Bit1 {
        debug_assert!(pos < self.len);
        let data = self.data[pos / 32];
        let data = data >> (pos % 32);
        Bit1::new(data & 1)
    }

    #[inline(always)]
    pub fn set(&mut self, pos: usize, val: Bit1) {
        debug_assert!(pos < self.len);
        let mut data = self.data[pos / 32];
        data &= !(1 << (pos % 32));
        data |= val.idx() << (pos % 32);
        self.data[pos / 32] = data;
    }

    #[inline(always)]
    pub fn fill(&mut self, val: Bit1) {
        let fill = Buffer1::FILL[val.idx() as usize];
        self.data.fill(fill);
    }

    pub fn fill_range(&mut self, range: Range<usize>, val: Bit1) {
        debug_assert!(range.start <= range.end && range.end <= self.len);

        let fill = Buffer1::FILL[val.idx() as usize];
        if range.start >= range.end {
        } else if range.start / 32 == range.end / 32 {
            let mask = (1 << (range.start % 32)) - 1;
            let mask = mask ^ ((1 << (range.end % 32)) - 1);
            let temp = self.data[range.start / 32];
            let temp = (temp & !mask) | (fill & mask);
            self.data[range.start / 32] = temp;
        } else {
            let mask = (1 << (range.start % 32)) - 1;
            let temp = self.data[range.start / 32];
            let temp = (temp & mask) | (fill & !mask);
            self.data[range.start / 32] = temp;

            self.data[(range.start / 32 + 1)..(range.end / 32)].fill(fill);

            if range.end % 32 != 0 {
                let mask = (1 << (range.end % 32)) - 1;
                let temp = self.data[range.end / 32];
                let temp = (temp & !mask) | (fill & mask);
                self.data[range.end / 32] = temp;
            }
        }
    }
}

/// A vector for holding double bits represented as 0, 1, 2 or 3.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Buffer2 {
    data: Vec<u32>,
    len: usize,
}

impl Buffer2 {
    const FILL: [u32; 4] = [0x00000000, 0x55555555, 0xaaaaaaaa, 0xffffffff];

    pub fn new(len: usize, val: Bit2) -> Self {
        let fill = Buffer2::FILL[val.idx() as usize];
        let data = vec![fill; (len + 15) / 16];
        Self { data, len }
    }

    pub fn append(&mut self, len: usize, val: Bit2) {
        let fill = Buffer2::FILL[val.idx() as usize];
        if self.len % 16 != 0 {
            let mask = (1 << (2 * (self.len % 16))) - 1;
            let val = self.data.last_mut().unwrap();
            *val = (*val & mask) | (fill & !mask);
        }
        self.len += len;
        self.data.resize((self.len + 15) / 16, fill);
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn get(&self, pos: usize) -> Bit2 {
        debug_assert!(pos < self.len);
        let data = self.data[pos / 16];
        let data = data >> (2 * (pos % 16));
        Bit2::new(data & 3)
    }

    #[inline(always)]
    pub fn set(&mut self, pos: usize, val: Bit2) {
        debug_assert!(pos < self.len);
        let mut data = self.data[pos / 16];
        data &= !(3 << (2 * (pos % 16)));
        data |= val.idx() << (2 * (pos % 16));
        self.data[pos / 16] = data;
    }

    #[inline(always)]
    pub fn fill(&mut self, val: Bit2) {
        self.data.fill(Buffer2::FILL[val.idx() as usize]);
    }

    #[inline(always)]
    pub fn fill_range(&mut self, range: Range<usize>, val: Bit2) {
        debug_assert!(range.start <= range.end && range.end <= self.len);

        let fill = Buffer2::FILL[val.idx() as usize];
        if range.start >= range.end {
        } else if range.start / 16 == range.end / 16 {
            let mask = (1 << (2 * (range.start % 16))) - 1;
            let mask = mask ^ ((1 << (2 * (range.end % 16))) - 1);
            let temp = self.data[range.start / 16];
            let temp = (temp & !mask) | (fill & mask);
            self.data[range.start / 16] = temp;
        } else {
            let mask = (1 << (2 * (range.start % 16))) - 1;
            let temp = self.data[range.start / 16];
            let temp = (temp & mask) | (fill & !mask);
            self.data[range.start / 16] = temp;

            self.data[(range.start / 16 + 1)..(range.end / 16)].fill(fill);

            if range.end % 16 != 0 {
                let mask = (1 << (2 * (range.end % 16))) - 1;
                let temp = self.data[range.end / 16];
                let temp = (temp & !mask) | (fill & mask);
                self.data[range.end / 16] = temp;
            }
        }
    }

    /// Updates all values in this buffer by applying the given binary
    /// operation to values coming from another buffer indexed by the
    /// given iterator.
    pub fn apply<ITER>(&mut self, op: Op222, other: &Self, iter: &mut ITER)
    where
        ITER: Iterator<Item = usize>,
    {
        let mut last = 0;
        for (pos1, pos2) in iter.enumerate() {
            self.set(pos1, op.of(self.get(pos1), other.get(pos2)));
            last = pos1 + 1;
        }
        debug_assert!(last == self.len);
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
        let mut buf1a = Buffer1::new(vec.len(), Bit1::new(0));
        let mut buf2a = Buffer2::new(vec.len(), Bit2::new(0));
        let mut buf1b: Buffer1 = Default::default();
        let mut buf2b: Buffer2 = Default::default();

        for (i, a) in vec.iter().enumerate() {
            buf1a.set(i, Bit1::new(a & 1));
            buf2a.set(i, Bit2::new(a & 3));
            buf1b.append(3, Bit1::new(a & 1));
            buf2b.append(3, Bit2::new(a & 3));
        }
        assert!(buf1a.len() == vec.len());
        assert!(buf2a.len() == vec.len());
        assert!(buf1b.len() == 3 * vec.len());
        assert!(buf2b.len() == 3 * vec.len());

        for (i, a) in vec.iter().enumerate() {
            assert_eq!(buf1a.get(i), Bit1::new(a & 1));
            assert_eq!(buf2a.get(i), Bit2::new(a & 3));
            for j in 0..3 {
                assert_eq!(buf1b.get(3 * i + j), Bit1::new(a & 1));
                assert_eq!(buf2b.get(3 * i + j), Bit2::new(a & 3));
            }
        }

        buf1a.fill(Bit1::new(1));
        buf2a.fill(Bit2::new(1));
        for i in 0..vec.len() {
            assert_eq!(buf1a.get(i), Bit1::new(1));
            assert_eq!(buf2a.get(i), Bit2::new(1));
        }
    }

    #[test]
    fn fill() {
        let vec = random(0x12345678, 11111);
        let mut buf1a = Buffer1::new(317, Bit1::new(0));
        let mut buf1b = Buffer1::new(317, Bit1::new(0));
        let mut buf2a = Buffer2::new(317, Bit2::new(0));
        let mut buf2b = Buffer2::new(317, Bit2::new(0));

        for &a in vec.iter() {
            let start = (a as usize) % (buf1a.len() + 1);
            let end = (a as usize >> 8) % (buf1a.len() + 1);
            let end = start.max(end);

            let val1 = Bit1::new((a >> 16) & 1);
            buf1a.fill_range(start..end, val1);
            for pos in start..end {
                buf1b.set(pos, val1);
            }
            assert_eq!(buf1a, buf1b);

            let val2 = Bit2::new((a >> 16) & 3);
            buf2a.fill_range(start..end, val2);
            for pos in start..end {
                buf2b.set(pos, val2);
            }
            assert_eq!(buf2a, buf2b);
        }
    }
}
