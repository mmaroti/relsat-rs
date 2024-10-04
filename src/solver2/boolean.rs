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

use std::cmp;
use std::fmt;
use std::ops;

#[derive(Default, Debug, Clone, Copy)]
pub struct Bool(i8);

impl Bool {
    pub fn is_true(self) -> bool {
        self.0 > 0
    }

    pub fn is_undef(self) -> bool {
        self.0 == 0
    }

    pub fn is_false(self) -> bool {
        self.0 < 0
    }
}

pub const FALSE: Bool = Bool(-1);
pub const UNDEF: Bool = Bool(0);
pub const TRUE: Bool = Bool(1);

impl ops::Not for Bool {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(-self.0)
    }
}

impl ops::BitAnd for Bool {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0.min(rhs.0))
    }
}

impl ops::BitOr for Bool {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0.max(rhs.0))
    }
}

impl ops::BitXor<bool> for Bool {
    type Output = Self;

    fn bitxor(self, rhs: bool) -> Self::Output {
        if rhs {
            !self
        } else {
            self
        }
    }
}

impl fmt::Display for Bool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.0.cmp(&0) {
                cmp::Ordering::Greater => "true",
                cmp::Ordering::Less => "false",
                cmp::Ordering::Equal => "undef",
            }
        )
    }
}
