use uuid::Uuid;

#[derive(Debug)]
pub struct LibraryItemId(Uuid);

impl LibraryItemId {
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for LibraryItemId {
    fn default() -> Self {
        Self(Uuid::now_v7())
    }
}

impl From<Uuid> for LibraryItemId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}
