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

    /// Returns true if the list of lengths of this and the other shape is equal.
    pub fn equals(&self, other: &Shape) -> bool {
        if self.axes.len() != other.axes.len() {
            return false;
        }
        for (a, b) in self.axes.iter().zip(other.axes.iter()) {
            if a.length != b.length {
                return false;
            }
        }
        return true;
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

    /// Returns the position in the flat array of the element at the given
    /// indices. The number of indices must match the dimension of the tensor.
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

    /// Returns an iterator through all valid positions, volume many in total.
    pub fn positions(&self) -> Iter {
        Iter::new(self)
    }
}

#[derive(Debug, Clone)]
struct Axis2 {
    stride: usize,
    index: usize,
    length: usize,
    product: usize,
}

/// ShapeIter iterator that returns all valid positions, size many in total.
#[derive(Debug, Clone)]
pub struct Iter {
    length: usize,
    position: usize,
    axes: Vec<Axis2>,
}

impl Iter {
    /// Creates a new iterator for the given shape.
    fn new(shape: &Shape) -> Self {
        let mut axes: Vec<Axis2> = Vec::with_capacity(shape.axes.len());
        let mut volume = 1;
        for axis in shape.axes.iter() {
            volume *= axis.length;
            if let Some(axis2) = axes.last_mut() {
                if axis2.product == axis.stride {
                    axis2.length *= axis.length;
                    axis2.product *= axis.length;
                    continue;
                }
            }
            axes.push(Axis2 {
                stride: axis.stride,
                length: axis.length,
                index: 0,
                product: axis.stride * axis.length,
            });
        }
        Iter {
            length: volume,
            position: shape.offset,
            axes,
        }
    }
}

impl Iterator for Iter {
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

impl ExactSizeIterator for Iter {
    fn len(&self) -> usize {
        self.length
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
        assert_eq!(shape.positions().axes.len(), 1);
        let pos: Vec<usize> = shape.positions().collect();
        assert_eq!(pos, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        assert_eq!(shape.position([1, 2, 1].into_iter()), 11);

        let view = shape.permute(&[2, 0, 1]);
        assert_eq!(view.positions().axes.len(), 2);
        let pos: Vec<usize> = view.positions().collect();
        assert_eq!(pos, vec![0, 2, 4, 6, 8, 10, 1, 3, 5, 7, 9, 11]);

        let view = shape.swap(0, 1);
        assert_eq!(view.positions().axes.len(), 3);
        let pos: Vec<usize> = view.positions().collect();
        assert_eq!(pos, vec![0, 2, 4, 1, 3, 5, 6, 8, 10, 7, 9, 11]);

        let view = shape.polymer(vec![2, 3, 2].into_iter(), &[2, 1, 2]);
        assert_eq!(view.positions().axes.len(), 3);
        let pos: Vec<usize> = view.positions().collect();
        assert_eq!(pos, vec![0, 0, 2, 2, 4, 4, 7, 7, 9, 9, 11, 11]);
    }
}
