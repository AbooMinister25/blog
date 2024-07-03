use std::net::SocketAddr;
use std::path::Path;

use color_eyre::Result as EResult;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::StatusCode;
use hyper::{body::Incoming, service::Service};
use hyper::{Request, Response};
use hyper_staticfile::util::FileBytesStream;
use hyper_staticfile::vfs::TokioFileAccess;
use hyper_staticfile::{Body, Static};
use hyper_util::rt::TokioIo;
use hyper_util::service::{TowerToHyperService, TowerToHyperServiceFuture};
use tokio::fs::File;
use tokio::net::TcpListener;
use tower::{Layer, ServiceBuilder};
use tower_livereload::LiveReloadLayer;

#[derive(Debug, Clone)]
pub struct Logger<S> {
    inner: S,
}

impl<S> Logger<S> {
    pub fn new(inner: S) -> Self {
        Logger { inner }
    }
}
type Req = Request<Incoming>;

impl<S> Service<Req> for Logger<S>
where
    S: Service<Req>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;
    fn call(&self, req: Req) -> Self::Future {
        println!("processing request: {} {}", req.method(), req.uri().path());
        self.inner.call(req)
    }
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    static_: Static,
) -> Result<Response<Body>, std::io::Error> {
    let response = static_.clone().serve(req).await?;

    if !response.status().is_success() {
        match response.status() {
            StatusCode::NOT_FOUND => {
                let resp = Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::Full(FileBytesStream::new(TokioFileAccess::new(
                        File::open("public/404.html").await?,
                    ))))
                    .expect("Problem while opening 404 file.");
                return Ok(resp);
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                todo!()
            }
            _ => (),
        }
    }

    Ok(response)
}

pub async fn serve(livereload: LiveReloadLayer) -> EResult<()> {
    let static_ = Static::new(Path::new("public/"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    let listener = TcpListener::bind(addr).await?;

    println!("Server running on http://{addr}/");

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let reload = livereload.clone();

        let static_ = static_.clone();
        tokio::task::spawn(async move {
            let svc = tower::service_fn(move |req| handle_request(req, static_.clone()));
            let svc = ServiceBuilder::new().layer(reload).service(svc);
            let svc = TowerToHyperService::new(svc);
            let svc = ServiceBuilder::new().layer_fn(Logger::new).service(svc);
            if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                eprintln!("Error serving connection: {err:?}");
            }
        });
    }
}
