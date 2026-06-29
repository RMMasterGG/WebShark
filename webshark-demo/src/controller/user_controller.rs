use crate::helpers::html_file::send_http_file;
use webshark::bytes::Bytes;
use webshark::cookie::Cookie;
use webshark::routing::socket_context::WebSocketContext;
use webshark::tokio_tungstenite::tungstenite::Message;
use webshark::tracing::{error, info};
use webshark::{Request, Response};
use webshark::macros::{controller, get, post, websocket};
use crate::filters::auth_filter::AuthFilter;

pub struct UserController;

#[controller(path = "/users", filters = [AuthFilter])]
impl UserController {

    #[get(path = "/test")]
    pub async fn home_handler() -> Response<Bytes> {
        let file = match send_http_file("home.html".to_string()) {
            Ok(file) => file,
            Err(e) => {
                println!("{}", e);
                return Response::internal_error();
            }
        };

        Response::ok_body(file)
    }

    #[post("/cookie")]
    pub async fn cookie_handler(req: Request<Bytes>) -> Response<Bytes> {
        let cookie = Cookie::new("test", "test");
        Response::ok_body(req.body_bytes()).set_cookie(cookie)
    }

    #[post(path = "/delete-account", filters = [AuthFilter, AdminOnlyFilter])]
    pub async fn post_handler(req: Request<Bytes>) -> Response<Bytes> {
        Response::ok_body(req.body_bytes())
    }

    #[websocket("/rofl")]
    pub async fn test_websocket(req: Request<Bytes>, mut ctx: WebSocketContext) {
        info!(
            "Акула поймала новое WebSocket подключение на путь: {}",
            req.uri().path()
        );

        // 1. Отправляем приветственный фрейм сразу при подключении
        if let Err(e) = ctx.send("Добро пожаловать в webshark эхо-сервер!").await
        {
            error!("Не удалось отправить приветствие: {}", e);
            return;
        }

        // 2. Бесконечный асинхронный цикл общения
        loop {
            match ctx.recv().await {
                Ok(Message::Text(client_text)) => {
                    // Оптимизация: передаем client_text по ссылке, format! сам подставит её без клонирования
                    let echo_response = format!("Эхо от Акулы: {client_text}");

                    if let Err(e) = ctx.send(echo_response).await {
                        error!("Ошибка при отправке эхо-ответа: {}", e);
                        break;
                    }
                }
                Ok(Message::Binary(_)) => {
                    info!("Получен бинарный фрейм, для эхо-сервера игнорируем.");
                }
                Err(e) => {
                    info!("WebSocket соединение закрыто: {}", e);
                    break;
                }
                _ => {}
            }
        }
    }
}
