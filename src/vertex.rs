use glam::Vec2;

enum CardinalAxis {
    Vert,
    Horz,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrdinalAxis {
    Inc,
    Dec,
}

impl OrdinalAxis {
    fn neg(self) -> Self {
        match self {
            Self::Inc => Self::Dec,
            Self::Dec => Self::Inc,
        }
    }

    fn ordinals(self) -> [Ordinal; 2] {
        match self {
            Self::Inc => [Ordinal::Northeast, Ordinal::Southwest],
            Self::Dec => [Ordinal::Northwest, Ordinal::Southeast],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Cardinal {
    East,
    North,
    South,
    West,
}

impl Cardinal {
    fn axis(self) -> CardinalAxis {
        match self {
            Self::North | Self::South => CardinalAxis::Vert,
            Self::East | Self::West => CardinalAxis::Horz,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Ordinal {
    Northeast,
    Northwest,
    Southeast,
    Southwest,
}

impl Ordinal {
    fn axis(self) -> OrdinalAxis {
        match self {
            Self::Northeast | Self::Southwest => OrdinalAxis::Inc,
            Self::Northwest | Self::Southeast => OrdinalAxis::Dec,
        }
    }

    fn neg(self) -> Self {
        match self {
            Self::Northeast => Self::Southwest,
            Self::Northwest => Self::Southeast,
            Self::Southeast => Self::Northwest,
            Self::Southwest => Self::Northeast,
        }
    }

    fn contains_cardinal(self, other: Cardinal) -> bool {
        matches!(
            (self, other),
            (Self::Northeast, Cardinal::North)
                | (Self::Northeast, Cardinal::East)
                | (Self::Northwest, Cardinal::North)
                | (Self::Northwest, Cardinal::West)
                | (Self::Southeast, Cardinal::South)
                | (Self::Southeast, Cardinal::East)
                | (Self::Southwest, Cardinal::South)
                | (Self::Southwest, Cardinal::West)
        )
    }

    fn shared_cardinal(self, other: Self) -> Option<Cardinal> {
        match (self, other) {
            (Self::Northeast, Self::Northwest) | (Self::Northwest, Self::Northeast) => {
                Some(Cardinal::North)
            }
            (Self::Southeast, Self::Southwest) | (Self::Southwest, Self::Southeast) => {
                Some(Cardinal::South)
            }
            (Self::Northeast, Self::Southeast) | (Self::Southeast, Self::Northeast) => {
                Some(Cardinal::East)
            }
            (Self::Northwest, Self::Southwest) | (Self::Southwest, Self::Northwest) => {
                Some(Cardinal::West)
            }
            _ => None,
        }
    }

    fn reflect(self, other: CardinalAxis) -> Self {
        match (self, other) {
            (Self::Northeast, CardinalAxis::Vert) | (Self::Southwest, CardinalAxis::Horz) => {
                Self::Northwest
            }
            (Self::Northeast, CardinalAxis::Horz) | (Self::Southwest, CardinalAxis::Vert) => {
                Self::Southeast
            }
            (Self::Northwest, CardinalAxis::Vert) | (Self::Southeast, CardinalAxis::Horz) => {
                Self::Northeast
            }
            (Self::Northwest, CardinalAxis::Horz) | (Self::Southeast, CardinalAxis::Vert) => {
                Self::Southwest
            }
        }
    }

    pub(crate) fn parts(self) -> (bool, bool) {
        match self {
            Self::Northeast => (true, true),
            Self::Northwest => (true, false),
            Self::Southeast => (false, true),
            Self::Southwest => (false, false),
        }
    }

    pub(crate) fn as_vec2(self) -> Vec2 {
        match self {
            Self::Northeast => Vec2::ONE,
            Self::Northwest => Vec2::new(-1., 1.),
            Self::Southeast => Vec2::new(1., -1.),
            Self::Southwest => Vec2::NEG_ONE,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum VertexNormal {
    None,
    One(Ordinal),
    TwoAdj(Cardinal),
    TwoDiag(OrdinalAxis),
    Three(Ordinal),
    Four,
}

impl VertexNormal {
    pub(crate) fn normals(self) -> impl IntoIterator<Item = (Ordinal, bool)> {
        match self {
            Self::One(ordinal) => vec![(ordinal, false)],
            Self::TwoDiag(axis) => axis
                .neg()
                .ordinals()
                .into_iter()
                .map(|ordinal| (ordinal, true))
                .collect(),
            Self::Three(ordinal) => vec![(ordinal, true)],
            _ => vec![],
        }
    }

    pub(crate) fn add(self, rhs: Ordinal) -> Self {
        match self {
            Self::None => Self::One(rhs),
            Self::One(lhs) if lhs != rhs => lhs
                .shared_cardinal(rhs)
                .map(Self::TwoAdj)
                .unwrap_or_else(|| Self::TwoDiag(lhs.axis())),
            Self::TwoAdj(dir) if !rhs.contains_cardinal(dir) => {
                Self::Three(rhs.reflect(dir.axis()).neg())
            }
            Self::TwoDiag(axis) if axis != rhs.axis() => Self::Three(rhs),
            Self::Three(lhs) if lhs == rhs.neg() => Self::Four,
            _ => panic!("invalid add {self:?} + {rhs:?}"),
        }
    }

    pub(crate) fn add_assn(&mut self, rhs: Ordinal) {
        *self = self.add(rhs);
    }
}
