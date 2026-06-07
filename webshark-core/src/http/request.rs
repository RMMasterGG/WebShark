//! Модуль, отвечающий за создание входящих запросов (Request).
//!
//! Позволяет удобно их распарсить и преобразовать в необходимые типы.
//! Модуль содержит в себе перечисление типов методов отправки.

use crate::auth::authentication::{Authentication, Authorization};
use bytes::Bytes;
use http::{HeaderMap, HeaderName, HeaderValue, Method, Request as HttpRequest, Uri};
use std::fmt::Debug;
use std::str::FromStr;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

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
    pub async fn parse<T>(stream: &mut T) -> Result<Self, Box<dyn std::error::Error>>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        let mut buffer = Vec::with_capacity(1024);
        let mut chunk = [0u8; 512];
        let mut headers_end_pos = None;

        loop {
            let read_size = stream.read(&mut chunk).await?;
            if read_size == 0 {
                if buffer.is_empty() {
                    return Err(Box::from(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "Empty request",
                    )));
                }
                break;
            }

            buffer.extend_from_slice(&chunk[..read_size]);

            if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                headers_end_pos = Some(pos);
                break;
            }

            if buffer.len() > 8192 {
                return Err(Box::from("HTTP headers too large"));
            }
        }

        let pos = headers_end_pos.ok_or("Malformed HTTP request")?;
        let headers_bytes = &buffer[..pos];

        let mut body_bytes = buffer[pos + 4..].to_vec();

        let headers_str = String::from_utf8_lossy(headers_bytes);
        let mut header_lines = headers_str.split("\r\n");
        let first_line = header_lines.next().unwrap_or("");
        let mut first_line_words = first_line.split_whitespace();

        let mut headers = HeaderMap::new();
        for line in header_lines {
            let line = line.trim();
            if line.is_empty() { continue; }
            if let Some((key, value)) = line.split_once(":") {
                let clean_key = key.trim().to_lowercase();
                let clean_value = value.trim_matches(|c: char| c.is_whitespace() || c == '\r' || c == '\n');

                if let (Ok(name), Ok(val)) = (
                    HeaderName::from_str(&clean_key),
                    HeaderValue::from_str(clean_value),
                ) {
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
            && body_bytes.len() < content_length
        {
            let current_body_len = body_bytes.len();
            let mut body_buffer = vec![0; content_length - current_body_len];
            stream.read_exact(&mut body_buffer).await?;
            body_bytes.extend_from_slice(&body_buffer); // Склеиваем байты, а не строки!
        }


        let method_str = first_line_words.next().ok_or("Missing method")?;
        let uri_str = first_line_words.next().unwrap_or("/");

        let mut http_req_builder = HttpRequest::builder()
            .method(Method::from_str(method_str)?)
            .uri(uri_str);

        if let Some(h) = http_req_builder.headers_mut() {
            *h = headers;
        }

        let inner = http_req_builder.body(Bytes::from(body_bytes))?;

        let authentication = Authentication::new(authorization, None, false);

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
    pub fn get_header<T>(&self, key: T) -> Option<&str>
    where
        T: TryInto<HeaderName>,
        T::Error: Debug,
    {
        key.try_into()
            .ok()
            .and_then(|header_name| self.inner.headers().get(header_name))
            .and_then(|header_value| header_value.to_str().ok())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     mod request_parse {
//         use http::header::{CONTENT_LENGTH, HOST};
//         use super::*;
//
//         #[test]
//         fn test_parse_request() -> Result<(), Box<dyn std::error::Error>> {
//             let mut raw_request =
//                 "GET /index.html HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\n\r\n"
//                     .as_bytes();
//
//             let request = Request::parse(&mut raw_request)?;
//
//             assert_eq!(Method::GET, request.method());
//             assert_eq!("/index.html", request.uri().path());
//             assert_eq!(Some("localhost"), request.get_header(HOST));
//             assert_eq!(Some("0"), request.get_header(CONTENT_LENGTH));
//             Ok(())
//         }
//
//         #[test]
//         fn test_parse_request_post_with_body() -> Result<(), Box<dyn std::error::Error>> {
//             let body = r#"{"id":123}"#;
//
//             let raw_string = format!(
//                 "POST /api/login HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
//                 body.len(),
//                 body
//             );
//
//             let mut raw_request = raw_string.as_bytes();
//
//             let request = Request::parse(&mut raw_request)?;
//
//             let string_body_len = body.len().to_string();
//
//             assert_eq!(Method::POST, request.method());
//             assert_eq!("/api/login", request.uri().path());
//             assert_eq!(
//                 Some(string_body_len.as_str()),
//                 request.get_header(CONTENT_LENGTH)
//             );
//             assert_eq!(body, request.body());
//             Ok(())
//         }
//     }
//
//     mod request_get_header {
//         use http::header::{AUTHORIZATION, HOST};
//         use super::*;
//
//         #[test]
//         fn test_header_exists() -> Result<(), Box<dyn std::error::Error>> {
//             let mut raw_request =
//                 "GET /index.html HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\n\r\n"
//                     .as_bytes();
//
//             let request = Request::parse(&mut raw_request)?;
//
//             assert_eq!(Some("localhost"), request.get_header(HOST));
//             Ok(())
//         }
//
//         #[test]
//         fn test_header_missing() -> Result<(), Box<dyn std::error::Error>> {
//             let mut raw_request = "GET /index.html HTTP/1.1\r\nHost: localhost\r\n\r\n".as_bytes();
//             let request = Request::parse(&mut raw_request)?;
//
//             assert_eq!(None, request.get_header(AUTHORIZATION));
//             Ok(())
//         }
//     }
// }
