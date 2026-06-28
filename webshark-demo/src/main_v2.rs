use webshark::routing::scope::Scope;
use webshark::routing::socket::Socket;
use webshark::{Router, Server};
use webshark_macros::build_router;

#[tokio::main]
async fn main() {
    let mut router = Router::new();

    // TODO было бы круто вынести конфиг отдельно функцией with_config(config: Config)

    // router.add_route(Route::new(Method::GET, "/", home_handler));
    // router.add_route(Route::new(Method::POST, "/start", post_handler));

    // 1. Собираем роуты из контроллера автоматически!
    let user_routes = build_router![crate::controller::user_controller::user];

    let mut users = Scope::new("/users");

    // users.add_enpoints(); // По идеи добавить метод позволяющий сразу регистрировать список роутов и сокетов

    // Вроде он должен внутри сам будет сделать какой-то аналог вот этого и записать все данные
    // for route in user_routes {
    //     users = users.add_route(route);
    // }

    // // Добавляем вебсокет (пока вручную или через get_sockets()) // Это уйдёт вообще
    // users = users.add_websocket(Socket::new("/tests", test_websocket));

    let api_v1 = Scope::new("/api/v1").nest(users);
    router.add_scope(api_v1);

    let server = Server::new(router)
        .http1(true)
        .http2(true)
        .http3(true);

    server.start_server().await.expect("Failed to start server");
}

async fn test() {
    let mut users = Scope::new("/users");
    user_controller::configure(&mut users);

    let mut users = Scope::new("/users");
    users.add_enpoints(user_endpoints);

    let api_v1 = Scope::new("/api/v1").nest(users);
    router.add_scope(api_v1);

    let server = Server::new(router)
        .http1(true)
        .http2(true)
        .http3(true);

    server.start_server().await.expect("Failed to start server");
}
