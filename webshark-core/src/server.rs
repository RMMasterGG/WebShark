//! Модуль сетевого движка сервера.
//!
//! Отвечает за инициализацию TCP-слушателя, прием входящих соединений
//! в бесконечном цикле и их маршрутизацию через [`Router`].

use crate::routing::router::{Handler, RouteType, Router, WSHandler};
use crate::routing::socket_context::WebSocketContext;
use crate::utils::config_system::{APP_CONFIG, Config};
use crate::utils::console_util::SHARK_BANNER;
use crate::utils::websocket::generate_accept_key;
use crate::{Request, Response};
use bytes::Bytes;
use http::Method;
use http::header::{
    ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
    ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, CONNECTION, ORIGIN, REFERER,
    UPGRADE,
};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, DuplexStream, split};
use tokio::net::TcpListener;
use tracing::{error, info, warn};
use tracing_subscriber::fmt;

pub struct Server {
    router: Router,
}

impl Server {
    /// Создает новый экземпляр сервера с привязанным маршрутизатором.
    pub fn new(router: Router) -> Self {
        Self { router }
    }

    /// Запускает HTTP-сервер на указанном сетевом адресе.
    ///
    /// Принимает адрес (например, `"127.0.0.1:8080"`) и настроенный [`Router`].
    /// Оборачивает роутер в [`std::sync::Arc`] для безопасного совместного использования.
    ///
    /// # Errors
    ///
    /// Возвращает [`std::io::Error`], если не удалось забиндить указанный порт.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use webshark::start_server;
    /// use webshark::routing::request::{Method, Request};
    /// use webshark::routing::response::Response;
    /// use webshark::routing::route::Route;
    /// use webshark::routing::router::Router;
    ///
    ///#[tokio::main]
    /// async fn main() {
    ///     // Создаём основной обработчик маршрутов
    ///     let mut router = Router::new();
    ///
    ///     // Добавляем в обработчик маршрутов новый маршрут
    ///     router.add_route(Route::new(Method::GET, "/", test_handler));
    ///
    ///     // Запускаем сервер
    ///     start_server(router).await.unwrap();
    /// }
    ///
    /// fn test_handler(request: Request) -> Response {
    ///     let response = Response::ok();
    ///     println!("{:#}", response);
    ///     response
    /// }
    /// ```
    pub async fn start_server(&self) -> std::io::Result<()> {
        Config::init_config();
        let config = APP_CONFIG.get().unwrap();

        let format = fmt::format()
            .with_thread_ids(true)
            .with_thread_names(true)
            .compact();

        // Logging subscribe
        tracing_subscriber::fmt().event_format(format).init();

        let tcp_listener = TcpListener::bind(config.server().server_and_port()).await?;

        let router = Arc::new(self.router.clone());

        println!("{}", SHARK_BANNER);

        println!("HTTP-сервер успешно запущен и слушает подключения...");

        loop {
            let (stream, _) = tcp_listener.accept().await?;
            let router_clone = router.clone();

            tokio::spawn(async move {
                handle_client(stream, router_clone).await;
            });
        }
    }
}

/// Проверяет входящий HTTP-запрос на соответствие политикам безопасности CORS и CSRF.
///
/// # Назначение
/// Метод выполняет многоуровневую фильтрацию сетевого трафика:
/// 1. Разрешает прямые не-браузерные запросы (например, от `curl`, Postman или ручного ввода URL).
/// 2. Защищает от CSRF-атак (Cross-Site Request Forgery), валидируя заголовок `Referer` при отсутствии `Origin`.
/// 3. Проверяет домен отправителя (`Origin`) на соответствие белому списку из конфигурации приложения.
/// 4. Обрабатывает предварительные CORS-запросы (Preflight/`OPTIONS`), проверяя запрашиваемые методы и кастомные заголовки.
///
/// # Возвращаемое значение
/// * `true` — запрос полностью валиден и безопасен, его можно передавать дальше по цепочке фильтров к роутеру.
/// * `false` — обнаружено нарушение политик безопасности, запрос должен быть немедленно заблокирован с ответом `403 Forbidden` (или `400 Bad Request`).
///
/// # Литература и стандарты
/// * [Семейство спецификаций W3C CORS](https://w3.org) [1]
/// * [RFC 9110: HTTP Semantics (Раздел 12: Заголовки авторизации и происхождения)](https://ietf.org) [2]
fn validate_cors_and_origin(request: &Request<Bytes>) -> bool {
    // TODO: [Производительность] Избежать распаковки глобального конфига при каждом вызове.
    // Передавать `&Config` в качестве аргумента функции или перенести метод внутрь структуры `FilterChain`.
    let config = match APP_CONFIG.get() {
        Some(c) => c,
        None => return true,
    };

    let origin = request.get_header(ORIGIN).unwrap_or("");
    let referer = request.get_header(REFERER).unwrap_or("");

    // TODO: [Безопасность / Host Injection] Добавить обязательную сверку заголовка `Host` из запроса
    // с реальным `config.server().server_and_port()`. Если они не совпадают — блокировать запрос,
    // чтобы предотвратить подмену хоста (Host Header Attack).

    // 1. Прямой запрос без заголовков происхождения (ввод URL руками в браузере или curl) — разрешаем
    if origin.is_empty() && referer.is_empty() {
        return true;
    }

    // 2. Защита от CSRF по Referer (если запрос инициирован переходом по ссылке со стороннего ресурса)
    if origin.is_empty() && !referer.is_empty() {
        let host = config.server().server_and_port();

        // TODO: [Продакшен Оптимизация] Избежать аллокаций динамических строк `format!` через кучу при каждом запросе.
        // Заранее сгенерировать строки "http://..." и "https://..." на этапе `init_config` и сохранить их в конфиг,
        // либо проверять вхождение через побайтовое сравнение срезов.
        let is_valid_http = referer.contains(format!("http://{}", host).as_str());
        let is_valid_https = referer.contains(format!("https://{}", host).as_str());

        // TODO: [Безопасность / Строгий парсинг] Метод `.contains` уязвим, если атакующий создаст домен вида `localhost:://evil.com`.
        // Реферер должен парситься как полноценный URL (выделять схему и хост) и сравниваться на точное равенство с `host`.
        if !is_valid_http && !is_valid_https {
            return false;
        }

        return true;
    }

    // TODO: [Продакшен / Reverse Proxy] Если сервер стоит за Nginx, Cloudflare или балансировщиком нагрузки,
    // заголовок `Origin` может содержать IP прокси. Нужно добавить чтение заголовков `X-Forwarded-Host` или `Forwarded`.

    // 3. Проверка домена (CORS Origin)
    if !config.cors().is_allowed_origin(origin) {
        return false;
    }

    if config.cors().allowed_origins().contains(&"*".to_string()) {
        return true;
    }

    // 4. Проверки для Preflight-запросов (OPTIONS)
    if request.method() == Method::OPTIONS {
        let request_method = request
            .get_header(ACCESS_CONTROL_REQUEST_METHOD)
            .unwrap_or("");

        if !request_method.is_empty() {
            if let Ok(req_method) = Method::from_bytes(request_method.as_bytes()) {
                if !config.cors().is_allowed_method(req_method) {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(req_headers_str) = request.get_header(ACCESS_CONTROL_REQUEST_HEADERS)
            && !req_headers_str.is_empty()
        {
            let all_headers_allowed = req_headers_str
                .split(",")
                .all(|h| config.cors().is_allowed_header(h));

            if !all_headers_allowed {
                return false;
            }
        }

        // TODO: [CORS Спецификация / Куки] Если в `config.toml` включена поддержка сессий/кук (`Allow-Credentials`),
        // то по стандарту браузеров `allowed_origins` НЕ МОЖЕТ содержать маску `*`.
        // Добавить проверку: если `origin == "*"` и `credentials == true`, возвращать `false` (защита от падения фронтенда).
    }

    true
}

/// Обрабатывает подключение конкретного TCP-клиента.
///
/// Метод парсит входящие байты в [`Request`], ищет подходящий эндпоинт
/// в [`Router`], вызывает его и отправляет полученный [`Response`] обратно в сеть.
async fn handle_client<T>(mut stream: T, routers: Arc<Router>)
where
    T: AsyncRead + AsyncWrite + Unpin + Send + 'static + Sync,
{
    let request = match Request::parse(&mut stream).await {
        Ok(req) => req,
        Err(e) => return error!("Error parsing request: {}", e),
    };

    if !validate_cors_and_origin(&request) {
        let response = Response::forbidden();
        let _ = response.send(&mut stream).await;
        let _ = stream.flush().await;
        return;
    }

    let client_origin = request.get_header(ORIGIN).map(|s| s.to_string());

    if request.method() == Method::OPTIONS {
        let config = APP_CONFIG.get().unwrap();

        let origin_str = client_origin.as_deref().unwrap_or("*");

        let allowed_methods_str = config
            .cors()
            .allowed_methods()
            .iter()
            .map(|method| method.as_str())
            .collect::<Vec<&str>>()
            .join(", ");

        // Отправляем пустой ответ 204 с разрешениями
        let cors_response = Response::no_content()
            .header(ACCESS_CONTROL_ALLOW_ORIGIN, origin_str)
            .header(ACCESS_CONTROL_ALLOW_METHODS, allowed_methods_str)
            .header(
                ACCESS_CONTROL_ALLOW_HEADERS,
                config.cors().allowed_headers().join(", "),
            );

        if let Err(e) = cors_response.send(&mut stream).await {
            println!("Ошибка отправки предзапроса OPTIONS: {}", e);
        }
        return;
    }

    let req_method = request.method().clone();
    let req_path = request.uri().path().to_string();

    let is_connection = request
        .get_header(CONNECTION)
        .map(|value| {
            value
                .split(",")
                .any(|v| v.trim().eq_ignore_ascii_case("upgrade"))
        })
        .unwrap_or(false);

    let is_upgrade = request
        .get_header(UPGRADE)
        .map(|value| value.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false);

    let is_websocket = is_connection && is_upgrade;

    let router_type = if is_websocket {
        RouteType::WebSocket
    } else {
        RouteType::Http
    };

    if let Some(compiled_route) = routers.get_route(&req_method, router_type, &req_path) {
        let client_origin = request.get_header(ORIGIN).map(|s| s.to_string());

        let mut chain = compiled_route.filter_chain().clone();

        let handler_arc = compiled_route.handler().clone();

        let request_clone = request.clone();

        let response_result = chain.execute::<_, ()>(request, move |req| {
            let handler_arc_clone = handler_arc.clone();

            let future: crate::utils::other::BoxFuture<'static, Result<Response<Bytes>, &'static str>> =
                Box::pin(async move {
                    match &handler_arc_clone {
                        Handler::Http(_) => {
                            Ok(handler_arc_clone.invoke_http(req).await)
                        }
                        Handler::WebSocket(_) => {
                            Ok(Response::websocket_upgraded())
                        }
                        Handler::WebTransport(_) => {
                            Ok(Response::default())
                        }
                    }
                });

            future
        }).await;


        let mut response = response_result.unwrap_or_else(|err_msg| {
            println!("[ERROR] Ошибка выполнения цепочки фильтров: {}", err_msg);
            Response::internal_error_body(err_msg)
        });

        if response.is_websocket_upgraded() {
            if let Handler::WebSocket(ws_handler) = &*compiled_route.handler() {
                if let Some(accept_key) = generate_accept_key(&request_clone) {
                    let handshake_raw = format!(
                        "HTTP/1.1 101 Switching Protocols\r\n\
                 Upgrade: websocket\r\n\
                 Connection: Upgrade\r\n\
                 Sec-WebSocket-Accept: {}\r\n\r\n",
                        accept_key
                    );

                    println!("[SHARK] Отправляем хэндшейк с ключом: {}", accept_key);

                    stream
                        .write_all(handshake_raw.as_bytes())
                        .await
                        .unwrap();
                    stream.flush().await.unwrap();

                    websocket_handler(stream, ws_handler, request_clone);
                    return;
                } else {
                    response = Response::bad_request();
                }
            }
        }

        if let Some(origin_str) = client_origin
            && !origin_str.is_empty()
        {
            response = response.header(ACCESS_CONTROL_ALLOW_ORIGIN, origin_str);
        }

        if let Err(e) = response.send(&mut stream).await {
            info!("Error sending response: {}", e);
        }
    } else {
        let client_origin = request.get_header(ORIGIN).unwrap_or("");
        let mut response = Response::not_found();
        if !client_origin.is_empty() {
            response = response.header(ACCESS_CONTROL_ALLOW_ORIGIN, client_origin);
        }
        let _ = response.send(&mut stream).await;
        stream.flush().await.unwrap();
    }
}

fn websocket_handler<T>(stream: T, ws_handler: &WSHandler, request: Request<Bytes>)
where
    T: AsyncRead + AsyncWrite + Unpin + Sync + Send + 'static,
{
    let ctx = WebSocketContext::new(stream);

    let future = ws_handler.invoke(request, ctx);

    tokio::spawn(future);
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::routing::route::Route;
//     use std::io::Cursor;
//
//     fn init_test_config() {
//         // Гарантируем, что конфиг инициализирован для тестов
//         let _ = Config::init_config();
//     }
//
//     fn mock_handler(_req: Request<Bytes>) -> Response<Bytes> {
//         Response::ok_body("shark_data")
//     }
//
//     #[test]
//     fn test_handle_client_success() {
//         init_test_config();
//         let mut router = Router::new();
//         router.add_route(Route::new(Method::GET, "/api", mock_handler));
//         let router_arc = Arc::new(router);
//
//         let input = "GET /api HTTP/1.1\r\nHost: localhost\r\n\r\n";
//         let mut mock_stream = Cursor::new(input.as_bytes().to_vec());
//
//         handle_client(&mut mock_stream, router_arc);
//
//         let response = String::from_utf8_lossy(mock_stream.get_ref());
//         assert!(response.contains("HTTP/1.1 200 OK"));
//         assert!(response.contains("shark_data"));
//     }
//
//     #[test]
//     fn test_handle_client_404() {
//         init_test_config();
//         let router = Arc::new(Router::new());
//
//         let input = "GET /missing HTTP/1.1\r\n\r\n";
//         let mut mock_stream = Cursor::new(input.as_bytes().to_vec());
//
//         handle_client(&mut mock_stream, router);
//
//         let response = String::from_utf8_lossy(mock_stream.get_ref());
//         assert!(response.contains("HTTP/1.1 404 Not Found"));
//     }
//
//     #[test]
//     fn test_handle_options_preflight() {
//         init_test_config();
//         let router = Arc::new(Router::new());
//
//         // Симулируем браузерный предзапрос
//         let input = "OPTIONS /api HTTP/1.1\r\nOrigin: http://localhost:3000\r\nAccess-Control-Request-Method: GET\r\n\r\n";
//         let mut mock_stream = Cursor::new(input.as_bytes().to_vec());
//
//         handle_client(&mut mock_stream, router);
//
//         let response = String::from_utf8_lossy(mock_stream.get_ref());
//         assert!(response.contains("HTTP/1.1 204 No Content"));
//         let response_lower = response.to_lowercase();
//         assert!(response_lower.contains("access-control-allow-origin: http://localhost:3000"));
//     }
// }
