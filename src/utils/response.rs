//! Модуль, отвечающий за формирование и отправку ответов сервера (Response).
//!
//! Предоставляет удобный построитель (Builder) ответов и перечисление
//! стандартных статус-кодов протокола HTTP.

use bytes::Bytes;
use cookie::Cookie;
use http::header::{CONNECTION, CONTENT_TYPE};
use http::status::StatusCode;
use http::{HeaderMap, HeaderName, HeaderValue};
use std::fmt::Display;
use std::io::{Error, Write};
use http::response as HttpResponse;

/// Структура HTTP-ответа.
///
/// Использует паттерн «Строитель» (Builder) для поэтапной настройки
/// статуса, типа контента и тела ответа перед отправкой.
#[derive(Debug, PartialEq)]
pub struct Response<B> {
    status: StatusCode,
    headers: HeaderMap,
    body: B,
}

impl Display for Response<Bytes> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let raw_body = String::from_utf8_lossy(&self.body);

        if f.alternate() {
            write!(
                f,
                "\nStatus: {},\nBody:\n{}",
                self.status.as_u16(),
                raw_body.trim()
            )
        } else {
            let clean_body = raw_body.replace("\n", " ").replace("\r", "");
            write!(
                f,
                "Status: {}, Body: {}",
                self.status.as_u16(),
                clean_body.trim()
            )
        }
    }
}

impl Default for Response<Bytes> {
    fn default() -> Self {
        Self {
            status: StatusCode::OK,
            headers: Default::default(),
            body: Bytes::new(),
        }
    }
}

impl Response<Bytes> {
    /// Создает базовый пустой ответ со статусом `200 OK` и типом `text/html`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Возвращает текущий HTTP-статус ответа.
    /// (HttpStatus копируемый, поэтому возвращаем по значению)
    pub fn get_status(&self) -> StatusCode {
        self.status
    }

    pub fn get_content_type(&self) -> Option<&str> {
        self.headers.get(CONTENT_TYPE).and_then(|v| v.to_str().ok())
    }

    /// Возвращает ссылку на бинарное тело ответа.
    pub fn get_body(&self) -> &[u8] {
        self.body.as_ref()
    }

    /// Пытается интерпретировать тело ответа как UTF-8 строку.
    /// Полезно для логирования или отладки.
    pub fn body_as_str(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.body)
    }

    /// Устанавливает HTTP-статус ответа.
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn content_type(mut self, value: &'static str) -> Self {
        let header_value = HeaderValue::from_static(value);
        self.headers.insert(CONTENT_TYPE, header_value);
        self
    }

    /// Устанавливаем кастомные заголовки.
    pub fn header(mut self, key: &str, value: impl AsRef<str>) -> Self {
        if let Ok(header_name) = HeaderName::from_bytes(key.as_bytes()) {
            if let Ok(value) = HeaderValue::from_str(value.as_ref()) {
                self.headers.insert(header_name, value);
            } else {
                eprintln!(
                    "[WARN] Некорректные символы в значении заголовка: {}",
                    value.as_ref()
                );
            }
        } else {
            eprintln!("[WARN] Некорректное имя заголовка: {}", key);
        }
        self
    }

    /// Записывает данные в тело ответа.
    ///
    /// Принимает любые типы, конвертируемые в `Bytes` (например, `&str`, `String`, `Vec<u8>`, `&[u8]`).
    pub fn body(mut self, body: impl Into<Bytes>) -> Self {
        self.body = body.into();
        self
    }

    /// Добавляет заголовок `Set-Cookie`.
    pub fn set_cookie(self, cookie: Cookie) -> Self {
        self.header("Set-Cookie", cookie.to_string())
    }

    /// Сериализует ответ в формате HTTP/1.1, отправляет его в поток и возвращает `Self` для логов.
    ///
    /// Метод полностью забирает владение структурой (`self`). Работает с любым объектом,
    /// реализующим трейт [`Write`].
    pub fn send(self, stream: &mut impl Write) -> Result<Self, Error> {
        let mut header_buf = Vec::with_capacity(512);

        let status_phrase = self.status.canonical_reason().unwrap_or("");

        writeln!(
            &mut header_buf,
            "HTTP/1.1 {} {}\r",
            self.status.as_u16(),
            status_phrase
        )?;
        let has_body = !self.status.is_informational()
            && self.status != StatusCode::NO_CONTENT
            && self.status != StatusCode::NOT_MODIFIED;

        if has_body {
            writeln!(&mut header_buf, "content-length: {}\r", self.body.len())?;
        }

        for (name, value) in self.headers.iter() {
            header_buf.write_all(name.as_str().as_bytes())?;
            header_buf.write_all(b": ")?;
            header_buf.write_all(value.as_bytes())?;
            header_buf.write_all(b"\r\n")?;
        }

        if !self.headers.contains_key(CONNECTION) {
            header_buf.write_all(b"connection: close\r\n")?;
        }

        header_buf.write_all(b"\r\n")?;

        stream.write_all(&header_buf)?;

        if has_body && !self.body.is_empty() {
            stream.write_all(&self.body)?;
        }

        stream.flush()?;

        Ok(self)
    }
}

/// Ассоциированные статические методы для быстрого формирования ответов в стиле Spring.
///
/// Позволяют генерировать готовые объекты ответов без вызова конструктора [`Response::new`].
///
/// # Examples
///
/// ```no_run
/// use webshark::utils::response::Response;
///
/// // Быстрый пустой успешный ответ
/// let res_ok = Response::ok();
///
/// // Создание ресурса с JSON-телом
/// let json_data = r#"{"id": 42}"#;
/// let res_created = Response::created_body(json_data).content_type("application/json");
///
/// // Ошибка 404 с кастомным сообщением
/// let res_not_found = Response::not_found_body("Страница не существует");
/// ```
impl Response<Bytes> {
    /// Быстрый пустой ответ `200 OK`.
    pub fn ok() -> Self {
        Self::new().status(StatusCode::OK)
    }

    /// Быстрый ответ `200 OK` со строковым или бинарным телом.
    ///
    /// Принимает любые типы, конвертируемые в `Vec<u8>`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use webshark::utils::response::Response;
    ///
    /// let res_str = Response::ok_body("Привет, мир!");
    /// let res_bytes = Response::ok_body(b"Hello".to_vec());
    /// ```
    pub fn ok_body(body: impl Into<Bytes>) -> Self {
        Self::new().status(StatusCode::OK).body(body)
    }

    /// Быстрый пустой ответ `201 Created`.
    pub fn created() -> Self {
        Self::new().status(StatusCode::CREATED)
    }

    /// Быстрый ответ `201 Created` со строковым или бинарным телом ресурса.
    ///
    /// Принимает любые типы, конвертируемые в `Vec<u8>`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use webshark::utils::response::Response;
    ///
    /// let res = Response::created_body("User registered successfully");
    /// ```
    pub fn created_body(body: impl Into<Bytes>) -> Self {
        Self::new().status(StatusCode::CREATED).body(body)
    }

    /// Быстрый пустой ответ `204 No Content`.
    ///
    /// Гарантирует строгое отсутствие заголовков длины, типа и самого тела при отправке.
    pub fn no_content() -> Self {
        Self::new().status(StatusCode::NO_CONTENT)
    }

    /// Быстрый пустой ответ `400 Bad Request`.
    pub fn bad_request() -> Self {
        Self::new().status(StatusCode::BAD_REQUEST)
    }

    /// Быстрый пустой ответ `404 Not Found`.
    pub fn not_found() -> Self {
        Self::new().status(StatusCode::NOT_FOUND)
    }

    pub fn internal_error() -> Self {
        Self::new().status(StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn internal_error_body(body: impl Into<Bytes>) -> Self {
        Self::new()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body)
    }

    /// Быстрый ответ `404 Not Found` с текстовым описанием ошибки.
    ///
    /// Принимает любые типы, конвертируемые в `Vec<u8>`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use webshark::utils::response::Response;
    ///
    /// let res = Response::not_found_body("Файл index.html не найден на диске");
    /// ```
    pub fn not_found_body(message: impl Into<Vec<u8>>) -> Self {
        Self::new()
            .status(StatusCode::NOT_FOUND)
            .body(message.into())
    }

    pub fn forbidden() -> Self {
        Self::new().status(StatusCode::FORBIDDEN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod response_display {
        use super::*;

        #[test]
        fn test_standard_display() {
            let response = Response::new().body("Test");
            assert_eq!(
                format!("{}", response),
                "Status: 200, Body: Test"
            );
        }

        #[test]
        fn test_alternate_display() {
            let response = Response::new().body("Test");
            assert_eq!(
                format!("{:#}", response),
                "\nStatus: 200,\nBody:\nTest"
            );
        }
    }

    mod response_builder {
        use super::*;

        #[test]
        fn test_builder_methods() {
            let response = Response::new()
                .status(StatusCode::OK)
                .content_type("application/json; charset=utf-8")
                .body("Test");
            assert_eq!(response.status, StatusCode::OK);
            assert_eq!(response.get_content_type(), Some("application/json; charset=utf-8"));
            assert_eq!(response.body.as_ref(), b"Test");
        }

        #[test]
        fn test_short_style_methods() {
            let ok_response = Response::ok();
            assert_eq!(ok_response.status, StatusCode::OK);

            let created_response = Response::created();
            assert_eq!(created_response.status, StatusCode::CREATED);

            let no_content_response = Response::no_content();
            assert_eq!(no_content_response.status, StatusCode::NO_CONTENT);

            let bad_request_response = Response::bad_request();
            assert_eq!(bad_request_response.status, StatusCode::BAD_REQUEST);

            let not_found_response = Response::not_found();
            assert_eq!(not_found_response.status, StatusCode::NOT_FOUND);
        }
    }

    mod response_send {
        use super::*;

        #[test]
        fn test_send_standard_response() -> Result<(), Box<dyn std::error::Error>> {
            let response = Response::new()
                .content_type("text/plain")
                .body(b"Hello".to_vec());

            let mut mock_stream = Vec::new();

            let _ = response.send(&mut mock_stream)?;

            let result_string = String::from_utf8(mock_stream)?;

            // Все заголовки ниже теперь написаны строго на английской раскладке
            let expected = "HTTP/1.1 200 OK\r\n\
                    content-length: 5\r\n\
                    content-type: text/plain\r\n\
                    connection: close\r\n\r\n\
                    Hello";

            assert_eq!(expected, result_string);
            Ok(())
        }


        #[test]
        fn test_send_no_content_response() -> Result<(), Box<dyn std::error::Error>> {
            let response = Response::no_content();
            let mut mock_stream = Vec::new();

            let _ = response.send(&mut mock_stream)?;
            let result_string = String::from_utf8(mock_stream)?;

            let expected = "HTTP/1.1 204 No Content\r\n\
                            connection: close\r\n\r\n";
            assert_eq!(expected, result_string);
            Ok(())
        }
    }
}
