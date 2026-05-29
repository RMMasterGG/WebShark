//! Модуль, отвечающий за создание входящих запросов (Request).
//!
//! Позволяет удобно их распарсить и преобразовать в необходимые типы.
//! Модуль содержит в себе перечисление типов методов отправки.

use crate::auth::authentication::{Authentication, Authorization};
use bytes::Bytes;
use http::{HeaderMap, HeaderName, HeaderValue, Method, Uri, Request as HttpRequest};
use std::io::Read;
use std::str::FromStr;

/// Структура запроса.
///
/// Имеет поля: метод [`Method`], часть пути, список всех заголовков и тело запроса.
#[derive(Debug, Clone, Default)]
pub struct Request<B> {
    inner: HttpRequest<B>,
    authentication: Authentication,
}

/// Реализация, позволяющая преобразовать данные из потока в структуру запроса.
///
/// # Examples
///
/// ```no_run
/// use webshark::routing::request::Request;
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
        let (headers_part, body_part) = raw_data.split_once("\r\n\r\n").unwrap_or((&raw_data, ""));
        let mut body_string = body_part.to_string();

        let mut header_lines = headers_part.lines();
        let first_line = header_lines.next().unwrap_or("");
        let mut first_line_words = first_line.split_whitespace();

        let mut headers = HeaderMap::new();
        for line in header_lines {
            if let Some((key, value)) = line.split_once(":") {
                if let (Ok(name), Ok(val)) = (HeaderName::from_str(key.trim()), HeaderValue::from_str(value.trim())) {
                    headers.insert(name, val);
                }
            }
        }

        let authorization = if let Some(auth_value) = headers.remove(http::header::AUTHORIZATION) {
            Authorization::parse(auth_value.to_str().unwrap_or(""))
        } else {
            Authorization::parse("")
        };

        if let Some(cl_val) = headers.get(http::header::CONTENT_LENGTH)
            && let Ok(cl_str) = cl_val.to_str()
            && let Ok(content_length) = cl_str.parse::<usize>()
            && body_string.len() < content_length
        {
            let mut body_buffer = vec![0; content_length - body_string.len()];
            stream.read_exact(&mut body_buffer)?;
            body_string.push_str(&String::from_utf8_lossy(&body_buffer));
        }

        let method_str = first_line_words.next().ok_or("Missing method")?;
        let uri_str = first_line_words.next().unwrap_or("/");

        let mut http_req_builder = HttpRequest::builder()
            .method(Method::from_str(method_str)?)
            .uri(uri_str);

        if let Some(h) = http_req_builder.headers_mut() {
            *h = headers;
        }

        let inner = http_req_builder.body(Bytes::from(body_string))?;

        let authentication = Authentication::new(
            authorization,
            None,
            false
        );

        Ok(Self {
            inner,
            authentication,
        })
    }

    pub fn method(&self) -> &Method {
        self.inner.method()
    }

    pub fn method_copy(&self) -> Method {
        self.inner.method().clone()
    }

    pub fn uri(&self) -> &Uri {
        &self.inner.uri()
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.inner.headers()
    }

    pub fn body(&self) -> &Bytes {
        &self.inner.body()
    }

    pub fn body_bytes(&self) -> Bytes {
        self.inner.body().clone()
    }

    pub fn authentication(&self) -> &Authentication {
        &self.authentication
    }

    pub fn authorization(&self) -> &Authorization {
        &self.authentication.credentials()
    }

    /// Метод позволяет выбрать определённые данные из заголовка.
    pub fn get_header(&self, key: &'static str) -> Option<&str> {
        if let Ok(header_name) = HeaderName::from_bytes(key.as_bytes()) {
            self.inner.headers()
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
