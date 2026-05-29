//! # 🦈 Webshark
//!
//! `webshark` — это быстрый, легковесный и безопасный HTTP-сервер, написанный на чистом Rust.
//! Он предоставляет модульный движок для парсинга сетевых пакетов, гибкую регистронезависимую
//! маршрутизацию и абстракции над HTTP-протоколом.
//!
//! ## 🛠 Основные возможности
//!
//! * **Быстрый парсинг:** Эффективная работа со стримами через типаж [`std::io::Read`].
//! * **Безопасность:** Нулевое количество скрытых паник (`panic!`), устойчивость к некорректным данным.
//! * **Удобный роутинг:** Быстрое сопоставление путей с автоматическим затиранием типов обработчиков.
//! * **Регистронезависимость:** Заголовки HTTP обрабатываются корректно независимо от регистра.
//!
//! ## 🚀 Быстрый старт
//!
//! Ниже представлен пример создания простейшего эндпоинта и запуска сервера:
//!
//! ```no_run
//! use webshark::{start_server, Router, Route, Method, Request, Response};
//!
//! #[tokio::main]
//! async fn main() {
//!     // 1. Создаем маршрутизатор
//!     let mut router = Router::new();
//!
//!     // 2. Регистрируем обработчик для главной страницы
//!     router.add_route(Route::new(Method::GET, "/", home_handler));
//!
//!     // 3. Запускаем сервер (он подтянет конфигурацию и начнет слушать порт)
//!     start_server(router).await.expect("Не удалось запустить сервер");
//! }
//!
//! // Простой обработчик возвращающий статус 200 OK
//! fn home_handler(_request: Request) -> Response {
//!     Response::ok()
//! }
//! ```
//!
//! ## Проект планирует переезд на Hyper.

pub mod dto;
pub mod http;
pub mod utils;
pub mod routing;
pub mod auth;
pub mod server;

pub use http::request::Request;
pub use http::response::Response;
pub use routing::route::Route;
pub use routing::router::Router;
pub use server::Server;
