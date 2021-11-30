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

#[derive(Clone, Copy, Debug)]
pub struct Symbol {
    pub arity: u32,
    pub name: &'static str,
}

impl Symbol {
    pub fn new(name: &'static str, arity: u32) -> Self {
        Self { arity, name }
    }
}

#[derive(Clone, Debug)]
pub struct Literal {
    symbol: Symbol,
    arity: u32,
    sign: bool,
    vars: Vec<u32>,
}

impl Literal {
    pub fn new(symbol: Symbol, sign: bool, arity: u32, vars: Vec<u32>) -> Self {
        assert!(vars.len() == symbol.arity as usize);
        for &var in &vars {
            assert!(var < arity);
        }
        Self {
            symbol,
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
