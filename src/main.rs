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

#![allow(dead_code)]

mod buffer;
mod shape;
mod theory;

// use shape::*;
use theory::*;

fn main() {
    let mut theory: Theory = Default::default();
    let _mul = theory.add_variable("mul", 3);
    let _inv = theory.add_variable("inv", 2);
    let _one = theory.add_variable("one", 1);

    theory.print()
}
