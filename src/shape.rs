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

//! Structures for working with tensor shapes and views.

use std::ops::{Index, Range};

/// Data type to store arities.
pub type ARITY = u32;

/// Data type to store dimensions.
pub type DIMEN = u32;

/// The shape of a tensor, which is just a vector of non-negative dimensions.
#[derive(PartialEq, Eq, Debug)]
pub struct Shape {
    dimensions: Vec<DIMEN>,
    size: usize,
}

impl Index<ARITY> for Shape {
    type Output = DIMEN;

    fn index(&self, index: ARITY) -> &Self::Output {
        self.dimensions.index(index as usize)
    }
}

impl Shape {
    /// Creates a new shape with the given dimensions.
    pub fn new(dimensions: Vec<DIMEN>) -> Self {
        assert!(dimensions.len() <= ARITY::MAX as usize);
        let mut size = 1;
        for &d in dimensions.iter() {
            size *= d as usize;
        }
        Self { dimensions, size }
    }

    /// Iterates over the dimensions.
    pub fn iter(&self) -> std::slice::Iter<DIMEN> {
        self.dimensions.iter()
    }

    /// Returns the number of dimensions.
    pub fn rank(&self) -> ARITY {
        self.dimensions.len() as ARITY
    }

    /// The number of elements, which is just the product of all dimensions.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the position in a flat array of the given set of coordinates.
    /// The length of the coordinates must match the rank.
    pub fn position(&self, coordinates: &[DIMEN]) -> usize {
        debug_assert!(coordinates.len() == self.dimensions.len());
        let mut n = 0;
        for (&c, &d) in coordinates.iter().zip(self.dimensions.iter()) {
            debug_assert!(c < d);
            n = n * (d as usize) + (c as usize);
        }
        n
    }

    /// Sets the coordinates that correspond to the given position. The length
    /// of the coordinates must match the rank.
    pub fn coordinates(&self, mut position: usize, coordinates: &mut [DIMEN]) {
        debug_assert!(position < self.size);
        debug_assert!(coordinates.len() == self.dimensions.len());
        for i in (0..self.dimensions.len()).rev() {
            let d = self.dimensions[i] as usize;
            coordinates[i] = (position % d) as DIMEN;
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
    strides: Vec<(DIMEN, usize)>, // dim, stride
    offset: usize,
}

impl View {
    /// Creates the canonical view of the given shape, which is the fortran order,
    /// where the first coordinate has stride 1.
    pub fn new(shape: &Shape) -> Self {
        let mut s: usize = 1;
        let strides: Vec<(DIMEN, usize)> = shape
            .dimensions
            .iter()
            .map(|&d| {
                let t = s;
                s *= d as usize;
                (d, t)
            })
            .collect();
        debug_assert!(strides.len() <= ARITY::MAX as usize);
        let offset = 0;
        Self { strides, offset }
    }

    /// Returns the number of dimensions.
    pub fn rank(&self) -> ARITY {
        self.strides.len() as ARITY
    }

    /// The number of elements, which is just the product of all dimensions.
    pub fn size(&self) -> usize {
        let mut n = 1;
        for &(d, _) in self.strides.iter() {
            n *= d as usize;
        }
        n
    }

    /// Returns the position in a flat array of the given set of coordinates.
    /// The length of the coordinates must match the rank.
    pub fn position(&self, coordinates: &[DIMEN]) -> usize {
        debug_assert!(coordinates.len() == self.strides.len());
        let mut n = self.offset;
        for (&c, &(d, s)) in coordinates.iter().zip(self.strides.iter()) {
            debug_assert!(c < d);
            n += (c as usize) * s;
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
    pub fn permute(&self, map: &[ARITY]) -> Self {
        debug_assert!(map.len() == self.strides.len());
        let mut strides = vec![(DIMEN::MAX, usize::MAX); self.strides.len()];
        for (i, &x) in map.iter().enumerate() {
            debug_assert!(strides[x as usize] == (DIMEN::MAX, usize::MAX));
            strides[x as usize] = self.strides[i];
        }
        let offset = self.offset;
        Self { strides, offset }
    }

    /// Computes the polymer of the given view, which allows the introduction
    /// dummy variables and identification of variables. The map must be of
    /// size rank. The old coordinate `i` will be placed at the new coordinate
    /// `map[i]`.
    pub fn polymer(&self, shape: &Shape, map: &[ARITY]) -> Self {
        debug_assert!(map.len() == self.strides.len());
        let mut strides: Vec<(DIMEN, usize)> = shape.iter().map(|&d| (d, 0)).collect();
        for (i, &x) in map.iter().enumerate() {
            debug_assert!(self.strides[i].0 == strides[x as usize].0);
            strides[x as usize].1 += self.strides[i].1;
        }
        let offset = self.offset;
        Self { strides, offset }
    }

    /// Returns another view whose positions are the same but might have fewer
    /// dimensions because some could be merged into larger indices.
    pub fn simplify(&self) -> Self {
        let mut strides = self.strides.clone();

        let mut tail = 0;
        let mut head = 1;
        while head < strides.len() {
            debug_assert!(tail < head);
            if strides[head].0 == 0 {
                tail = 0;
                strides[0] = (0, 0);
                break;
            }
            let s = (strides[tail].0 as usize) * strides[tail].1;
            if s == strides[head].1 {
                strides[tail].0 *= strides[head].0;
            } else {
                tail += 1;
                strides[tail] = strides[head];
            }
            head += 1;
        }

        strides.resize(tail + 1, (0, 0));
        let offset = self.offset;
        Self { strides, offset }
    }
}

/// View iterator that returns all valid positions, size many in total.
#[derive(Debug)]
pub struct Iter {
    index: usize,
    entries: Vec<(DIMEN, DIMEN, usize)>, // coord, dim, stride
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
            self.index -= (e.0 as usize) * e.2;
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
                    self.index -= (e.0 as usize) * e.2;
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
