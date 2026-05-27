use crate::domain::email::Email;
use crate::domain::user_id::UserId;
use std::fmt::Display;
use uuid::Uuid;

#[derive(Clone)]
pub struct User {
    id: UserId,
    pub username: Option<String>,
    pub email: Email,
    pub role: UserRole,
}

impl User {
    pub fn new(
        id: UserId,
        username: Option<impl Into<String>>,
        email: Email,
        role: UserRole,
    ) -> Self {
        Self {
            id,
            username: username.map(Into::into),
            email,
            role,
        }
    }

    #[allow(dead_code)]
    pub fn id(&self) -> Uuid {
        self.id.into()
    }
}

impl Default for User {
    fn default() -> Self {
        Self::new(
            UserId::next(),
            None::<String>,
            Email::default(),
            UserRole::Player(Player::default()),
        )
    }
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(
                f,
                "\nID: {}, \nName: {}, \nEmail: {}, \nRole: {}",
                self.id.as_uuid(),
                self.username.as_deref().unwrap_or("None"),
                self.email.as_str(),
                self.role,
            )
        } else {
            write!(
                f,
                "ID: {}, Name: {}, Email: {}, Role: {}",
                self.id.as_uuid(),
                self.username.as_deref().unwrap_or("None"),
                self.email.as_str(),
                self.role,
            )
        }
    }
}

impl User {
    #[allow(dead_code)]
    pub fn update_from(&mut self, other: &Self) {
        *self = other.clone();
    }
}

#[derive(Clone)]
pub struct Admin {
    access_level: u16,
}

#[derive(Clone, Default)]
pub struct Player;

#[allow(dead_code)]
impl Admin {
    pub fn new(level: u16) -> Self {
        Self {
            access_level: level,
        }
    }
}

#[derive(Clone)]
pub struct Guest;

#[derive(Clone)]
pub enum UserRole {
    Admin(Admin),
    Player(Player),
    Guest(Guest),
}

impl Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admin(admin) => write!(f, "Admin (access_level: {})", admin.access_level),
            Self::Player(_) => write!(f, "Player"),
            Self::Guest(_) => write!(f, "Guest"),
        }
    }
}

impl UserRole {
    #[allow(dead_code)]
    pub fn is_powerful_admin(&self) -> bool {
        matches!(&self, Self::Admin(admin) if admin.access_level == admin.access_level)
    }
}
