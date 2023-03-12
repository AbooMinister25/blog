use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_staticfile::{Body, Static};
use std::io::Error as IoError;
use std::net::SocketAddr;
use std::path::Path;
use tokio::net::TcpListener;

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    static_: Static,
) -> Result<Response<Body>, IoError> {
    if req.uri().path() == "/" {
        todo!()
    } else {
        static_.clone().serve(req).await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let static_ = Static::new(Path::new("public/"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        let static_ = static_.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    stream,
                    service_fn(move |req| handle_request(req, static_.clone())),
                )
                .await
            {
                println!("Error serving connection: {:?}", err)
            }
        });
    }
}
