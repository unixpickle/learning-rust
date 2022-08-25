use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use std::env::args;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let args = Vec::from_iter(args());
    if args.len() != 3 {
        println!("Usage: {} <listen_port> <destination_url>", args[0]);
        return ExitCode::from(1);
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], args[1].parse().unwrap()));
    let destination_url = args[2].clone();
    let make_service = make_service_fn(move |_conn| {
        let d_url = destination_url.clone();
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

async fn forward_request<'a>(req: Request<Body>, destination_url: String) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from(destination_url)))
}
