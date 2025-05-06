use crate::{
    cube::Sticker,
    math::{Axis, Direction, Face},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge {
    /// Packed field `---onnba`
    data: u8,
}

impl Edge {
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

    #[inline]
    pub const fn a(self) -> Direction {
        Direction::from_bool(self.data & 0b01 != 0)
    }

    #[inline]
    pub const fn b(self) -> Direction {
        Direction::from_bool(self.data & 0b10 != 0)
    }

    pub const fn faces(self) -> [Face; 2] {
        let normal = self.normal();
        let a = Direction::from_bool(self.data & 0b01 != 0);
        let b = Direction::from_bool(self.data & 0b10 != 0);
        [Face::new(normal.next(), a), Face::new(normal.prev(), b)]
    }

    pub fn orientation_axis(self) -> Axis {
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

    pub fn oriented_from_faces([f1, f2]: [Face; 2]) -> Edge {
        let normal = Axis::other(f1.axis(), f2.axis());
        let properly_ordered = f1.axis() == normal.next();
        let (a, b) = if properly_ordered {
            (f1.direction(), f2.direction())
        } else {
            (f2.direction(), f1.direction())
        };

        Edge {
            data: (a.u8() << 0) + (b.u8() << 1) + (normal.u8() << 2),
        }
    }

    /// Given the origin [`Edge`] and the current `edges`, find the current state
    /// of the given edge.
    pub const fn current(self, edges: &[Edge; 12]) -> Edge {
        let index = self.data & 0b01111;
        edges[index as usize]
    }

    fn direction_on_axis(self, axis: Axis) -> Direction {
        assert_ne!(self.normal(), axis);
        if self.normal().next() == axis {
            self.a()
        } else {
            self.b()
        }
    }

    fn face_on_axis(self, axis: Axis) -> Face {
        Face::new(axis, self.direction_on_axis(axis))
    }

    // fn faces_of(index: u8) -> [Face; 2] {
    //     let s = Edge::solved(index);
    //     [
    //         Face::new(s.normal().next(), s.a()),
    //         Face::new(s.normal().next().next(), s.b()),
    //     ]
    // }

    // fn position_index(&self) -> u8 {
    //     self.data % 4
    // }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgePosition(Edge);

impl EdgePosition {
    pub fn from_faces([f1, f2]: [Face; 2]) -> Self {
        EdgePosition(Edge::oriented_from_faces([f1, f2]))
    }

    // TODO: Maybe this should take the array directly. It's just 12 bytes...
    pub fn pick(self, edges: &[Edge; 12]) -> Edge {
        edges[self.0.data as usize]
    }
}

pub fn sticker(edge: Edge, position: EdgePosition, face: Face) -> Sticker {
    let edge_oriented = edge.is_oriented();
    let is_orientation_axis = position.0.orientation_axis() == face.axis();
    if edge_oriented == is_orientation_axis {
        position.0.orientation_face()
    } else {
        assert!(position.0.orientation_face() != position.0.other_face());
        position.0.other_face()
    }
}

// pub fn index_from_faces([f1, f2]: [Face; 2]) -> u8 {
//     assert_ne!(f1.axis(), f2.axis());
//     let normal = Axis::other(f1.axis(), f2.axis());
//     let (a, b) = if normal.next() == f1.axis() {
//         (f1.direction(), f2.direction())
//     } else {
//         (f2.direction(), f1.direction())
//     };

//     println!("{:?}{:?}, {:?}, {:?}", f1, f2, a, b);
//     let offset = |face: Face| {
//         if face.axis() == normal.next() {
//             0
//         } else {
//             assert_eq!(face.axis(), normal.next().next());
//             1
//         }
//     };

//     ((normal as u8) << 2)
//         + ((f1.direction() as u8) << offset(f1))
//         + ((f2.direction() as u8) << offset(f2))

//     // dbg!(a as u8, b as u8, normal as u8);
//     // ((a as u8) << 0) + ((b as u8) << 1) + ((normal as u8) << 2)
// }
