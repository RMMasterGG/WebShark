use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use bytes::Bytes;
use http::header::SEC_WEBSOCKET_KEY;
use sha1::{Digest, Sha1};
use crate::Request;

pub(crate) fn generate_accept_key(request: &Request<Bytes>) -> Option<String> {

    let client_key = request.get_header(SEC_WEBSOCKET_KEY)?.trim();

    let magic_uuid = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    let mut hasher = Sha1::new();

    let mut update_key = String::with_capacity(client_key.len() + magic_uuid.len());
    update_key.push_str(client_key);
    update_key.push_str(magic_uuid);

    hasher.update(update_key.as_bytes());

    let sha1_result = hasher.finalize();
    let accept_key = BASE64_STANDARD.encode(sha1_result);
    Some(accept_key)
}