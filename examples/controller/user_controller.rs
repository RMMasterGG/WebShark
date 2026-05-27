use bytes::Bytes;
use webshark::{Request, Response};
use crate::helpers::html_file::send_http_file;

pub fn home_handler() -> Response<Bytes> {
    let file = send_http_file("home.html".to_string()).unwrap();

    Response::ok_body(file)
}


pub fn post_handler(req: Request<Bytes>) -> Response<Bytes> {
    Response::ok_body(req.body_bytes())
}