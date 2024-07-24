use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use color_eyre::Result;
use hyper::{body::Incoming, server::conn::http1, service::Service, Request, Response, StatusCode};
use hyper_staticfile::{util::FileBytesStream, vfs::TokioFileAccess, Body, Static};
use hyper_util::{rt::TokioIo, service::TowerToHyperService};
use tokio::{fs::File, net::TcpListener};
use tower::ServiceBuilder;
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

async fn handle_request<T: AsRef<Path> + Send>(
    req: Request<hyper::body::Incoming>,
    static_: Static,
    output_dir: T,
) -> Result<Response<Body>, std::io::Error> {
    let response = static_.clone().serve(req).await?;

    if !response.status().is_success() || !response.status().is_redirection() {
        match response.status() {
            StatusCode::NOT_FOUND => {
                let resp = Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::Full(FileBytesStream::new(TokioFileAccess::new(
                        File::open(output_dir.as_ref().join("404/index.html")).await?,
                    ))))
                    .expect("Problem while opening 404 file.");
                return Ok(resp);
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                let resp = Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::Full(FileBytesStream::new(TokioFileAccess::new(
                        File::open(output_dir.as_ref().join("500.html")).await?,
                    ))))
                    .expect("Problem while opening 500 file.");
                return Ok(resp);
            }
            _ => (),
        }
    }

    Ok(response)
}

#[allow(clippy::missing_errors_doc, clippy::redundant_pub_crate)]
pub async fn serve(livereload: LiveReloadLayer, output_dir: PathBuf) -> Result<()> {
    let static_ = Static::new(&output_dir);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    let listener = TcpListener::bind(addr).await?;

    println!("Server running on http://{addr}/");

    loop {
        tokio::select! {
            Ok((stream, _)) = listener.accept() => {
                let io = TokioIo::new(stream);
                let reload = livereload.clone();

                let static_ = static_.clone();
                let op = output_dir.clone();
                tokio::task::spawn(async move {
                    let svc =
                        tower::service_fn(move |req| handle_request(req, static_.clone(), op.clone()));
                    let svc = ServiceBuilder::new().layer(reload).service(svc);
                    let svc = TowerToHyperService::new(svc);
                    let svc = ServiceBuilder::new().layer_fn(Logger::new).service(svc);
                    if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                        eprintln!("Error serving connection: {err:?}");
                    }
                });
            },
            _ = tokio::signal::ctrl_c() => {
                eprintln!("Shutting down server");
                break;
            }
        }
    }

    Ok(())
}
