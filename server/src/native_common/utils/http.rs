use bytes::Bytes;
use headers::{ContentLength, ContentType, HeaderMapExt};
use hyper::{Response, StatusCode};
use tihu_native::http::Body;

pub fn response_html<T: Into<Bytes>>(body: T) -> Response<Body> {
    let body: Bytes = body.into();
    let content_length = body.len() as u64;
    let mut response = Response::new(body.into());
    response.headers_mut().typed_insert(ContentType::html());
    response
        .headers_mut()
        .typed_insert(ContentLength(content_length));
    return response;
}

pub fn response_text<T: Into<Bytes>>(body: T) -> Response<Body> {
    let body: Bytes = body.into();
    let content_length = body.len() as u64;
    let mut response = Response::new(body.into());
    response
        .headers_mut()
        .typed_insert(ContentType::text_utf8());
    response
        .headers_mut()
        .typed_insert(ContentLength(content_length));
    return response;
}

pub fn response_json<T: Into<Bytes>>(body: T) -> Response<Body> {
    let body: Bytes = body.into();
    let content_length = body.len() as u64;
    let mut response = Response::new(body.into());
    response.headers_mut().typed_insert(ContentType::json());
    response
        .headers_mut()
        .typed_insert(ContentLength(content_length));
    return response;
}

pub fn response_not_found() -> Response<Body> {
    let status_code = StatusCode::NOT_FOUND;
    let status_text = status_code.canonical_reason().unwrap_or("Not Found");
    let content_length = status_text.len() as u64;
    let mut response = Response::new(Body::from(status_text));
    *response.status_mut() = status_code;
    response
        .headers_mut()
        .typed_insert(ContentType::text_utf8());
    response
        .headers_mut()
        .typed_insert(ContentLength(content_length));
    return response;
}

pub fn response_not_modified() -> Response<Body> {
    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::NOT_MODIFIED;
    return response;
}

pub fn response_server_error<T: Into<Bytes>>(body: T) -> Response<Body> {
    let mut response = response_text(body);
    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    return response;
}

pub fn response_bad_request<T: Into<Bytes>>(body: T) -> Response<Body> {
    let mut response = response_text(body);
    *response.status_mut() = StatusCode::BAD_REQUEST;
    return response;
}
