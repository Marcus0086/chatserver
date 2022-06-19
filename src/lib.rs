use rocket::{
    form::Form,
    response::stream::{Event, EventStream},
    serde::{Deserialize, Serialize},
    tokio::select,
    tokio::sync::broadcast::{channel, error::RecvError, Sender},
    Shutdown, State,
};
use shuttle_service::ShuttleRocket;

#[macro_use]
extern crate rocket;

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Message {
    #[field(validate = len(..30))]
    pub room: String,
    #[field(validate = len(..20))]
    pub user_name: String,
    pub message: String,
}

#[get("/hello")]
fn hello() -> &'static str {
    "Hello World"
}

#[post("/post", data = "<form>")]
fn post(form: Form<Message>, queue: &State<Sender<Message>>) {
    let _res = queue.send(form.into_inner());
}

#[get("/events")]
async fn events(queue: &State<Sender<Message>>, mut end: Shutdown) -> EventStream![] {
    let mut rx = queue.subscribe();
    EventStream! {
        loop {
            let message = select! {
                message = rx.recv() => match message {
                    Ok(message) => message,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break
            };
            yield Event::json(&message);
        }
    }
}

#[shuttle_service::main]
async fn init() -> ShuttleRocket {
    let server = rocket::build()
        .manage(channel::<Message>(1024).0)
        .mount("/", routes![post, events, hello]);
    Ok(server)
}
