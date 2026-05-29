//! Модуль электронных почт.
//!
//! Обеспечивает инкапсуляцию строковых данных в тип [Email]
//! на протяжении всего жизненного цикла приложения.

use std::fmt::{Display, Formatter};

/// Объект-значение (Value Object),
/// позволяет иметь постоянно валидный адрес электронной почты.
///
/// Создать экземпляр можно только через реализацию [`TryFrom`],
/// которая проверяет соответствие бизнес-правилам.
#[derive(Clone, Debug, PartialEq)]
pub struct Email(String);

impl TryFrom<String> for Email {
    type Error = EmailError;

    /// Пытается создать [`Email`] из [`String`].
    ///
    /// # Errors
    ///
    /// Возвращает [`EmailError`] если строка пустая,
    /// слишком длинная или адрес электронной почты не соответствует паттерну.
    ///
    /// # Examples
    ///
    /// ```
    /// use untitled::domain::email::Email;
    /// let email_string = String::from("test_email@example.com");
    /// let email = Email::try_from(email_string).unwrap();
    /// assert_eq!(email.as_str(), "test_email@example.com");
    ///
    /// let bad_email_string = String::from("bad_test_email");
    /// let bad_email = Email::try_from(bad_email_string);
    /// assert!(bad_email.is_err());
    /// ```
    fn try_from(email: String) -> Result<Self, Self::Error> {
        validate_email(email)
    }
}

/// Пытается создать [`Email`] из [`&str`].
///
/// # Errors
///
/// Возвращает [`EmailError`] если строка пустая,
/// слишком длинная или адрес электронной почты не соответствует паттерну.
///
/// # Example
///
/// ```
/// use untitled::domain::email::Email;
/// let email = Email::try_from("test_email@example.com").unwrap();
/// assert_eq!(email.as_str(), "test_email@example.com");
///
///
/// let bad_email = Email::try_from("bad_test_email");
/// assert!(bad_email.is_err());
/// ```
impl TryFrom<&str> for Email {
    type Error = EmailError;

    fn try_from(email: &str) -> Result<Self, Self::Error> {
        validate_email(email.to_string())
    }
}

/// Внутренняя функция валидации электронной почты.
fn validate_email(email: String) -> Result<Email, EmailError> {
    if email.is_empty() {
        return Err(EmailError::Empty);
    }

    if email.len() > 255 {
        return Err(EmailError::TooLong {
            max: 255,
            actual: email.len(),
        });
    }

    let at_index = email
        .find('@')
        .ok_or_else(|| EmailError::InvalidFormat(email.clone()))?;

    let domain = &email[at_index + 1..];

    if !domain.contains(".") || domain.starts_with(".") || domain.ends_with(".") {
        return Err(EmailError::InvalidFormat(email));
    }

    if domain.split('.').last().unwrap_or("").len() < 2 {
        return Err(EmailError::InvalidFormat(email));
    }

    Ok(Email(email))
}

/// Значение по умолчанию.
///
/// TODO В продакшене убрать.
impl Default for Email {
    fn default() -> Self {
        Self("placeholder@example.com".into())
    }
}

/// Позволяет переделывать имеющийся типа [`Email`] в типа [`String`].
impl From<Email> for String {
    fn from(email: Email) -> String {
        email.0
    }
}

/// Определяет, как объект электронной почты будет форматироваться в виде строки.
impl Display for Email {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Email {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Enum реализующий все возможные виды ошибок при валидации электронной почты.
#[derive(Debug, PartialEq)]
pub enum EmailError {
    /// Передана пустая строка
    Empty,
    /// Строка не соответствуем паттернам
    InvalidFormat(String),
    /// Строка превышает допустимые лимиты (255)
    TooLong { max: usize, actual: usize },
}

/// Определяет, как объект ошибки будет форматироваться в виде строки.
impl Display for EmailError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailError::Empty => write!(f, "Empty email"),
            EmailError::InvalidFormat(e) => write!(f, "Invalid format: {}", e),
            EmailError::TooLong { max, actual } => {
                write!(f, "Too long email: {} > {}", actual, max)
            }
        }
    }
}
