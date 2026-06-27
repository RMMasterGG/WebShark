use std::mem;
use webshark::bytes::Bytes;
use webshark::macros::{controller, get, post};
use webshark::{Request, Response, Route};
use webshark::cookie::Cookie;
use webshark::http::Method;
use webshark::routing::scope::Scope;
use crate::helpers::html_file::send_http_file;

pub struct StoresController;

#[controller]
impl StoresController {
    pub fn configure(scope_ref: &mut Scope) {
        let new_route_home_handler = Route::new(Method::GET, "/test", Self::home_handler);
        let new_route_cookie_handler = Route::new(Method::GET, "/cookie", Self::cookie_handler);

        let mut scope = mem::take(scope_ref);

        scope = scope
            .add_route(new_route_home_handler)
            .add_route(new_route_cookie_handler);

        *scope_ref = scope;
    }

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
}