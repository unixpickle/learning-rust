use futures_core::stream::Stream;
use hyper::body::Bytes;
use hyper::client::{Client, HttpConnector};
use hyper::header::CONTENT_TYPE;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, Uri};
use std::convert::Infallible;
use std::env::args;
use std::error::Error as StdError;
use std::fmt;
use std::net::SocketAddr;
use std::pin::Pin;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

#[tokio::main]
async fn main() -> ExitCode {
    let args = Vec::from_iter(args());
    if args.len() != 3 {
        eprintln!("Usage: {} <listen_port> <destination_url>", args[0]);
        return ExitCode::from(1);
    }

    // Do this first because our service fn later will
    // consume args.
    let addr = SocketAddr::from(([0, 0, 0, 0], args[1].parse().unwrap()));

    let client = Arc::new(Client::new());
    let logger = Arc::new(Mutex::new(RequestLogger::new()));
    let make_service = make_service_fn(move |_conn| {
        let d_url = args[2].clone();
        let client_clone = client.clone();
        let logger_clone = logger.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                forward_request(
                    req,
                    d_url.clone(),
                    client_clone.clone(),
                    logger_clone.clone(),
                )
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    ExitCode::SUCCESS
}

async fn forward_request(
    req: Request<Body>,
    destination_url: String,
    client: Arc<Client<HttpConnector>>,
    logger: Arc<Mutex<RequestLogger>>,
) -> Result<Response<Body>, Infallible> {
    match forward_request_or_fail(req, destination_url, client, logger).await {
        Ok(res) => Ok(res),
        Err(err) => Ok(Response::new(Body::from(format!("{}", err)))),
    }
}

async fn forward_request_or_fail(
    req: Request<Body>,
    destination_url: String,
    client: Arc<Client<HttpConnector>>,
    logger: Arc<Mutex<RequestLogger>>,
) -> Result<Response<Body>, GenericError> {
    if req.uri().path() == "/robots.txt" {
        logger.lock().unwrap().log_robots(&req);
        return Ok(Response::new(Body::from(
            "User-agent: *\r\nDisallow: /\r\n",
        )));
    }

    let destination_uri = destination_url.parse::<Uri>()?;
    let source_uri = req.uri().clone();
    let forward_uri = Uri::builder()
        .scheme(destination_uri.scheme().unwrap().clone())
        .authority(destination_uri.authority().unwrap().clone())
        .path_and_query(source_uri.path_and_query().unwrap().clone())
        .build()?;

    logger.lock().unwrap().log_request(&req, &forward_uri);

    let mut builder = Request::builder()
        .method(req.method().clone())
        .uri(forward_uri);

    if let Some(content_type) = req.headers().get(CONTENT_TYPE) {
        builder = builder.header(CONTENT_TYPE, content_type)
    }

    let new_req = builder.body(logging_body(req.into_body(), logger.clone(), true))?;
    let response = client.request(new_req).await?;
    Ok(response.map(|body| logging_body(body, logger, false)))
}

#[derive(Debug, Clone)]
struct GenericError {
    msg: String,
}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl<E: StdError> From<E> for GenericError {
    fn from(x: E) -> Self {
        GenericError {
            msg: format!("hyper error: {}", x),
        }
    }
}

struct RequestLogger {
    request_bytes: i64,
    response_bytes: i64,
}

impl RequestLogger {
    fn new() -> RequestLogger {
        RequestLogger {
            request_bytes: 0,
            response_bytes: 0,
        }
    }

    fn log_robots(&mut self, req: &Request<Body>) {
        println!(
            "{} {} => built-in response (headers: {})",
            req.method(),
            req.uri(),
            RequestLogger::format_headers(req)
        )
    }

    fn log_request(&mut self, req: &Request<Body>, forward_uri: &Uri) {
        println!(
            "{} {} => {} (headers: {})",
            req.method(),
            req.uri(),
            forward_uri,
            RequestLogger::format_headers(req),
        )
    }

    fn format_headers(req: &Request<Body>) -> String {
        let mut header_strs = Vec::<String>::new();
        for (name, value) in req.headers() {
            if let Ok(value_str) = std::str::from_utf8(value.as_bytes()) {
                header_strs.push(format!("{}={}", name, value_str));
            }
        }
        header_strs.join(" ")
    }

    fn log_body_ended(&mut self, is_request: bool) {
        if is_request {
            println!("-  total request bytes: {}", self.request_bytes);
        } else {
            println!("- total response bytes: {}", self.response_bytes);
        }
    }
}

struct LoggingStream<S: Stream<Item = hyper::Result<Bytes>>> {
    wrapped: S,
    logger: Arc<Mutex<RequestLogger>>,
    is_request: bool,
}

impl<S: Stream<Item = hyper::Result<Bytes>>> LoggingStream<S> {
    fn get_wrapped(self: Pin<&mut Self>) -> Pin<&mut S> {
        // Using the "Pinning is structural for field" pattern as describe in:
        // https://doc.rust-lang.org/std/pin/index.html#structs.
        //
        // This requires that our struct is not packed, among other restrictions
        // in how the struct is implemented.
        unsafe { self.map_unchecked_mut(|s| &mut s.wrapped) }
    }
}

impl<S: Stream<Item = hyper::Result<Bytes>>> Stream for LoggingStream<S> {
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let res = self.as_mut().get_wrapped().poll_next(cx);
        match res {
            Poll::Ready(Some(Ok(data))) => {
                let size = data.len() as i64;
                let mut logger = self.logger.lock().unwrap();
                if self.is_request {
                    logger.request_bytes += size;
                } else {
                    logger.response_bytes += size;
                }
                Poll::Ready(Some(Ok(data)))
            }
            x => x,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        return self.wrapped.size_hint();
    }
}

impl<S: Stream<Item = hyper::Result<Bytes>>> Drop for LoggingStream<S> {
    fn drop(&mut self) {
        inner_drop(unsafe { Pin::new_unchecked(self) });
        fn inner_drop<S: Stream<Item = hyper::Result<Bytes>>>(this: Pin<&mut LoggingStream<S>>) {
            let is_req = this.is_request;
            this.logger.lock().unwrap().log_body_ended(is_req);
        }
    }
}

fn logging_body(wrapped: Body, logger: Arc<Mutex<RequestLogger>>, is_request: bool) -> Body {
    Body::wrap_stream(LoggingStream {
        wrapped,
        logger,
        is_request,
    })
}
