use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use url::Url;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
    pub fn parse<'a>(src: &'a str) -> nom::IResult<&'a str, HttpHeader> {
        match nom::sequence::tuple((
            nom::bytes::complete::take_while1(|c: char| {
                c.is_alphanumeric() || c == '-' || c == '_'
            }),
            nom::character::complete::space0,
            nom::character::complete::char(':'),
            nom::character::complete::space0,
            nom::bytes::complete::take_until("\r\n"),
            nom::character::complete::crlf,
        ))(src)
        {
            Ok((
                remaining_src,
                (
                    name,
                    _, // whitespace
                    _, // colon
                    _, // whitespace
                    value,
                    _, // newline
                ),
            )) => Ok((remaining_src, HttpHeader::new(name, value))),
            Err(e) => Err(e),
        }
    }
    pub fn emit(&self) -> Vec<u8> {
        format!("{}: {}\r\n", self.name, self.value)
            .as_bytes()
            .to_vec()
    }
}
impl Display for HttpHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.emit()))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct HttpStatus {
    pub value: usize,
}
impl HttpStatus {
    pub fn new(value: usize) -> Result<HttpStatus, ()> {
        match value {
            200 | 301 | 400 | 404 => Ok(HttpStatus { value }),
            _ => Err(()),
        }
    }
    pub fn parse<'a>(_src: &'a str) -> nom::IResult<&'a str, HttpStatus> {
        unimplemented!()
    }
    pub fn emit(&self) -> Vec<u8> {
        (match self.value {
            200 => "200 Ok",
            301 => "301 Moved Permanently",
            400 => "400 Bad Request",
            404 => "404 Not Found",
            _ => "",
        })
        .as_bytes()
        .to_vec()
    }
}
impl Display for HttpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.emit()))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
    pub fn new(value: &str) -> Result<HttpMethod, ()> {
        if let Ok((_rest, method)) = HttpMethod::parse(value) {
            Ok(method)
        } else {
            Err(())
        }
    }
    pub fn parse<'a>(src: &'a str) -> nom::IResult<&'a str, HttpMethod> {
        match nom::branch::alt((
            nom::bytes::complete::tag_no_case("GET"),
            nom::bytes::complete::tag_no_case("HEAD"),
            nom::bytes::complete::tag_no_case("POST"),
            nom::bytes::complete::tag_no_case("PUT"),
            nom::bytes::complete::tag_no_case("DELETE"),
            nom::bytes::complete::tag_no_case("CONNECT"),
            nom::bytes::complete::tag_no_case("OPTIONS"),
            nom::bytes::complete::tag_no_case("TRACE"),
            nom::bytes::complete::tag_no_case("PATCH"),
        ))(src)
        {
            Ok((remaining_src, method)) => match &method.to_ascii_uppercase()[..] {
                "GET" => Ok((remaining_src, HttpMethod::Get)),
                "HEAD" => Ok((remaining_src, HttpMethod::Head)),
                "POST" => Ok((remaining_src, HttpMethod::Post)),
                "PUT" => Ok((remaining_src, HttpMethod::Put)),
                "DELETE" => Ok((remaining_src, HttpMethod::Delete)),
                "CONNECT" => Ok((remaining_src, HttpMethod::Connect)),
                "OPTIONS" => Ok((remaining_src, HttpMethod::Options)),
                "TRACE" => Ok((remaining_src, HttpMethod::Trace)),
                "PATCH" => Ok((remaining_src, HttpMethod::Patch)),
                _ => Err(nom::Err::Failure(nom::error::Error::new(
                    src,
                    nom::error::ErrorKind::Verify,
                ))),
            },
            Err(e) => Err(e),
        }
    }
    pub fn emit(&self) -> Vec<u8> {
        (match self {
            HttpMethod::Get => "GET",
            HttpMethod::Head => "HEAD",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Connect => "CONNECT",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Trace => "TRACE",
            HttpMethod::Patch => "PATCH",
        })
        .as_bytes()
        .to_vec()
    }
}
impl Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.emit()))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: Url,
    pub version: String,
    pub headers: Vec<HttpHeader>,
}
impl HttpRequest {
    pub fn new(_value: &str) -> Result<HttpRequest, ()> {
        unimplemented!()
    }
    pub fn parse<'a>(src: &'a str) -> nom::IResult<&'a str, HttpRequest> {
        match nom::sequence::tuple((
            HttpMethod::parse,
            nom::character::complete::space0,
            nom::bytes::complete::take_until(" HTTP/"),
            nom::bytes::complete::tag(" HTTP/"),
            nom::character::complete::digit1,
            nom::character::complete::char('.'),
            nom::character::complete::digit1,
            nom::character::complete::crlf,
            nom::multi::many0(HttpHeader::parse),
            nom::character::complete::crlf,
        ))(src)
        {
            Ok((
                remaining_src,
                (
                    method,   // method
                    _,        // whitespace
                    path,     // path
                    _,        // http
                    version0, // version 0
                    _,        // period
                    version1, // version 1
                    _,        // newline
                    headers,  // headers
                    _,        // newline
                ),
            )) => {
                let base = Url::parse(&format!("http://{}/", crate::BIND_ADDR)).unwrap();
                if let Ok(path) = base.join(path) {
                    Ok((
                        remaining_src,
                        HttpRequest {
                            method,
                            path,
                            version: format!("{}.{}", version0, version1),
                            headers,
                        },
                    ))
                } else {
                    Err(nom::Err::Failure(nom::error::Error::new(
                        src,
                        nom::error::ErrorKind::Verify,
                    )))
                }
            }
            Err(e) => Err(e),
        }
    }
    pub fn emit(&self) -> Vec<u8> {
        let mut out = format!(
            "{} {} HTTP/{}\r\n",
            self.method,
            self.path.path(),
            self.version
        )
        .as_bytes()
        .to_vec();
        for header in &self.headers {
            out.append(&mut header.emit());
        }
        out.append(&mut b"\r\n".to_vec());
        out
    }
}
impl Display for HttpRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.emit()))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct HttpResponseBuilder {
    version: String,
    status: HttpStatus,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}
impl HttpResponseBuilder {
    pub fn new() -> HttpResponseBuilder {
        HttpResponseBuilder {
            version: "1.1".to_owned(),
            status: HttpStatus::new(200).unwrap(),
            headers: HashMap::new(),
            body: vec![],
        }
    }
    pub fn version(mut self, version: &str) -> HttpResponseBuilder {
        self.version = String::from(version);
        self
    }
    pub fn status(mut self, status: usize) -> HttpResponseBuilder {
        self.status = HttpStatus::new(status).unwrap();
        self
    }
    pub fn header(mut self, header_name: &str, header_value: &str) -> HttpResponseBuilder {
        self.headers
            .insert(String::from(header_name), String::from(header_value));
        self
    }
    pub fn body(mut self, body: Vec<u8>) -> HttpResponseBuilder {
        self.body = body;
        self
    }
    pub fn build(mut self) -> HttpResponse {
        let body_len = self.body.len();
        self = self.header("Content-Length", &body_len.to_string());
        self = self.header("Content-Type", "text/html");
        HttpResponse {
            version: self.version,
            status: self.status,
            headers: self.headers,
            body: self.body,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct HttpResponse {
    version: String,
    status: HttpStatus,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}
impl HttpResponse {
    pub fn new() -> HttpResponseBuilder {
        HttpResponseBuilder::new()
    }
    pub fn parse<'a>(_src: &'a str) -> nom::IResult<&'a str, HttpResponse> {
        unimplemented!()
    }
    pub fn emit(&self) -> Vec<u8> {
        let mut out = format!("HTTP/{} {}\r\n", self.version, self.status)
            .as_bytes()
            .to_vec();
        for header in &self.headers {
            out.append(&mut HttpHeader::new(header.0, header.1).emit());
        }
        out.append(&mut b"\r\n".to_vec());
        out.append(&mut self.body.clone());
        out
    }
    pub fn set_header(&mut self, header_name: &str, header_value: &str) {
        self.headers
            .insert(String::from(header_name), String::from(header_value));
    }
}
impl Display for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.emit()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_new() {
        let header = HttpHeader::new("X-Powered-By", "red bull and skittles");
        assert_eq!(header.name, "X-Powered-By");
        assert_eq!(header.value, "red bull and skittles");
    }
    #[test]
    fn header_parse() {
        let reference = HttpHeader::new("X-Powered-By", "red bull and skittles");
        let correct = "X-Powered-By: red bull and skittles\r\n";
        assert_eq!(Ok(("", reference)), HttpHeader::parse(correct));
        let incorrect_name = "X-Powe red-By: red bull and skittles\r\n";
        assert!(HttpHeader::parse(incorrect_name).is_err());
        let incorrect_colon = "X-Powered-By red bull and skittles\r\n";
        assert!(HttpHeader::parse(incorrect_colon).is_err());
        let incorrect_newline = "X-Powered-By: red bull and skittles";
        assert!(HttpHeader::parse(incorrect_newline).is_err());
    }

    #[test]
    fn method_parse() {
        let correct1 = "GET";
        assert_eq!(Ok(("", HttpMethod::Get)), HttpMethod::parse(correct1));
        let correct2 = "post";
        assert_eq!(Ok(("", HttpMethod::Post)), HttpMethod::parse(correct2));
        let correct3 = "puttest";
        assert_eq!(Ok(("test", HttpMethod::Put)), HttpMethod::parse(correct3));
    }

    #[test]
    fn request_parse() {
        let correct =
            "GET / HTTP/1.1\r\nX-Powered-By: red bull and skittles\r\nserver: GlassCannon\r\n\r\n";
        assert!(HttpMethod::parse(correct).is_ok());
    }
}
