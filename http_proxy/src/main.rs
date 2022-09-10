use clap::Parser;
use futures_core::Stream;
use futures_util::StreamExt;
use hyper::client::{Client, HttpConnector};
use hyper::header::CONTENT_TYPE;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, Uri};
use std::convert::Infallible;
use std::error::Error as StdError;
use std::fmt;
use std::mem::take;
use std::net::SocketAddr;
use std::pin::Pin;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

#[derive(Parser, Clone)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long, value_parser)]
    robots: bool,

    #[clap(short, long, value_parser)]
    stats_page: bool,

    #[clap(value_parser)]
    port: u16,

    #[clap(value_parser)]
    destination_url: String,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    // Do this first because our service fn later will
    // consume args.
    let addr = SocketAddr::from(([0, 0, 0, 0], cli.port));

    let client = Arc::new(Client::new());
    let logger = Arc::new(Mutex::new(RequestLogger::new()));
    let make_service = make_service_fn(move |_conn| {
        let flags = cli.clone();
        let client_clone = client.clone();
        let logger_clone = logger.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                forward_request(
                    req,
                    flags.clone(),
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
    flags: Cli,
    client: Arc<Client<HttpConnector>>,
    logger: Arc<Mutex<RequestLogger>>,
) -> Result<Response<Body>, Infallible> {
    match forward_request_or_fail(req, flags, client, logger).await {
        Ok(res) => Ok(res),
        Err(err) => Ok(Response::new(Body::from(format!("{}", err)))),
    }
}

async fn forward_request_or_fail(
    req: Request<Body>,
    flags: Cli,
    client: Arc<Client<HttpConnector>>,
    logger: Arc<Mutex<RequestLogger>>,
) -> Result<Response<Body>, GenericError> {
    if flags.robots && req.uri().path() == "/robots.txt" {
        logger.lock().unwrap().log_robots(&req);
        return Ok(Response::new(Body::from(
            "User-agent: *\r\nDisallow: /\r\n",
        )));
    }
    if flags.stats_page && req.uri().path() == "/stats.txt" {
        let info = logger.lock().unwrap();
        return Ok(Response::new(Body::from(format!(
            "req bytes: {}\r\nresp bytes: {}\r\n",
            info.request_bytes, info.response_bytes
        ))));
    }

    let destination_uri = flags.destination_url.parse::<Uri>()?;
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

fn logging_body(wrapped: Body, logger: Arc<Mutex<RequestLogger>>, is_request: bool) -> Body {
    let logger_clone = logger.clone();
    let counter_stream = wrapped
        .inspect(move |obj| {
            if let Ok(data) = obj {
                let size = data.len() as i64;
                let mut logger = logger.lock().unwrap();
                if is_request {
                    logger.request_bytes += size;
                } else {
                    logger.response_bytes += size;
                }
            }
        })
        .chain(EmptyStream {
            drop_fn: Some(move || logger_clone.lock().unwrap().log_body_ended(is_request)),
        });
    Body::wrap_stream(counter_stream)
}

struct EmptyStream<F: FnOnce() -> ()> {
    drop_fn: Option<F>,
}

impl<F: FnOnce() -> ()> Stream for EmptyStream<F> {
    type Item = <Body as Stream>::Item;

    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(None)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(1))
    }
}

impl<F: FnOnce() -> ()> Drop for EmptyStream<F> {
    fn drop(&mut self) {
        if let Some(f) = take(&mut self.drop_fn) {
            f();
        }
    }
}
