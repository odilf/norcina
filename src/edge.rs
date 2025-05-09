use std::{array, fmt, mem::transmute};

use crate::{
    cube::Sticker,
    math::{Axis, Direction, Face},
    mov::{Amount, Move},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge {
    /// Packed field `---onnab`
    // /// Packed field `-oxxyyzz`
    // ///
    // /// Coordinates are stored in 2-bit 2's complement. That is, the first bit is 1, the second -2. So
    // /// -1 = 0b10
    // ///  0 = 0b00
    // ///  1 = 0b01
    // ///
    // /// And you can also represent -2, but that's an invalid bit-pattern.
    // ///
    // /// This is mostly to keep 0 at 0b00.
    data: u8,
}

impl Edge {
    #[inline]
    pub const fn a(self) -> Direction {
        Direction::from_bool(self.data & 0b01 != 0)
    }

    #[inline]
    pub const fn b(self) -> Direction {
        Direction::from_bool(self.data & 0b10 != 0)
    }

    pub const fn solved(index: u8) -> Edge {
        assert!(index < 12);
        Edge { data: index }
    }

    #[inline]
    pub const fn is_oriented(self) -> bool {
        //            ---onnba
        self.data & 0b00010000 == 0
    }

    #[inline]
    pub const fn normal(self) -> Axis {
        Axis::from_u8((self.data >> 2) & 0b11)
    }

    pub const fn faces(self) -> [Face; 2] {
        let normal = self.normal();
        let a = Direction::from_bool(self.data & 0b01 != 0);
        let b = Direction::from_bool(self.data & 0b10 != 0);
        [Face::new(normal.next(), a), Face::new(normal.prev(), b)]
    }

    /// Given the origin [`Edge`] and the current `edges`, find the current state
    /// of the given edge.
    pub const fn current(self, edges: &[Edge; 12]) -> Edge {
        let index = self.data & 0b01111;
        edges[index as usize]
    }

    fn position(self) -> EdgePosition {
        // SAFETY: Both [`Edge`] and [`EdgePosition`] are a single `u8` in memory.
        unsafe { transmute(self) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgePosition {
    data: u8,
}

impl EdgePosition {
    #[inline]
    pub const fn a(self) -> Direction {
        Direction::from_bool(self.data & 0b01 != 0)
    }

    #[inline]
    pub const fn b(self) -> Direction {
        Direction::from_bool(self.data & 0b10 != 0)
    }

    pub fn from_faces([f1, f2]: [Face; 2]) -> Self {
        let normal = Axis::other(f1.axis(), f2.axis());
        let properly_ordered = f1.axis() == normal.next();
        let (a, b) = if properly_ordered {
            (f1.direction(), f2.direction())
        } else {
            (f2.direction(), f1.direction())
        };

        EdgePosition {
            data: (a.u8() << 0) + (b.u8() << 1) + (normal.u8() << 2),
        }
    }

    // TODO: Maybe this should take the array directly. It's just 12 bytes...
    pub fn pick(self, edges: &[Edge; 12]) -> Edge {
        edges[self.data as usize]
    }

    #[inline]
    pub const fn normal(self) -> Axis {
        Axis::from_u8((self.data >> 2) & 0b11)
    }

    #[inline]
    pub fn from_index(index: u8) -> Self {
        assert!(index < 12);
        EdgePosition { data: index }
    }

    #[inline]
    fn direction_on_axis(self, axis: Axis) -> Direction {
        assert_ne!(
            self.normal(),
            axis,
            "Tried to get the direction along normal"
        );
        if self.normal().next() == axis {
            self.a()
        } else {
            self.b()
        }
    }

    #[inline]
    fn face_on_axis(self, axis: Axis) -> Face {
        Face::new(axis, self.direction_on_axis(axis))
    }

    #[inline]
    pub fn orientation_axis(self) -> Axis {
        // TODO: Do this with bit twiddling
        if self.normal() == Axis::Y {
            Axis::Z
        } else {
            Axis::Y
        }
    }

    pub fn non_orientation_axis(self) -> Axis {
        match self.normal() {
            Axis::X => Axis::Z,
            Axis::Y => Axis::X,
            Axis::Z => Axis::X,
        }
    }

    pub fn orientation_face(self) -> Face {
        self.face_on_axis(self.orientation_axis())
    }

    pub fn other_face(self) -> Face {
        self.face_on_axis(self.non_orientation_axis())
    }

    pub fn contains_face(self, face: Face) -> bool {
        face.axis() != self.normal() && face.direction() == self.direction_on_axis(face.axis())
    }
}

pub fn sticker(edge: Edge, position: EdgePosition, face: Face) -> Sticker {
    assert_ne!(face.axis(), position.normal());
    if (position.orientation_axis() == face.axis()) == edge.is_oriented() {
        edge.position().orientation_face()
    } else {
        edge.position().other_face()
    }
}

pub fn move_pieces(edges: [Edge; 12], mov: Move) -> [Edge; 12] {
    array::from_fn(|i| {
        let position = EdgePosition::from_index(i as u8);
        let (dir_mov, other_axis_offset) = if mov.face().axis() == position.normal().next() {
            (position.a(), 1)
        } else if mov.face().axis() == position.normal().prev() {
            (position.b(), 0)
        } else {
            return edges[i];
        };

        if dir_mov != mov.face().direction() {
            return edges[i];
        }

        assert_ne!(mov.face().axis(), position.normal());
        // Example to what happens to bits on an R move:
        //    RU    ->    RB    ->    RD    ->    RF    ->    RU
        // ---01000 -> ---00101 -> ---01010 -> ---00100 -> ---01000
        // ---onnab    ---onnab    ---onnab    ---onnab    ---onnab
        let new_edge_pos = match (mov.amount(), mov.face().direction()) {
            (Amount::Double, _) => return edges[i ^ (0b1 << other_axis_offset) as usize],
            (Amount::Single, Direction::Positive) | (Amount::Reverse, Direction::Negative) => {
                // Normal switches to the one that isn't the mov axis.
                // let normal = Axis::other(mov.face().axis(), position.normal());

                // `i` and `j` are the axes that get rotated.
                // Specifically (p[i], p[j]) -> (p[j], -p[i])   (where `p` is position).
                let i = (mov.face().axis().u8() + 1) % 3;
                let j = (mov.face().axis().u8() + 2) % 3;

                // Here, with ternary coordinates, one is 0 and the other is either 1 or -1.
                // If `i` is 0, then `p[j]` stays as-is. Otherwise, `p[j]` gets flipped.
                let other_face = if i == position.normal().u8() {
                    // assert!(j == position.normal().u8());
                    let axis = Axis::from_u8(i);
                    let dir = position.direction_on_axis(Axis::from_u8(j)).flip();
                    Face::new(axis, dir)
                } else {
                    let axis = Axis::from_u8(j);
                    let dir = position.direction_on_axis(Axis::from_u8(i));
                    Face::new(axis, dir)
                };

                let faces = [other_face, mov.face()];
                EdgePosition::from_faces(faces)
            }
            (Amount::Reverse, Direction::Positive) | (Amount::Single, Direction::Negative) => {
                // `i` and `j` are the axes that get rotated.
                // Specifically (p[i], p[j]) -> (p[j], -p[i])   (where `p` is position).
                let i = (mov.face().axis().u8() + 2) % 3;
                let j = (mov.face().axis().u8() + 1) % 3;

                // Here, with ternary coordinates, one is 0 and the other is either 1 or -1.
                // If `i` is 0, then `p[j]` stays as-is. Otherwise, `p[j]` gets flipped.
                let other_face = if i == position.normal().u8() {
                    // assert!(j == position.normal().u8());
                    let axis = Axis::from_u8(i);
                    let dir = position.direction_on_axis(Axis::from_u8(j)).flip();
                    Face::new(axis, dir)
                } else {
                    let axis = Axis::from_u8(j);
                    let dir = position.direction_on_axis(Axis::from_u8(i));
                    Face::new(axis, dir)
                };

                let faces = [other_face, mov.face()];
                EdgePosition::from_faces(faces)
            }
        };

        let out = new_edge_pos.pick(&edges);

        // single Y-axis moves flip orientation.
        // This value is 1 if move is on x-axis, 0 otherwise.
        let is_on_z_axis = mov.face().axis().u8() >> 1;
        assert!(if mov.face().axis() == Axis::Z {
            is_on_z_axis == 1
        } else {
            is_on_z_axis == 0
        });

        Edge {
            data: out.data ^ (is_on_z_axis << 4),
        }
    })
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b] = self.faces();
        write!(
            f,
            "{a:?}{b:?} ({})",
            if self.is_oriented() { '✓' } else { 'x' }
        )
    }
}

impl fmt::Display for EdgePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Bodge
        let [a, b] = Edge { data: self.data }.faces();
        write!(f, "{a:?}{b:?}")
    }
}

#[cfg(test)]
mod tests {
    use quickcheck::{Arbitrary, Gen};

    use super::*;

    impl Arbitrary for Edge {
        fn arbitrary(g: &mut Gen) -> Self {
            let normal = Axis::arbitrary(g);
            let a = Direction::arbitrary(g);
            let b = Direction::arbitrary(g);
            Edge {
                data: (a.u8() << 0) + (b.u8() << 1) + (normal.u8() << 2),
            }
        }
    }
}
