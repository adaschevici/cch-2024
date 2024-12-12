use std::fmt;

enum BoardLocation {
    Cookie,
    Empty,
    Milk,
    Wall,
}

impl fmt::Display for TrafficLight {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BoardLocation::Cookie => write!(f, "🍪"),
            BoardLocation::Empty => write!(f, "⬛"),
            BoardLocation::Milk => write!(f, "🥛"),
            BoardLocation::Wall => write!(f, "⬜"),
        }
    }
}
