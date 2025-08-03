#![allow(unused)]

use crate::board::Colour;
use std::ops::{Index, IndexMut};

#[rustfmt::skip]
#[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(u8)]
pub enum Piece {
    WP,
    WN,
    WB,
    WR,
    WQ,
    WK,

    BP,
    BN,
    BB,
    BR,
    BQ,
    BK,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(u8)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Copy)]
pub enum OccupancyIndex {
    WhiteOccupancies,
    BlackOccupancies,
    BothOccupancies,
}

pub enum CastlingType {
    WhiteKingside,
    WhiteQueenside,
    BlackKingside,
    BlackQueenside,
}

pub const PIECES: [Piece; 12] = [
    Piece::WP,
    Piece::WB,
    Piece::WN,
    Piece::WR,
    Piece::WQ,
    Piece::WK,
    Piece::BP,
    Piece::BN,
    Piece::BB,
    Piece::BR,
    Piece::BQ,
    Piece::BK,
];

pub const WHITE_PIECES: [Piece; 6] = [
    Piece::WP,
    Piece::WB,
    Piece::WN,
    Piece::WR,
    Piece::WQ,
    Piece::WK,
];

pub const BLACK_PIECES: [Piece; 6] = [
    Piece::BP,
    Piece::BB,
    Piece::BN,
    Piece::BR,
    Piece::BQ,
    Piece::BK,
];
impl<T> Index<Square> for [T; 64] {
    type Output = T;

    fn index(&self, index: Square) -> &Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked(index as usize) }
    }
}

impl<T> IndexMut<Square> for [T; 64] {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}

impl<T> Index<Piece> for [T; 12] {
    type Output = T;

    fn index(&self, index: Piece) -> &Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked(index as usize) }
    }
}

impl<T> IndexMut<Piece> for [T; 12] {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}

impl<T> Index<PieceType> for [T; 6] {
    type Output = T;

    fn index(&self, index: PieceType) -> &Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked(index as usize) }
    }
}

impl<T> IndexMut<PieceType> for [T; 6] {
    fn index_mut(&mut self, index: PieceType) -> &mut Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}

// SAFETY: used for caphist only so in this case PieceType will never be king
impl<T> Index<PieceType> for [T; 5] {
    type Output = T;

    fn index(&self, index: PieceType) -> &Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked(index as usize) }
    }
}

// SAFETY: used for caphist only so in this case PieceType will never be king
impl<T> IndexMut<PieceType> for [T; 5] {
    fn index_mut(&mut self, index: PieceType) -> &mut Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}

impl<T> Index<OccupancyIndex> for [T; 3] {
    type Output = T;

    fn index(&self, index: OccupancyIndex) -> &Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked(index as usize) }
    }
}

impl<T> IndexMut<OccupancyIndex> for [T; 3] {
    fn index_mut(&mut self, index: OccupancyIndex) -> &mut Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}

impl<T> Index<CastlingType> for [T; 4] {
    type Output = T;

    fn index(&self, index: CastlingType) -> &Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked(index as usize) }
    }
}

impl<T> IndexMut<CastlingType> for [T; 4] {
    fn index_mut(&mut self, index: CastlingType) -> &mut Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}

impl Square {
    /// # Safety
    ///
    /// must not pass in a value outside of [0, 63]
    #[must_use]
    pub const unsafe fn from(x: u8) -> Self {
        std::mem::transmute(x)
    }

    /// # Safety
    ///
    /// must not pass in a value greater than 64 - self as u8
    #[must_use]
    pub const unsafe fn add_unchecked(self, x: u8) -> Self {
        Self::from(self as u8 + x)
    }

    /// # Safety
    ///
    /// must not pass in a value which is greater than self as u8
    #[must_use]
    pub const unsafe fn sub_unchecked(self, x: u8) -> Self {
        Self::from(self as u8 - x)
    }
}

impl Piece {
    /// # Safety
    ///
    /// must not pass a value outside of [0, 11]
    #[must_use]
    pub const unsafe fn from(x: u8) -> Self {
        std::mem::transmute(x)
    }

    #[must_use]
    pub fn colour(self) -> Colour {
        match self {
            Piece::WP | Piece::WN | Piece::WB | Piece::WR | Piece::WQ | Piece::WK => Colour::White,
            _ => Colour::Black,
        }
    }

    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Piece::WP => Piece::BP,
            Piece::WN => Piece::BN,
            Piece::WB => Piece::BB,
            Piece::WR => Piece::BR,
            Piece::WQ => Piece::BQ,
            Piece::WK => Piece::BK,
            Piece::BP => Piece::WP,
            Piece::BN => Piece::WN,
            Piece::BB => Piece::WB,
            Piece::BR => Piece::WR,
            Piece::BQ => Piece::WQ,
            Piece::BK => Piece::WK,
        }
    }
}

impl PieceType {
    #[must_use]
    pub fn to_white_piece(self) -> Piece {
        match self {
            PieceType::Pawn => Piece::WP,
            PieceType::Knight => Piece::WN,
            PieceType::Bishop => Piece::WB,
            PieceType::Rook => Piece::WR,
            PieceType::Queen => Piece::WQ,
            PieceType::King => Piece::WK,
        }
    }

    /// # Safety
    ///
    /// must not pass a value outside of [0, 5]
    #[must_use]
    pub const unsafe fn from(x: u8) -> Self {
        std::mem::transmute(x)
    }
}
