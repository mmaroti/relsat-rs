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

//! Structures for working with tensor shapes and views.

use std::ops::{Index, Range};

/// The shape of a tensor, which is just a vector of non-negative dimensions.
#[derive(PartialEq, Eq, Debug)]
pub struct Shape {
    dimensions: Box<[usize]>,
    size: usize,
}

impl Index<usize> for Shape {
    type Output = usize;

    fn index(&self, index: usize) -> &Self::Output {
        self.dimensions.index(index)
    }
}

impl Shape {
    /// Creates a new shape with the given dimensions.
    pub fn new(dimensions: Vec<usize>) -> Self {
        let dimensions = dimensions.into_boxed_slice();
        let mut size = 1;
        for &d in dimensions.iter() {
            size *= d;
        }
        Self { dimensions, size }
    }

    /// Iterates over the dimensions.
    pub fn iter(&self) -> std::slice::Iter<usize> {
        self.dimensions.iter()
    }

    /// Returns the number of dimensions.
    pub fn rank(&self) -> usize {
        self.dimensions.len()
    }

    /// The number of elements, which is just the product of all dimensions.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the position in a flat array of the given set of coordinates.
    /// The length of the coordinates must match the rank.
    pub fn position(&self, coordinates: &[usize]) -> usize {
        debug_assert!(coordinates.len() == self.dimensions.len());
        let mut n = 0;
        for (&c, &d) in coordinates.iter().zip(self.dimensions.iter()) {
            debug_assert!(c < d);
            n = n * d + c;
        }
        n
    }

    /// Sets the coordinates that correspond to the given position. The length
    /// of the coordinates must match the rank.
    pub fn coordinates(&self, mut position: usize, coordinates: &mut [usize]) {
        debug_assert!(position < self.size);
        debug_assert!(coordinates.len() == self.dimensions.len());
        for i in (0..self.dimensions.len()).rev() {
            let d = self.dimensions[i];
            coordinates[i] = position % d;
            position /= d;
        }
        debug_assert!(position == 0);
    }

    /// Returns an iterator through all valid positions, size many in total.
    pub fn positions(&self) -> Range<usize> {
        0..self.size
    }
}

/// The shape of a view into a tensor, which is a list of dimensions and the
/// corresponding strides.
#[derive(PartialEq, Eq, Debug)]
pub struct View {
    strides: Box<[(usize, usize)]>, // dim, stride
    offset: usize,
}

impl View {
    /// Creates the canonical view of the given shape, which is the fortran order,
    /// where the first coordinate has stride 1.
    pub fn new(shape: &Shape) -> Self {
        let mut s = 1;
        let strides = shape
            .dimensions
            .iter()
            .map(|&d| {
                let t = s;
                s *= d;
                (d, t)
            })
            .collect();
        let offset = 0;
        Self { strides, offset }
    }

    /// Returns the number of dimensions.
    pub fn rank(&self) -> usize {
        self.strides.len()
    }

    /// The number of elements, which is just the product of all dimensions.
    pub fn size(&self) -> usize {
        let mut n = 1;
        for &(d, _) in self.strides.iter() {
            n *= d;
        }
        n
    }

    /// Returns the position in a flat array of the given set of coordinates.
    /// The length of the coordinates must match the rank.
    pub fn position(&self, coordinates: &[usize]) -> usize {
        debug_assert!(coordinates.len() == self.strides.len());
        let mut n = self.offset;
        for (&c, &(d, s)) in coordinates.iter().zip(self.strides.iter()) {
            debug_assert!(c < d);
            n += c * s;
        }
        n
    }

    /// Returns an iterator through all valid positions, size many in total.
    /// You might want to call `simplify` before to speed up the iteration.
    pub fn positions(&self) -> Iter {
        Iter::new(self)
    }

    /// Returns the shape of this view as a new object.
    pub fn shape(&self) -> Shape {
        Shape::new(self.strides.iter().map(|&(d, _)| d).collect())
    }

    /// Permutes the coordinates of the given view. The map mast be of size rank.
    /// The old coordinate `i` will be placed at the new coordinate `map[i]`.
    pub fn permute(&self, map: &[usize]) -> Self {
        debug_assert!(map.len() == self.strides.len());
        let mut strides = vec![(0, 0); self.strides.len()].into_boxed_slice();
        for (i, &x) in map.iter().enumerate() {
            debug_assert!(strides[x] == (0, 0));
            strides[x] = self.strides[i];
        }
        let offset = self.offset;
        Self { strides, offset }
    }

    /// Computes the polymer of the given view, which allows the introduction
    /// dummy variables and identification of variables. The map must be of
    /// size rank. The old coordinate `i` will be placed at the new coordinate
    /// `map[i]`.
    pub fn polymer(&self, shape: &Shape, map: &[usize]) -> Self {
        debug_assert!(map.len() == self.strides.len());
        let strides: Vec<(usize, usize)> = shape.iter().map(|&d| (d, 0)).collect();
        let mut strides = strides.into_boxed_slice();
        for (i, &x) in map.iter().enumerate() {
            debug_assert!(self.strides[i].0 == strides[x].0);
            strides[x].1 += self.strides[i].1;
        }
        let offset = self.offset;
        Self { strides, offset }
    }

    /// Returns another view whose positions are the same but might have fewer
    /// dimensions because some could be merged into larger indices.
    pub fn simplify(&self) -> Self {
        let mut strides = self.strides.clone().into_vec();

        let mut tail = 0;
        let mut head = 1;
        while head < strides.len() {
            debug_assert!(tail < head);
            if strides[head].0 == 0 {
                tail = 0;
                strides[0] = (0, 0);
                break;
            }
            let s = strides[tail].0 * strides[tail].1;
            if s == strides[head].1 {
                strides[tail].0 *= strides[head].0;
            } else {
                tail += 1;
                strides[tail] = strides[head];
            }
            head += 1;
        }

        strides.truncate(tail + 1);
        let strides = strides.into_boxed_slice();
        let offset = self.offset;
        Self { strides, offset }
    }
}

/// View iterator that returns all valid positions, size many in total.
#[derive(Debug)]
pub struct Iter {
    index: usize,
    entries: Box<[(usize, usize, usize)]>, // coord, dim, stride
    done: bool,
}

impl Iter {
    /// Creates a new iterator for the given view.
    fn new(view: &View) -> Self {
        let mut done = false;
        let entries = view
            .strides
            .iter()
            .map(|&(d, s)| {
                done |= d == 0;
                (0, d, s)
            })
            .collect();

        let index = view.offset;
        Self {
            index,
            entries,
            done,
        }
    }

    /// Resets the iterator to the first element.
    fn reset(&mut self) {
        self.done = false;
        for e in self.entries.iter_mut() {
            self.done |= e.1 == 0;
            self.index -= e.0 * e.2;
            e.0 = 0;
        }
    }
}

impl Iterator for Iter {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.done {
            None
        } else {
            let index = self.index;
            for e in self.entries.iter_mut() {
                self.index += e.2;
                e.0 += 1;
                if e.0 >= e.1 {
                    self.index -= e.0 * e.2;
                    e.0 = 0;
                } else {
                    return Some(index);
                }
            }
            self.done = true;
            Some(index)
        }
    }
}
