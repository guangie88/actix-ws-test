use actix::{Actor, ActorContext, StreamHandler, System};
use actix_web::{error, server, ws, App, HttpRequest, HttpResponse, Json, State};
use serde_derive::*;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
struct Task {
    wait_secs: Vec<u32>,
}

#[derive(Serialize)]
struct TaskResponse {
    task_id: String,
}

struct AppState {
    next_id: Arc<Mutex<u64>>,
}

// HTTP actor
struct MyWebSocket;

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self, AppState>;
}

impl StreamHandler<ws::Message, ws::ProtocolError> for MyWebSocket {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => {
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {}
            ws::Message::Text(text) => ctx.text(text),
            ws::Message::Binary(bin) => ctx.binary(bin),
            ws::Message::Close(_) => {
                ctx.stop();
            }
        }
    }
}

fn post_task((state, _task): (State<AppState>, Json<Task>)) -> Json<TaskResponse> {
    let mut next_id = state.next_id.lock().unwrap();
    let current_id = *next_id;
    *next_id += 1;

    Json(TaskResponse {
        task_id: format!("{}", current_id),
    })
}

fn status_ws(req: &HttpRequest<AppState>) -> Result<HttpResponse, error::Error> {
    ws::start(req, MyWebSocket)
}

fn main() {
    let sys = System::new("ws-test");

    server::new(|| {
        App::with_state(AppState {
            next_id: Arc::new(Mutex::new(0)),
        })
        .resource("/task/", |r| r.post().with(post_task))
        .resource("/status/", |r| r.f(status_ws))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .start();

    let _ = sys.run();
}
