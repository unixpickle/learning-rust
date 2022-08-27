use hyper::client::Client;
use hyper::header::CONTENT_TYPE;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, Uri};
use std::convert::Infallible;
use std::env::args;
use std::error::Error as StdError;
use std::fmt;
use std::net::SocketAddr;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let args = Vec::from_iter(args());
    if args.len() != 3 {
        eprintln!("Usage: {} <listen_port> <destination_url>", args[0]);
        return ExitCode::from(1);
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], args[1].parse().unwrap()));
    let make_service = make_service_fn(move |_conn| {
        let d_url = args[2].clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                forward_request(req, d_url.clone())
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
) -> Result<Response<Body>, Infallible> {
    match forward_request_or_fail(req, destination_url).await {
        Ok(res) => Ok(res),
        Err(err) => Ok(Response::new(Body::from(format!("{}", err)))),
    }
}

async fn forward_request_or_fail(
    req: Request<Body>,
    destination_url: String,
) -> Result<Response<Body>, GenericError> {
    let destination_uri = destination_url.parse::<Uri>()?;
    let source_uri = req.uri().clone();
    let forward_uri = Uri::builder()
        .scheme(destination_uri.scheme().unwrap().clone())
        .authority(destination_uri.authority().unwrap().clone())
        .path_and_query(source_uri.path_and_query().unwrap().clone())
        .build()?;

    let client = Client::new();

    let mut builder = Request::builder()
        .method(req.method().clone())
        .uri(forward_uri);

    if let Some(content_type) = req.headers().get(CONTENT_TYPE) {
        builder = builder.header(CONTENT_TYPE, content_type)
    }

    let new_req = builder.body(req.into_body())?;
    let response = client.request(new_req).await?;
    Ok(response)
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
