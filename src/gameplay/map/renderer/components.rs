use std::fmt::Display;

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum MaterialKey {
    Beach,
    Mountain,
    Water,
    Grass,
    Forest,
    Player,
    Debug,
}

impl Display for MaterialKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            MaterialKey::Beach => "Beach",
            MaterialKey::Mountain => "Mountain",
            MaterialKey::Water => "Water",
            MaterialKey::Grass => "Grass",
            MaterialKey::Forest => "Forest",
            MaterialKey::Player => "Player",
            MaterialKey::Debug => "Debug",
        };

        write!(f, "{name}")
    }
}
