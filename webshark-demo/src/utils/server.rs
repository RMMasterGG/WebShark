// use std::io::{Read, Write};
// use tokio::net::{TcpListener};
// use std::sync::Arc;
// use base64::Engine;
// use base64::prelude::BASE64_STANDARD;
// use bytes::Bytes;
// use http_util::header::{ACCESS_CONTROL_ALLOW_ORIGIN, CONNECTION, ORIGIN, SEC_WEBSOCKET_KEY, UPGRADE};
// use sha1::{Digest, Sha1};
// use tracing::{error, warn};
// use webshark::{Request, Response, Router};
// use webshark::routing::router::RouteType;
//
// pub struct Server {
//     router: Router,
// }
//
// impl Server {
//     pub fn new(router: Router) -> Self {
//         Self { router }
//     }
//
//     pub async fn run(&self) -> std::io::Result<()> {
//         let tcp_listener = TcpListener::bind("127.0.0.1:8080").await?;
//
//         let router = Arc::new(self.router.clone());
//
//         loop {
//             let (stream, _) = tcp_listener.accept().await?;
//             let router_clone = router.clone();
//
//             if let Ok(sync_stream) = stream.into_std()
//                 && sync_stream.set_nonblocking(false).is_ok()
//             {
//                 handle_client(sync_stream, router_clone);
//             }
//         }
//     }
// }
//
// fn handle_http() {
//
// }
//
// fn handle_ws(request: Request<Bytes>, mut stream: impl Read + Write) {
//     if let Some(accept_key) = generate_accept_key(&request) {
//         let handshake_response = format!(
//             "HTTP/1.1 101 Switching Protocols\r\n\
//              Upgrade: websocket\r\n\
//              Connection: Upgrade\r\n\
//              Sec-WebSocket-Accept: {}\r\n\r\n",
//             accept_key
//         );
//
//         if let Err(e) = stream.write_all(handshake_response.as_bytes()) {
//             error!("Ошибка записи рукопожатия в сокет: {}", e);
//             return;
//         }
//         let _ = stream.flush();
//         println!("Успешное WebSocket рукопожатие!");
//
//     loop {
//         let mut header = [0u8; 2];
//         if stream.read_exact(&mut header).is_err() { break; }
//
//         let opcode = header[0] & 0x0F;
//         if opcode == 8 { break; }
//
//         let payload_len = (header[1] & 0x7F) as usize;
//
//         if payload_len == 126 || payload_len == 127 {
//             println!("Сообщение слишком длинное для этого простого примера");
//             break;
//         }
//
//         let mut mask = [0u8; 4];
//         stream.read_exact(&mut mask).unwrap();
//
//         let mut payload = vec![0u8; payload_len];
//         stream.read_exact(&mut payload).unwrap();
//
//         for i in 0..payload_len {
//             payload[i] ^= mask[i % 4];
//         }
//
//         let text = String::from_utf8_lossy(&payload);
//         println!("Браузер прислал: {}", text);
//
//         let mut response_frame = vec![0x81, payload_len as u8];
//         response_frame.extend_from_slice(&payload);
//
//         if stream.write_all(&response_frame).is_err() { break; }
//         stream.flush().unwrap();
//     }
//     println!("Соединение закрыто.");
//
//     } else {
//         warn!("Не удалось сгенерировать Sec-WebSocket-Accept: неверный или отсутствующий ключ");
//
//         let client_origin = request.get_header(ORIGIN).unwrap_or("");
//         let mut response = Response::bad_request();
//
//         if !client_origin.is_empty() {
//             response = response.header(ACCESS_CONTROL_ALLOW_ORIGIN, client_origin);
//         }
//         let _ = response.send(&mut stream);
//         let _ = stream.flush();
//     }
// }
//
// fn generate_accept_key(p0: &Request<Bytes>) -> Option<String> {
//     Some(String::new())
// }
//
// fn handle_client(mut stream: impl Read + Write, routers: Arc<Router>) {
//
//     let request = match Request::parse(&mut stream) {
//         Ok(req) => req,
//         Err(e) => return error!("Error parsing request: {}", e)
//     };
//
//     let request_method = request.method();
//
//     let request_path = request.uri().path();
//
//     if let Some(compiled_route) = routers.get_route(&request_method, RouteType::Http, &request_path) {
//
//
//
//         let is_connection = request
//             .get_header(CONNECTION)
//             .map(|value| value.split(",").any(|value| value.trim().eq_ignore_ascii_case("upgrade")))
//             .unwrap_or(false);
//
//         let is_upgrade = request
//             .get_header(UPGRADE)
//             .map(|value| value.eq_ignore_ascii_case("websocket"))
//             .unwrap_or(false);
//
//         let is_websocket = is_connection && is_upgrade;
//
//         if compiled_route.is_websocket() && is_websocket {
//             handle_ws(request, stream);
//         } else {
//             handle_http()
//         }
//     } else {
//         let client_origin = request.get_header(ORIGIN).unwrap_or("");
//         let mut response = Response::not_found();
//         if !client_origin.is_empty() {
//             response = response.header(ACCESS_CONTROL_ALLOW_ORIGIN, client_origin);
//         }
//         let _ = response.send(&mut stream);
//     }
// }
