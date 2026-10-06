#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Authority {
    Real,
    Model,
    Imagined,
}

impl Authority {
    pub fn can_write_factual(self) -> bool {
        matches!(self, Authority::Real)
    }
}
