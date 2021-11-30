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

// mod bitvec;
// mod compute;
// mod theory;
mod shape;
use shape::*;

fn main() {
    let shape = Shape::new(vec![2, 3, 2]);
    println!("{:?}", shape.size());

    let view1 = View::new(&shape);
    println!("{:?}", view1);
    println!("{:?}", view1.positions().collect::<Vec<usize>>());

    let view2 = view1.polymer(&Shape::new(vec![3, 4, 2]), &[2, 0, 2]);
    println!("{:?}", view2);
    println!("{:?}", view2.positions().collect::<Vec<usize>>());

    let view3 = view1.permute(&[2, 0, 1]);
    println!("{:?}", view3);
    println!("{:?}", view3.positions().collect::<Vec<usize>>());

    let view4 = view3.simplify();
    println!("{:?}", view4);
    println!("{:?}", view4.positions().collect::<Vec<usize>>());
}
