//! Модуль, отвечающий за создание входящих запросов (Request).
//!
//! Позволяет удобно их распарсить и преобразовать в необходимые типы.
//! Модуль содержит в себе перечисление типов методов отправки.

use crate::utils::authentication::Authorization;
use bytes::Bytes;
use http::{HeaderMap, HeaderName, HeaderValue, Method, Uri};
use std::io::Read;
use std::str::FromStr;

/// Структура запроса.
///
/// Имеет поля: метод [`Method`], часть пути, список всех заголовков и тело запроса.
#[derive(Debug, Clone, Default)]
pub struct Request<B> {
    method: Method,
    uri: Uri,
    authorization: Authorization,
    headers: HeaderMap,
    body: B,
}

/// Реализация, позволяющая преобразовать данные из потока в структуру запроса.
///
/// # Examples
///
/// ```no_run
/// use webshark::utils::request::Request;
/// use std::net::TcpStream;
///
/// # fn main() -> Result<(), std::io::Error> {
///     let mut stream = TcpStream::connect("127.0.0.1:7878")?;
///     let request = Request::parse(&mut  stream)?;
///     Ok(())
/// # }
///
/// ```
impl Request<Bytes> {
    pub fn parse(stream: &mut impl Read) -> Result<Self, Box<dyn std::error::Error>> {
        let mut buffer = [0; 8192];
        let read_size = stream.read(&mut buffer)?;

        if read_size == 0 {
            return Err(Box::from(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Empty request",
            )));
        }

        let raw_data = String::from_utf8_lossy(&buffer[..read_size]);
        let mut parts = raw_data.split("\r\n\r\n");

        let mut headers = HeaderMap::new();
        let headers_part = parts.next().unwrap_or("");
        let mut body_part: String = parts.next().unwrap_or("").into();

        let mut header_lines = headers_part.lines();
        let first_line = header_lines.next().unwrap_or("");
        let mut first_line_words = first_line.split_whitespace();

        for line in header_lines {
            let (key, value) = line.split_once(":").unwrap();
            if let (Ok(key), Ok(value)) = (HeaderName::from_str(key), HeaderValue::from_str(value.trim()))
            {
                headers.insert(key, value);
            }
        }

        let authorization = if let Some(auth_value) = headers.remove(http::header::AUTHORIZATION) {
            let auth_str = auth_value.to_str().unwrap_or("");
            Authorization::parse(auth_str)
        } else {
            Authorization::parse("")
        };

        if let Some(content_length_val) = headers.get(http::header::CONTENT_LENGTH)
            && let Ok(content_length_str) = content_length_val.to_str()
            && let Ok(content_length) = content_length_str.parse::<usize>()
            && body_part.len() < content_length
        {
            let missing_length = content_length - body_part.len();
            let mut body_buffer = vec![0; missing_length];

            if let Err(_err) = stream.read_exact(&mut body_buffer) {
                return Err(Box::from(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Failed to read full HTTP body fields from stream",
                )));
            }
            body_part.push_str(&String::from_utf8_lossy(&body_buffer));
        }

        let method_str = first_line_words
            .next()
            .ok_or("Bad Request: Missing HTTP method")?;

        let method = Method::from_bytes(method_str.as_bytes())
            .map_err(|_| "Bad Request: Invalid HTTP method")?;

        let uri_str = first_line_words.next().unwrap_or("/");
        let uri = Uri::from_str(uri_str)?;

        let body = Bytes::from(body_part);

        Ok(Self {
            method,
            uri,
            authorization,
            headers,
            body,
        })
    }

    pub fn method(&self) -> Method {
        self.method.clone()
    }

    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn body(&self) -> &Bytes {
        &self.body
    }

    pub fn body_bytes(&self) -> Bytes {
        self.body.clone()
    }

    pub fn authorization(&self) -> &Authorization {
        &self.authorization
    }

    /// Метод позволяет выбрать определённые данные из заголовка.
    pub fn get_header(&self, key: &'static str) -> Option<&str> {
        if let Ok(header_name) = HeaderName::from_bytes(key.as_bytes()) {
            self.headers
                .get(header_name)
                .and_then(|value| value.to_str().ok())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod request_parse {
        use super::*;

        #[test]
        fn test_parse_request() -> Result<(), Box<dyn std::error::Error>> {
            let mut raw_request =
                "GET /index.html HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\n\r\n"
                    .as_bytes();

            let request = Request::parse(&mut raw_request)?;

            assert_eq!(Method::GET, request.method());
            assert_eq!("/index.html", request.uri().path());
            assert_eq!(Some("localhost"), request.get_header("host"));
            assert_eq!(Some("0"), request.get_header("content-length"));
            Ok(())
        }

        #[test]
        fn test_parse_request_post_with_body() -> Result<(), Box<dyn std::error::Error>> {
            let body = r#"{"id":123}"#;

            let raw_string = format!(
                "POST /api/login HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );

            let mut raw_request = raw_string.as_bytes();

            let request = Request::parse(&mut raw_request)?;

            let string_body_len = body.len().to_string();

            assert_eq!(Method::POST, request.method());
            assert_eq!("/api/login", request.uri().path());
            assert_eq!(
                Some(string_body_len.as_str()),
                request.get_header("content-length")
            );
            assert_eq!(body, request.body());
            Ok(())
        }
    }

    mod request_get_header {
        use super::*;

        #[test]
        fn test_header_exists() -> Result<(), Box<dyn std::error::Error>> {
            let mut raw_request =
                "GET /index.html HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\n\r\n"
                    .as_bytes();

            let request = Request::parse(&mut raw_request)?;

            assert_eq!(Some("localhost"), request.get_header("host"));
            Ok(())
        }

        #[test]
        fn test_header_missing() -> Result<(), Box<dyn std::error::Error>> {
            let mut raw_request = "GET /index.html HTTP/1.1\r\nHost: localhost\r\n\r\n".as_bytes();
            let request = Request::parse(&mut raw_request)?;

            assert_eq!(None, request.get_header("authorization"));
            Ok(())
        }
    }
}
