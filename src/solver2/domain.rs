/*
* Copyright (C) 2019-2022, Miklos Maroti
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

use std::rc::Rc;
use std::{fmt, ptr};

#[derive(Debug)]
pub struct Domain {
    size: usize,
    name: String,
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Domain {
    pub fn new(name: String, size: usize) -> Self {
        Self { name, size }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ptr_eq(&self, other: &Domain) -> bool {
        ptr::eq(self, other)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coord(pub usize);

pub fn get_coords(domains: &[Rc<Domain>], mut offset: usize, coords: &mut [Coord]) {
    debug_assert_eq!(domains.len(), coords.len());
    for (size, coord) in domains
        .iter()
        .map(|dom| dom.size())
        .zip(coords.iter_mut())
        .rev()
    {
        *coord = Coord(offset % size);
        offset /= size;
    }
    debug_assert_eq!(offset, 0);
}

pub fn get_offset<I>(domains: &[Rc<Domain>], coords: I) -> usize
where
    I: ExactSizeIterator<Item = Coord>,
{
    debug_assert_eq!(domains.len(), coords.len());
    let mut offset = 0;
    for (size, coord) in domains.iter().map(|dom| dom.size()).zip(coords) {
        debug_assert!(coord.0 < size);
        offset *= size;
        offset += coord.0;
    }
    offset
}
