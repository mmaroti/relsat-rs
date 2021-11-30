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

use std::fmt;

#[derive(Clone, Debug)]
pub struct Variable {
    pub arity: u32,
    pub name: String,
}

impl Variable {
    pub fn new(name: &str, arity: u32) -> Self {
        let name = name.to_string();
        Self { arity, name }
    }

    fn fmt<Iter>(&self, iter: Iter) -> VariableFmt<'_, Iter>
    where
        Iter: Clone + Iterator<Item = u32>,
    {
        let name = &self.name;
        VariableFmt { name, iter }
    }
}

struct VariableFmt<'a, Iter>
where
    Iter: Clone + Iterator<Item = u32>,
{
    name: &'a str,
    iter: Iter,
}

impl<'a, Iter> fmt::Display for VariableFmt<'a, Iter>
where
    Iter: Clone + Iterator<Item = u32>,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "{}(", self.name)?;
        let mut first = true;
        for x in self.iter.clone() {
            if first {
                first = false;
            } else {
                write!(formatter, ",")?;
            }
            write!(formatter, "x{}", x)?;
        }
        write!(formatter, ")")
    }
}

#[derive(Clone, Debug)]
pub struct Literal {
    variable: usize,
    arity: u32,
    sign: bool,
    vars: Vec<u32>,
}

impl Literal {
    pub fn new(variable: usize, sign: bool, arity: u32, vars: Vec<u32>) -> Self {
        for &var in &vars {
            assert!(var < arity);
        }
        Self {
            variable,
            arity,
            sign,
            vars,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Clause {
    literals: Vec<Literal>,
}

#[derive(Clone, Debug, Default)]
pub struct Theory {
    variables: Vec<Variable>,
    clauses: Vec<Clause>,
}

impl Theory {
    pub fn add_variable(&mut self, name: &str, arity: u32) -> usize {
        assert!(!self.variables.iter().any(|v| v.name == name));
        let v = self.variables.len();
        self.variables.push(Variable::new(name, arity));
        v
    }

    pub fn print(&self) {
        println!("variables:");
        for (i, v) in self.variables.iter().enumerate() {
            println!("{}. {}", i, v.fmt(0..v.arity));
            // println!("  {}: {}", i, v.fmt([0, 3, 2].iter().cloned()));
        }
        println!("clauses:");
    }
}
