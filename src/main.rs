use std::sync::atomic::{AtomicU16, Ordering::Relaxed};
use tide::{Body, Context};
use tide::http::{Response, StatusCode};

struct Data {
    health_status: AtomicU16,
}

fn main() -> Result<(), std::io::Error> {
    let data = Data {
        health_status: AtomicU16::new(200),
    };

    let mut app = tide::App::with_state(data);

    app.at("/health").get(|cx: Context<Data>| async move {

        let status = cx.state().health_status.load(Relaxed);
        let status = StatusCode::from_u16(status).unwrap();

        Response::builder()
            .status(status)
            .body(Body::empty())
            .unwrap()
    });

    app.at("/health/:status").put(|cx: Context<Data>| async move {
        let status: u16 = cx.param("status").unwrap();
        cx.state().health_status.store(status, Relaxed);
    });

    app.run("127.0.0.1:8000")
}
