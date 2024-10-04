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
use std::{fmt, ops, ptr};

use super::{get_coords, get_offset, Coord, Domain};

#[derive(Debug)]
pub struct Predicate {
    name: String,
    domains: Box<[Rc<Domain>]>,
    var_start: usize,
    var_count: usize,
}

impl Predicate {
    pub fn new(name: String, domains: Vec<Rc<Domain>>, var_start: usize) -> Self {
        let domains = domains.into_boxed_slice();
        let var_count = domains.iter().map(|dom| dom.size()).product();
        Self {
            name,
            domains,
            var_start,
            var_count,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arity(&self) -> usize {
        self.domains.len()
    }

    pub fn domain(&self, pos: usize) -> &Rc<Domain> {
        &self.domains[pos]
    }

    pub fn var_count(&self) -> usize {
        self.var_count
    }

    pub fn get_coords(&self, offset: usize, coords: &mut [Coord]) {
        get_coords(&self.domains, offset, coords);
    }

    pub fn get_offset<I>(&self, coords: I) -> usize
    where
        I: ExactSizeIterator<Item = Coord>,
    {
        get_offset(&self.domains, coords)
    }

    pub fn ptr_eq(&self, other: &Predicate) -> bool {
        ptr::eq(self, other)
    }
}

impl fmt::Display for Predicate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", self.name)?;
        let mut first = true;
        for dom in self.domains.iter() {
            if first {
                first = false;
            } else {
                write!(f, ",")?;
            }
            write!(f, "{}", dom.name())?;
        }
        write!(f, ")")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiteralIdx(usize);

impl LiteralIdx {
    pub fn new(negated: bool, variable: usize) -> Self {
        debug_assert!(variable <= (usize::MAX >> 1));
        Self((variable << 1) + (negated as usize))
    }

    pub fn negated(self) -> bool {
        (self.0 & 1) != 0
    }

    pub fn variable(self) -> usize {
        self.0 >> 1
    }
}

impl ops::Not for LiteralIdx {
    type Output = Self;

    fn not(self) -> Self {
        LiteralIdx(self.0 ^ 1)
    }
}

impl ops::BitXor<bool> for LiteralIdx {
    type Output = Self;

    fn bitxor(self, rhs: bool) -> Self {
        LiteralIdx(self.0 ^ (rhs as usize))
    }
}

#[derive(Debug)]
pub struct Literal<'a> {
    negated: bool,
    predicate: &'a Rc<Predicate>,
    coords: Vec<Coord>,
}

impl<'a> Literal<'a> {
    pub fn new(negated: bool, predicate: &'a Rc<Predicate>, coords: Vec<Coord>) -> Self {
        debug_assert_eq!(coords.len(), predicate.arity());
        Self {
            negated,
            predicate,
            coords,
        }
    }

    pub fn idx(&self) -> LiteralIdx {
        let var = self.predicate.var_start + self.predicate.get_offset(self.coords.iter().cloned());
        LiteralIdx::new(self.negated, var)
    }

    pub fn negated(&self) -> bool {
        self.negated
    }

    pub fn predicate(&self) -> &Rc<Predicate> {
        self.predicate
    }

    pub fn coords(&self) -> &[Coord] {
        &self.coords
    }

    pub fn destroy(self) -> Vec<Coord> {
        self.coords
    }
}

impl<'a> fmt::Display for Literal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}[",
            if self.negated { '-' } else { '+' },
            self.predicate.name
        )?;
        let mut first = true;
        for coord in self.coords.iter() {
            if first {
                first = false;
            } else {
                write!(f, ",")?;
            }
            write!(f, "{}", coord.0)?;
        }
        write!(f, "]")
    }
}
