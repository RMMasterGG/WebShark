use webshark::bytes::Bytes;
use webshark::macros::{controller, get, post};
use webshark::{Request, Response};
use webshark::cookie::Cookie;
use crate::helpers::html_file::send_http_file;

pub struct StoresController;

#[controller]
impl StoresController {

    #[get(path = "/sosal")]
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