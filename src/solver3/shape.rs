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

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
struct Axis {
    length: usize,
    stride: usize,
}

/// The rectangular shape of a tensor.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Shape {
    axes: Box<[Axis]>,
    offset: usize,
}

impl Shape {
    /// Creates a new shape with the given side lengths. The offset allows
    /// to map positions of this shape into a flat buffer index space.
    pub fn new<ITER>(lengths: ITER, offset: usize) -> Self
    where
        ITER: ExactSizeIterator<Item = usize>,
    {
        let mut stride = 1;
        let mut axes: Vec<Axis> = Vec::with_capacity(lengths.len());
        for length in lengths {
            axes.push(Axis { length, stride });
            stride *= length;
        }

        Self {
            axes: axes.into_boxed_slice(),
            offset,
        }
    }

    /// Returns the number of axes of the tensor.
    pub fn dimension(&self) -> usize {
        self.axes.len()
    }

    /// Returns an iterator for the lengths of all sides.
    pub fn lengths(&self) -> impl ExactSizeIterator<Item = usize> + '_ {
        self.axes.iter().map(|axis| axis.length)
    }

    /// Returns the side length along the given axis.
    pub fn length(&self, axis: usize) -> usize {
        self.axes[axis].length
    }

    /// The number of elements, which is just the product of all side length.
    pub fn volume(&self) -> usize {
        let mut volume = 1;
        for &dim in self.axes.iter() {
            volume *= dim.length;
        }
        volume
    }

    /// Returns the position in a flat array of the element at the given
    /// indices. The number of indices must match the rank of the tensor.
    pub fn position<ITER>(&self, indices: ITER) -> usize
    where
        ITER: ExactSizeIterator<Item = usize>,
    {
        debug_assert_eq!(indices.len(), self.dimension());
        let mut pos = self.offset;
        for (index, side) in indices.zip(self.axes.iter()) {
            debug_assert!(index < side.length);
            pos += index * side.stride;
        }
        pos
    }

    /// Permutes the axes of the given shape. The map must be of size
    /// dimension. The old coordinate `i` will be placed at the new
    /// coordinate `map[i]`.
    pub fn permute(&self, map: &[usize]) -> Self {
        debug_assert_eq!(map.len(), self.dimension());
        let mut axes = vec![
            Axis {
                length: 0,
                stride: 0
            };
            self.axes.len()
        ]
        .into_boxed_slice();
        for (old, &new) in map.iter().enumerate() {
            debug_assert_eq!(axes[new].length, 0);
            debug_assert_eq!(axes[new].stride, 0);
            axes[new] = self.axes[old];
        }
        Self {
            axes,
            offset: self.offset,
        }
    }

    /// Permutes two axes of the given shape. The two axes can be the same.
    pub fn swap(&self, axis1: usize, axis2: usize) -> Self {
        debug_assert!(axis1 < self.dimension() && axis2 < self.dimension());
        let mut axes = self.axes.clone();
        axes[axis1] = self.axes[axis2];
        axes[axis2] = self.axes[axis1];
        Self {
            axes,
            offset: self.offset,
        }
    }

    /// Computes the polymer of the given shape, which allows the introduction
    /// dummy variables and identification of variables. The map must be of
    /// size dimension. The old coordinate `i` will be placed at the new
    /// coordinate `map[i]`. The lengths iterator gives the lengths of the
    /// new shape, which is also used to obtain the lengths of dummy axes.
    pub fn polymer<ITER>(&self, lengths: ITER, map: &[usize]) -> Self
    where
        ITER: ExactSizeIterator<Item = usize>,
    {
        debug_assert_eq!(map.len(), self.dimension());
        let axes: Vec<Axis> = lengths.map(|length| Axis { length, stride: 0 }).collect();
        let mut axes = axes.into_boxed_slice();
        for (old, &new) in map.iter().enumerate() {
            debug_assert_eq!(axes[new].length, self.axes[old].length);
            axes[new].stride += self.axes[old].stride;
        }
        Self {
            axes,
            offset: self.offset,
        }
    }

    /// Returns another view whose positions are the same but might have
    /// smaller dimension because some axes could be merged.
    pub fn simplify(&self) -> Self {
        let mut axes = self.axes.clone().into_vec();

        let mut tail = 0;
        let mut head = 1;
        while head < axes.len() {
            debug_assert!(tail < head);
            if axes[head].length == 0 {
                tail = 0;
                axes[0] = Axis {
                    length: 0,
                    stride: 0,
                };
                break;
            }
            let s = axes[head].length * axes[head].stride;
            if s == axes[tail].stride {
                axes[tail].length *= axes[head].length;
                axes[tail].stride = axes[head].stride;
            } else {
                tail += 1;
                axes[tail] = axes[head];
            }
            head += 1;
        }

        axes.truncate(tail + 1);
        let axes = axes.into_boxed_slice();
        let offset = self.offset;
        Self { axes, offset }
    }

    /// Returns an iterator through all valid positions, volume many in total.
    /// You might want to call `simplify` before to speed up the iteration.
    pub fn positions(&self) -> ShapeIter {
        ShapeIter::new(self)
    }
}

#[derive(Debug, Clone, Copy)]
struct Axis2 {
    stride: usize,
    index: usize,
    length: usize,
    product: usize,
}

/// ShapeIter iterator that returns all valid positions, size many in total.
#[derive(Debug, Clone)]
pub struct ShapeIter2 {
    length: usize,
    position: usize,
    axes: Vec<Axis2>,
}

impl Iterator for ShapeIter2 {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            let pos = self.position;
            for axis in self.axes.iter_mut() {
                self.position += axis.stride;
                axis.index += 1;
                if axis.index >= axis.length {
                    axis.index = 0;
                    self.position -= axis.product;
                } else {
                    break;
                }
            }
            Some(pos)
        }
    }
}

impl ExactSizeIterator for ShapeIter2 {
    fn len(&self) -> usize {
        self.length
    }
}

/// ShapeIter iterator that returns all valid positions, size many in total.
#[derive(Debug)]
pub struct ShapeIter {
    index: usize,
    entries: Box<[(usize, usize, usize)]>, // coord, dim, stride
    done: bool,
}

impl ShapeIter {
    /// Creates a new iterator for the given shape.
    fn new(shape: &Shape) -> Self {
        let mut done = false;
        let entries = shape
            .axes
            .iter()
            .map(|a| {
                done |= a.length == 0;
                (0, a.length, a.stride)
            })
            .collect();

        let index = shape.offset;
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
        let shape = Shape::new(vec![2, 3, 2].into_iter(), 0);
        assert_eq!(shape.volume(), 12);
        assert_eq!(shape.dimension(), 3);
        assert_eq!(shape.length(0), 2);
        assert_eq!(shape.length(1), 3);
        assert_eq!(shape.length(2), 2);
        let pos: Vec<usize> = shape.positions().collect();
        assert_eq!(pos, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

        let view = shape.permute(&[2, 1, 0]);
        let pos: Vec<usize> = view.positions().collect();
        assert_eq!(pos, vec![0, 6, 2, 8, 4, 10, 1, 7, 3, 9, 5, 11]);

        let view = shape.polymer(vec![3, 2].into_iter(), &[1, 0, 1]);
        let pos: Vec<usize> = view.positions().collect();
        assert_eq!(pos, vec![0, 2, 4, 7, 9, 11]);
    }
}
