/*
* Copyright (C) 2019-2024, Miklos Maroti
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

use crate::solver1::bitops;
use crate::solver1::buffer;
use crate::solver1::shape;

mod solver;

pub fn main() {
    let mut sol: solver::Solver = Default::default();

    let set = sol.add_domain("set".into(), 7);
    let _one = sol.add_relation("one".into(), vec![set]);
    let _inv = sol.add_relation("inv".into(), vec![set, set]);
    let _mul = sol.add_relation("mul".into(), vec![set, set, set]);

    sol.print();
}
