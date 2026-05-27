use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct UserId(Uuid);

impl From<Uuid> for UserId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl From<UserId> for Uuid {
    fn from(id: UserId) -> Uuid {
        id.0
    }
}

impl UserId {
    pub fn next() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}
