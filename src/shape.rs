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

use std::ops::Range;

/// The rectangular shape of a tensor, which is just a vector of non-negative
/// integers.
#[derive(PartialEq, Eq, Debug)]
pub struct Shape {
    lengths: Box<[usize]>,
    offset: usize,
    volume: usize,
}

impl Shape {
    /// Creates a new shape with the given side lengths. The offset allows
    /// to map positions of this shape into a range of a flat buffer.
    pub fn new(lengths: Vec<usize>, offset: usize) -> Self {
        let lengths = lengths.into_boxed_slice();
        let mut volume = 1;
        for &d in lengths.iter() {
            volume *= d;
        }
        Self {
            lengths,
            volume,
            offset,
        }
    }

    /// Returns the number of side lengths.
    pub fn dimension(&self) -> usize {
        self.lengths.len()
    }

    /// Returns the side length along the given axis.
    pub fn length(&self, axis: usize) -> usize {
        self.lengths[axis]
    }

    /// The number of elements, which is just the product of all side lengths.
    pub fn volume(&self) -> usize {
        self.volume
    }

    /// Returns the position in a flat array of the element at the given
    /// coordinates. The number of coordinates must match the dimension.
    /// The last coordinate is advancing the fastest.
    pub fn position(&self, coordinates: &[usize]) -> usize {
        debug_assert!(coordinates.len() == self.lengths.len());
        let mut n = 0;
        for (&c, &d) in coordinates.iter().zip(self.lengths.iter()) {
            debug_assert!(c < d);
            n = n * d + c;
        }
        self.offset + n
    }

    /// Sets the coordinates that correspond to the given position. The length
    /// of the coordinates must match the dimension. The last coordinate is
    /// advancing the fastest.
    pub fn coordinates(&self, mut position: usize, coordinates: &mut [usize]) {
        debug_assert!(self.offset <= position && position < self.offset + self.volume);
        debug_assert!(coordinates.len() == self.dimension());
        position -= self.offset;
        for (i, &d) in self.lengths.iter().enumerate().rev() {
            coordinates[i] = position % d;
            position /= d;
        }
        debug_assert!(position == 0);
    }

    /// Returns an iterator through all valid positions, volume many in total.
    pub fn positions(&self) -> Range<usize> {
        self.offset..(self.offset + self.volume)
    }

    /// Creates the default view of this shape.
    pub fn view(&self) -> ShapeView {
        ShapeView::new(self)
    }
}

/// The shape of a view into a tensor, which is a list of side lengths
/// and the corresponding strides.
#[derive(PartialEq, Eq, Debug)]
pub struct ShapeView {
    strides: Box<[(usize, usize)]>, // length, stride
    offset: usize,
}

impl ShapeView {
    /// Creates the canonical view of the given shape, where the last coordinate
    /// is advancing the fastest.
    pub fn new(shape: &Shape) -> Self {
        let mut strides: Box<[(usize, usize)]> = shape.lengths.iter().map(|&d| (d, 0)).collect();
        let mut s = 1;
        for mut e in strides.iter_mut().rev() {
            e.1 = s;
            s *= e.0;
        }
        Self {
            strides,
            offset: shape.offset,
        }
    }

    /// Returns the number of dimensions.
    pub fn dimension(&self) -> usize {
        self.strides.len()
    }

    /// Returns the side length along the given axis.
    pub fn length(&self, axis: usize) -> usize {
        self.strides[axis].0
    }

    /// The number of elements, which is just the product of all side length.
    pub fn volume(&self) -> usize {
        let mut n = 1;
        for &(d, _) in self.strides.iter() {
            n *= d;
        }
        n
    }

    /// Returns the position in a flat array of the element at the given
    /// coordinates. The number of coordinates must match the dimension.
    /// The last coordinate is advancing the fastest.
    pub fn position(&self, coordinates: &[usize]) -> usize {
        debug_assert!(coordinates.len() == self.strides.len());
        let mut n = self.offset;
        for (&c, &(d, s)) in coordinates.iter().zip(self.strides.iter()) {
            debug_assert!(c < d);
            n += c * s;
        }
        n
    }

    /// Returns an iterator through all valid positions, volume many in total.
    /// You might want to call `simplify` before to speed up the iteration.
    pub fn positions(&self) -> ShapeIter {
        ShapeIter::new(self)
    }

    /// Permutes the coordinates of the given view. The map must be of size
    /// dimension. The old coordinate `i` will be placed at the new coordinate
    /// `map[i]`.
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
    /// size dimension. The old coordinate `i` will be placed at the new
    /// coordinate `map[i]`.
    pub fn polymer(&self, shape: &Shape, map: &[usize]) -> Self {
        debug_assert!(map.len() == self.strides.len());
        let strides: Vec<(usize, usize)> = shape.lengths.iter().map(|&d| (d, 0)).collect();
        let mut strides = strides.into_boxed_slice();
        for (i, &x) in map.iter().enumerate() {
            debug_assert!(self.strides[i].0 == strides[x].0);
            strides[x].1 += self.strides[i].1;
        }
        let offset = self.offset;
        Self { strides, offset }
    }

    /// Returns another view whose positions are the same but might have
    /// smaller dimension because some axis could be merged.
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
            let s = strides[head].0 * strides[head].1;
            if s == strides[tail].1 {
                strides[tail].0 *= strides[head].0;
                strides[tail].1 = strides[head].1;
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

/// ShapeView iterator that returns all valid positions, size many in total.
#[derive(Debug)]
pub struct ShapeIter {
    index: usize,
    entries: Box<[(usize, usize, usize)]>, // coord, dim, stride
    done: bool,
}

impl ShapeIter {
    /// Creates a new iterator for the given view.
    fn new(view: &ShapeView) -> Self {
        let mut done = false;
        let entries = view
            .strides
            .iter()
            .rev()
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
    pub fn reset(&mut self) {
        self.done = false;
        for e in self.entries.iter_mut() {
            self.done |= e.1 == 0;
            self.index -= e.0 * e.2;
            e.0 = 0;
        }
    }
}

impl Iterator for ShapeIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape() {
        let shape = Shape::new(vec![2, 3, 4], 0);
        let pos1: Vec<usize> = shape.positions().collect();

        let view = shape.view();
        assert_eq!(view.volume(), shape.volume());
        assert_eq!(view.dimension(), 3);
        assert_eq!(view.length(0), 2);
        assert_eq!(view.length(1), 3);
        assert_eq!(view.length(2), 4);
        let pos2: Vec<usize> = view.positions().collect();
        assert_eq!(pos1, pos2);

        let view = shape.view().simplify();
        assert_eq!(view.volume(), shape.volume());
        assert_eq!(view.dimension(), 1);
        assert_eq!(view.length(0), 24);
        let pos2: Vec<usize> = view.positions().collect();
        assert_eq!(pos1, pos2);

        let view = shape.view().permute(&[2, 0, 1]);
        assert_eq!(view.volume(), shape.volume());
        assert_eq!(view.dimension(), 3);
        assert_eq!(view.length(0), 3);
        assert_eq!(view.length(1), 4);
        assert_eq!(view.length(2), 2);
        let pos2: Vec<usize> = view.positions().collect();
        let pos3 = vec![
            0, 12, 1, 13, 2, 14, 3, 15, 4, 16, 5, 17, 6, 18, 7, 19, 8, 20, 9, 21, 10, 22, 11, 23,
        ];
        assert_eq!(pos2, pos3);
    }
}
