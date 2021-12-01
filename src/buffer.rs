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

use super::shape::{Shape, View};

#[derive(Debug)]
pub struct Buffer {
    data: Vec<u32>,
    shape: Shape,
}

impl Buffer {
    pub fn new(shape: Shape) -> Self {
        let len = (shape.size() + 31) / 32;
        let mut data = Vec::with_capacity(len);
        unsafe { data.set_len(len) };
        Self { data, shape }
    }

    pub fn size(&self) -> usize {
        self.shape.size()
    }

    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    pub fn view(&self) -> View {
        View::new(&self.shape)
    }

    pub fn get(&self, coordinates: &[usize]) -> bool {
        let pos = self.shape.position(coordinates);
        self.data[pos / 32] & (1 << (pos % 32)) != 0
    }

    pub fn set(&mut self, coordinates: &[usize]) {
        let pos = self.shape.position(coordinates);
        self.data[pos / 32] |= 1 << (pos % 32);
    }

    pub fn clr(&mut self, coordinates: &[usize]) {
        let pos = self.shape.position(coordinates);
        self.data[pos / 32] &= !(1 << (pos % 32));
    }
}
