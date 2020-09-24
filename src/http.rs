use std::collections::VecDeque;
use std::convert::{From, TryFrom};

pub struct HttpHeader {
    pub name: String,
    pub value: String,
}
impl HttpHeader {
    pub fn new(name: &str, value: &str) -> HttpHeader {
        HttpHeader {
            name: name.into(),
            value: value.into(),
        }
    }
}
impl From<HttpHeader> for String {
    fn from(header: HttpHeader) -> Self {
        format!("{}: {}\r\n", header.name, header.value)
    }
}
impl TryFrom<String> for HttpHeader {
    type Error = ();
    fn try_from(_s: String) -> Result<Self, Self::Error> {
        Err(())
    }
}

pub enum HttpMethod {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
}
impl HttpMethod {
    pub fn read(bytes: &mut VecDeque<u8>) -> Option<HttpMethod> {
        let mut method_bytes = vec![];
        while bytes.get(method_bytes.len()) != Some(&(' ' as u8)) {
            method_bytes.push(bytes.get(method_bytes.len()).unwrap());
        }
        if let Ok(method) = HttpMethod::try_from(String::from_utf8_lossy(method_bytes)) {
            Some(method)
        } else {
            None
        }
    }
}
impl From<HttpMethod> for String {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::Get => "GET",
            HttpMethod::Head => "HEAD",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Connect => "CONNECT",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Trace => "TRACE",
            HttpMethod::Patch => "PATCH",
        }
        .into()
    }
}
impl TryFrom<String> for HttpMethod {
    type Error = ();
    fn try_from(method: String) -> Result<Self, Self::Error> {
        match &*method {
            "GET" => Ok(HttpMethod::Get),
            "HEAD" => Ok(HttpMethod::Head),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "CONNECT" => Ok(HttpMethod::Connect),
            "OPTIONS" => Ok(HttpMethod::Options),
            "TRACE" => Ok(HttpMethod::Trace),
            "PATCH" => Ok(HttpMethod::Patch),
            _ => Err(()),
        }
    }
}

pub enum HttpStatusCode {}
impl HttpStatusCode {
    pub fn get_status_message(status: u32) -> String {
        match status {
            // 1XX
            100 => "Continue",
            101 => "Switching Protocol",
            103 => "Early Hints",
            // 2XX
            200 => "OK",
            201 => "Created",
            202 => "Accepted",
            203 => "Non-Authoritative Information",
            204 => "No Content",
            205 => "Reset Content",
            206 => "Partial Content",
            // 3XX
            300 => "Multiple Choice",
            301 => "Moved Permanently",
            302 => "Found",
            303 => "See Other",
            304 => "Not Modified",
            307 => "Temporary Redirect",
            308 => "Permanent Redirect",
            // 4XX
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            406 => "Not Acceptable",
            407 => "Proxy Authentication Required",
            408 => "Request Timeout",
            409 => "Conflict",
            410 => "Gone",
            411 => "Length Required",
            412 => "Precondition Failed",
            413 => "Payload Too Large",
            414 => "URI Too Long",
            415 => "Unsupported Media Type",
            416 => "Range Not Satisfiable",
            417 => "Expectation Failed",
            418 => "I'm a teapot",
            425 => "Too Early",
            426 => "Upgrade Required",
            428 => "Precondition Required",
            429 => "Too Many Requests",
            431 => "Request Header Fields Too Large",
            451 => "Unavailable For Legal Reasons",
            // 5XX
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Timeout",
            505 => "HTTP Version Not Supported",
            506 => "Variant Also Negotiates",
            510 => "Not Extended",
            511 => "Network Authentication Required",
            // Everything else
            _ => "",
        }
        .into()
    }
}

pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub version: String,
    pub headers: Vec<HttpHeader>,
}
impl From<HttpRequest> for String {
    fn from(request: HttpRequest) -> Self {
        let mut out = String::new();
        out += &format!(
            "{} {} {}\r\n",
            Into::<String>::into(request.method),
            request.path,
            request.version,
        );
        for header in request.headers {
            let s: String = header.into();
            out += &s;
        }
        out
    }
}

pub struct HttpResponse<T: Into<String>> {
    pub version: String,
    pub status: u32,
    pub headers: Vec<HttpHeader>,
    pub content: Option<T>,
}
impl<T: Into<String>> From<HttpResponse<T>> for String {
    fn from(response: HttpResponse<T>) -> Self {
        let mut out = String::new();
        out += &format!(
            "{} {} {}\r\n",
            response.version,
            response.status,
            HttpStatusCode::get_status_message(response.status),
        );
        for header in response.headers {
            out += &Into::<String>::into(header);
        }
        out += "\r\n";
        if let Some(content) = response.content {
            out += &Into::<String>::into(content);
        }
        out
    }
}
