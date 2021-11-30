/*
* Copyright (C) 2019-2020, Miklos Maroti
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
pub struct Buffer {
    data: Vec<u32>,
    length: usize,
}

impl Buffer {
    pub fn new(length: usize) -> Self {
        let n = (length + 31) / 32;
        let mut data = Vec::with_capacity(n);
        unsafe { data.set_len(n) };
        Self { data, length }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn get(&self, index: usize) -> bool {
        debug_assert!(index < self.length);
        let n = index / 32;
        let b = 1 << (index % 32);
        self.data[n] & b != 0
    }
}

#[derive(Debug)]
pub struct View {
    shape: Vec<(usize, usize)>, // dim, stride
    offset: usize,
}

impl View {
    pub fn dim(&self) -> usize {
        self.shape.len()
    }

    pub fn len(&self) -> usize {
        let mut len = 1;
        for &(d, _) in self.shape.iter() {
            len *= d;
        }
        len
    }

    pub fn index(&self, coord: &Vec<usize>) -> usize {
        assert!(coord.len() == self.shape.len());
        let mut index = self.offset;
        for (&c, &(d, s)) in coord.iter().zip(self.shape.iter()) {
            assert!(c < d);
            index += c * s;
        }
        index
    }
}

#[derive(Debug)]
pub struct Table<'a> {
    buffer: &'a Buffer,
    shape: Vec<(usize, usize)>, // dim, stride
    offset: usize,
}

impl<'a> Table<'a> {
    pub fn len(&self) -> usize {
        let mut len = 1;
        for &(dim, _) in self.shape.iter() {
            len *= dim;
        }
        len
    }

    pub fn dim(&self) -> usize {
        self.shape.len()
    }

    pub fn get(&self, coord: &Vec<usize>) -> bool {
        assert!(coord.len() == self.shape.len());
        let mut index = self.offset;
        for (&c, &(d, s)) in coord.iter().zip(self.shape.iter()) {
            assert!(c < d);
            index += c * s;
        }
        self.buffer.get(index)
    }
}

#[derive(Debug)]
pub struct BitIter<'a> {
    vector: &'a Buffer,
    shape: Vec<(usize, usize, usize)>, // coord, dim, stride
    offset: usize,
}

impl<'a> BitIter<'a> {
    pub fn len(&self) -> usize {
        let mut len = 1;
        for &(_, dim, _) in self.shape.iter() {
            len *= dim;
        }
        len
    }

    pub fn pos(&self) -> usize {
        let mut pos = 0;
        for &(coord, dim, _) in self.shape.iter() {
            debug_assert!(coord < dim);
            pos = pos * dim + coord;
        }
        pos
    }
}

impl<'a> Iterator for BitIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == usize::MAX {
            None
        } else {
            None
        }
    }
}
