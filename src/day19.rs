use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use actix::prelude::*;
use actix_web::{get, post, web, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use rand::{rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::json;

struct TableTennisWS {
    served: bool,
}

impl Actor for TableTennisWS {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for TableTennisWS {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                if text == "serve" {
                    self.served = true;
                } else if text == "ping" && self.served {
                    ctx.text("pong")
                }
            }
            _ => (),
        }
    }
}

#[get("/19/ws/ping")]
pub async fn day_19_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(TableTennisWS { served: false }, &req, stream);
    resp
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub id: usize,
    pub msg: String,
    pub room: i32,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub id: usize,
    pub name: i32,
}

#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rooms: HashMap<i32, HashSet<usize>>,
    rng: ThreadRng,
    tweet_count: Arc<AtomicUsize>,
}

impl ChatServer {
    pub fn new(tweet_count: Arc<AtomicUsize>) -> ChatServer {
        let rooms = HashMap::new();

        ChatServer {
            sessions: HashMap::new(),
            rooms,
            rng: rand::thread_rng(),
            tweet_count,
        }
    }
}

impl ChatServer {
    fn send_message(&self, room: i32, message: &str, skip_id: usize) {
        if let Some(sessions) = self.rooms.get(&room) {
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        addr.do_send(Message(message.to_owned()));
                    }
                }
            }
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _ctx: &mut Context<Self>) -> Self::Result {
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        id
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Context<Self>) {
        let mut rooms: Vec<i32> = Vec::new();

        if self.sessions.remove(&msg.id).is_some() {
            for (name, sessions) in &mut self.rooms {
                if sessions.remove(&msg.id) {
                    rooms.push(*name);
                }
            }
        }
    }
}

impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _ctx: &mut Context<Self>) {
        self.send_message(msg.room, msg.msg.as_str(), msg.id);
        self.tweet_count.fetch_add(1, Ordering::SeqCst);
    }
}

impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _ctx: &mut Context<Self>) {
        let Join { id, name } = msg;
        let mut rooms = Vec::new();

        for (n, sessions) in &mut self.rooms {
            if sessions.remove(&id) {
                rooms.push(n.to_owned());
            }
        }

        self.rooms.entry(name).or_default().insert(id);
    }
}

#[derive(Debug)]
pub struct WsChatSession {
    pub id: usize,
    pub room: i32,
    pub name: String,
    pub addr: Addr<ChatServer>,
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.addr
            .send(Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        self.addr.do_send(Disconnect { id: self.id });
        Running::Stop
    }
}

impl Handler<Message> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

#[derive(Deserialize)]
struct IncomingBirdAppMessage {
    message: String,
}

#[derive(Serialize)]
struct BroadcastBirdAppMessage {
    user: String,
    message: String,
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Text(text) => {
                let m: IncomingBirdAppMessage = serde_json::from_str(&text).unwrap();

                self.addr.do_send(ClientMessage {
                    id: self.id,
                    msg: m.message.clone(),
                    room: self.room,
                });

                ctx.text(
                    json!(BroadcastBirdAppMessage {
                        user: self.name.clone(),
                        message: m.message.clone()
                    })
                    .to_string(),
                )
            }
            _ => (),
        }
    }
}

#[post("/19/reset")]
pub async fn day_19_reset() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/19/views")]
async fn day_19_views(count: web::Data<AtomicUsize>) -> impl Responder {
    let current_count = count.load(Ordering::SeqCst);
    HttpResponse::Ok().body(current_count.to_string())
}

#[get("/19/ws/room/{room}/user/{user}")]
async fn day_19_chat(
    path: web::Path<(i32, String)>,
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<ChatServer>>,
) -> Result<HttpResponse, Error> {
    let path = path.into_inner();
    let room = path.0;
    let user = path.1;

    ws::start(
        WsChatSession {
            id: 0,
            room: room,
            name: user,
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}
