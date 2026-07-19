//! Move notation: the six outer-face turns in the Half-Turn Metric (HTM).
//!
//! This is the move set the solver operates in (`U D L R F B`, each clockwise,
//! double, or counter-clockwise). Wide turns, slice moves (`M E S`), and
//! whole-cube rotations (`x y z`) are a later increment — they require tracking
//! the cube's spatial orientation, which the coordinate model here deliberately
//! does not (its centers are fixed). See `DESIGN.md` §4.2.

use core::fmt;
use core::str::FromStr;

/// One of the six cube faces. The discriminant is the canonical face index used
/// throughout the crate (also the order faces appear in the facelet array).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Face {
    U = 0,
    R = 1,
    F = 2,
    D = 3,
    L = 4,
    B = 5,
}

impl Face {
    /// All six faces in canonical order.
    pub const ALL: [Face; 6] = [Face::U, Face::R, Face::F, Face::D, Face::L, Face::B];

    /// The single letter used in notation.
    pub fn letter(self) -> char {
        match self {
            Face::U => 'U',
            Face::R => 'R',
            Face::F => 'F',
            Face::D => 'D',
            Face::L => 'L',
            Face::B => 'B',
        }
    }
}

/// How far a face is turned. Clockwise is defined looking directly at that face.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Turn {
    /// 90° clockwise — e.g. `U`.
    Cw = 1,
    /// 180° — e.g. `U2`.
    Double = 2,
    /// 90° counter-clockwise — e.g. `U'`.
    Ccw = 3,
}

impl Turn {
    /// The number of clockwise quarter-turns (1, 2, or 3).
    pub fn quarter_turns(self) -> u8 {
        self as u8
    }

    /// The inverse amount: `Cw <-> Ccw`, `Double` is its own inverse.
    pub fn inverse(self) -> Turn {
        match self {
            Turn::Cw => Turn::Ccw,
            Turn::Double => Turn::Double,
            Turn::Ccw => Turn::Cw,
        }
    }

    fn suffix(self) -> &'static str {
        match self {
            Turn::Cw => "",
            Turn::Double => "2",
            Turn::Ccw => "'",
        }
    }
}

/// A single face turn, e.g. `R`, `U'`, `F2`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    pub face: Face,
    pub turn: Turn,
}

impl Move {
    pub const fn new(face: Face, turn: Turn) -> Move {
        Move { face, turn }
    }

    /// The move that undoes this one (same face, opposite amount).
    pub fn inverse(self) -> Move {
        Move { face: self.face, turn: self.turn.inverse() }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.face.letter(), self.turn.suffix())
    }
}

/// Error returned when a move string cannot be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseMoveError {
    /// The token was empty.
    Empty,
    /// The first character was not one of `U R F D L B`.
    UnknownFace(char),
    /// The suffix was present but not `'` or `2`, or there was trailing junk.
    BadSuffix(String),
}

impl fmt::Display for ParseMoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseMoveError::Empty => write!(f, "empty move token"),
            ParseMoveError::UnknownFace(c) => write!(f, "unknown face '{c}'"),
            ParseMoveError::BadSuffix(s) => write!(f, "bad move suffix \"{s}\""),
        }
    }
}

impl std::error::Error for ParseMoveError {}

impl FromStr for Move {
    type Err = ParseMoveError;

    /// Parse a single move token such as `"R"`, `"U'"`, or `"F2"`.
    fn from_str(s: &str) -> Result<Move, ParseMoveError> {
        let mut chars = s.chars();
        let face = match chars.next() {
            None => return Err(ParseMoveError::Empty),
            Some('U') => Face::U,
            Some('R') => Face::R,
            Some('F') => Face::F,
            Some('D') => Face::D,
            Some('L') => Face::L,
            Some('B') => Face::B,
            Some(c) => return Err(ParseMoveError::UnknownFace(c)),
        };
        let rest = chars.as_str();
        let turn = match rest {
            "" => Turn::Cw,
            "'" | "’" => Turn::Ccw, // accept the typographic apostrophe too
            "2" => Turn::Double,
            other => return Err(ParseMoveError::BadSuffix(other.to_string())),
        };
        Ok(Move { face, turn })
    }
}

/// Parse a whitespace-separated sequence such as `"R U R' U'"`.
pub fn parse_sequence(s: &str) -> Result<Vec<Move>, ParseMoveError> {
    s.split_whitespace().map(Move::from_str).collect()
}

/// Format a slice of moves back into a space-separated notation string.
pub fn format_sequence(moves: &[Move]) -> String {
    moves
        .iter()
        .map(|m| m.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic() {
        assert_eq!("R".parse::<Move>().unwrap(), Move::new(Face::R, Turn::Cw));
        assert_eq!("U'".parse::<Move>().unwrap(), Move::new(Face::U, Turn::Ccw));
        assert_eq!("F2".parse::<Move>().unwrap(), Move::new(Face::F, Turn::Double));
    }

    #[test]
    fn parse_errors() {
        assert_eq!("".parse::<Move>(), Err(ParseMoveError::Empty));
        assert_eq!("X".parse::<Move>(), Err(ParseMoveError::UnknownFace('X')));
        assert!(matches!("U3".parse::<Move>(), Err(ParseMoveError::BadSuffix(_))));
        assert!(matches!("R''".parse::<Move>(), Err(ParseMoveError::BadSuffix(_))));
    }

    #[test]
    fn display_roundtrips() {
        for face in Face::ALL {
            for turn in [Turn::Cw, Turn::Double, Turn::Ccw] {
                let m = Move::new(face, turn);
                assert_eq!(m.to_string().parse::<Move>().unwrap(), m);
            }
        }
    }

    #[test]
    fn sequence_roundtrips() {
        let s = "R U R' U' F2 D";
        let moves = parse_sequence(s).unwrap();
        assert_eq!(moves.len(), 6);
        assert_eq!(format_sequence(&moves), s);
    }

    #[test]
    fn inverse() {
        assert_eq!("R".parse::<Move>().unwrap().inverse(), "R'".parse().unwrap());
        assert_eq!("U2".parse::<Move>().unwrap().inverse(), "U2".parse().unwrap());
    }
}
