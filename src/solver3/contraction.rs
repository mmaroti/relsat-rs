/*
* Copyright (C) 2019-2024, Miklos Maroti
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

use super::buffer::Buffer2;
use super::shape::Shape;

#[derive(Debug, Clone)]
struct Axis<const LEN: usize> {
    index: usize,
    length: usize,
    strides: [(usize, usize); LEN],
}

#[derive(Debug, Clone)]
struct Conj<const LEN: usize> {
    output: Shape,
    inputs: [Shape; LEN],
}

impl<const LEN: usize> Conj<LEN> {
    fn new(output: Shape, inputs: [Shape; LEN]) -> Self {
        for input in inputs.iter() {
            debug_assert!(output.equals(input));
        }
        Self { output, inputs }
    }

    fn apply(&self, buffer: &mut Buffer2) {}
}
