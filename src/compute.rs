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
pub struct Shape {
    dims: Vec<usize>,
}

impl Shape {
    pub fn new(dims: Vec<usize>) -> Self {
        Self { dims }
    }

    pub fn len(&self) -> usize {
        let mut n = 1;
        for &d in self.dims.iter() {
            n *= d;
        }
        n
    }

    pub fn pos(&self, idx: &Vec<usize>) -> usize {
        assert!(idx.len() == self.dims.len());
        let mut n = 0;
        for (&c, &d) in idx.iter().zip(self.dims.iter()) {
            assert!(c < d);
            n = n * d + c;
        }
        n
    }

    pub fn idx(&self, mut pos: usize) -> Vec<usize> {
        let mut idx = Vec::with_capacity(self.dims.len());
        unsafe { idx.set_len(self.dims.len()) };
        for i in (0..self.dims.len()).rev() {
            idx[i] = pos % self.dims[i];
            pos /= self.dims[i];
        }
        assert!(pos == 0);
        idx
    }
}

#[derive(Debug)]
pub struct Relation {
    data: Vec<u32>,
    shape: Vec<usize>,
    len: usize,
}

impl Relation {
    pub fn new(shape: Vec<usize>) -> Self {
        let mut len = 1;
        for &s in &shape {
            len *= s;
        }
        let mut data = Vec::with_capacity((len + 31) / 32);
        unsafe { data.set_len((len + 31) / 32) };
        Self { data, shape, len }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get(&self, tuple: Vec<usize>) -> bool {
        assert!(tuple.len() == self.shape.len());
        true
    }
}

#[derive(Debug)]
pub struct Polymer {}
