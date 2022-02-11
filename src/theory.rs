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

#[derive(Debug)]
pub struct Domain {
    name: String,
}

#[derive(Debug)]
pub struct Variable {
    name: String,
    domains: Vec<Rc<Domain>>,
}

impl Variable {
    pub fn arity(&self) -> usize {
        self.domains.len()
    }
}

#[derive(Debug)]
struct Literal {
    sign: bool,
    variable: Rc<Variable>,
    axes: Box<[usize]>,
}

#[derive(Debug)]
pub struct Clause {
    domains: Vec<Rc<Domain>>,
    variables: Vec<Rc<Variable>>,
    literals: Vec<Literal>,
}

#[derive(Debug)]
pub struct Theory {
    domains: Vec<Rc<Domain>>,
    variables: Vec<Rc<Variable>>,
    clauses: Vec<Rc<Clause>>,
}
