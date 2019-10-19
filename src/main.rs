use std::path::{Component, Path};
use std::sync::atomic::{AtomicU16, Ordering::Relaxed};
use std::{env, io};
use std::io::Empty;
use tiny_http::{Response, Method};

type BoxError = Box<dyn std::error::Error>;

static HEALTH_STATUS: AtomicU16 = AtomicU16::new(200);

fn get_status(_path: &Path) -> Response<Empty> {
    let status_code = HEALTH_STATUS.load(Relaxed);
    Response::empty(status_code)
}

fn put_status(path: &Path) -> Result<Response<Empty>, BoxError> {
    if let Some(Component::Normal(status_code)) = path.components().nth(2) {
        let status_code = status_code.to_str().ok_or("invalid status code")?;
        let status_code = status_code.parse()?;
        HEALTH_STATUS.store(status_code, Relaxed);
        Ok(Response::empty(204))
    }
    else {
        Ok(Response::empty(404))
    }
}

fn main() -> Result<(), io::Error> {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8000".into());

    let server = tiny_http::Server::http(addr).unwrap();
    let addr = server.server_addr();
    println!("Listening on {}", addr);

    loop {
        let request = match server.recv() {
            Ok(request) => request,
            Err(e) => { eprintln!("{}", e); continue }
        };

        let url = request.url().to_owned();
        let path = Path::new(&url);
        let method = request.method();

        let result =
            if path == Path::new("/health") && method == &Method::Get {
                request.respond(get_status(path))
            }
            else if path.starts_with("/health/") && method == &Method::Put {
                match put_status(path) {
                    Ok(response) => request.respond(response),
                    Err(error) => {
                        let message = error.to_string();
                        let response = Response::from_string(message).with_status_code(400);
                        request.respond(response)
                    },
                }
            }
            else {
                request.respond(Response::empty(404))
            };

        if let Err(e) = result {
            eprintln!("{}", e);
        }
    }
}
