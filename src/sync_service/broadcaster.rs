use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Mutex;
use std::task::{Context, Poll};
use std::time::Duration;

use crate::tickets::Ticket;
use actix_web::web::{Bytes, Data};
use actix_web::Error;
use futures::{Stream, StreamExt};
use serde_json::json;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::{interval_at, Instant};

pub struct Broadcaster {
    stores: HashMap<i32, Vec<Sender<Bytes>>>,
}

impl<'a> Broadcaster {
    pub fn create() -> Data<Mutex<Self>> {
        // Data ≃ Arc
        let me = Data::new(Mutex::new(Broadcaster::new()));
        // ping clients every 10 seconds to see if they are alive
        Broadcaster::spawn_ping(me.clone());
        me
    }
    pub fn new_client(&mut self, store_id: i32) -> Client {
        let (tx, rx) = channel(100);

        match tx
            .clone()
            .try_send(Bytes::from("data: {\"type\":\"connected\"}\n\n"))
        {
            Ok(res) => log::info!("connected sended {:?}", res),
            Err(err) => log::error!("sending connected error {:?}", err),
        }

        self.stores
            .entry(store_id)
            .or_insert_with(Vec::new)
            .push(tx);
        Client(rx)
    }

    pub fn send_new_ticket(&self, store_id: i32, ticket: &Ticket) {
        self.send(
            store_id,
            &json!({"type":"newticket", "payload":ticket}).to_string(),
        );
    }
    pub fn send(&self, store_id: i32, msg: &str) {
        let msg = Bytes::from(["data: ", msg, "\n\n"].concat());

        let clients = match self.stores.get(&store_id) {
            Some(it) => it,
            _ => {
                log::error!("could not retrive client with id {}", store_id);
                return;
            }
        };
        for client in clients {
            client.clone().try_send(msg.clone()).unwrap_or(());
        }
    }

    fn spawn_ping(me: Data<Mutex<Self>>) {
        actix_rt::spawn(async move {
            let mut task = interval_at(Instant::now(), Duration::from_secs(10));
            while let Some(_) = task.next().await {
                me.lock().unwrap().remove_stale_clients();
            }
        })
    }

    fn new() -> Self {
        Broadcaster {
            stores: HashMap::new(),
        }
    }

    fn take_store(&mut self, store_id: i32) -> Option<Vec<Sender<Bytes>>> {
        let store = self.stores.get_mut(&store_id)?;
        let old = std::mem::replace(store, Vec::new());
        Some(old)
    }
    fn remove_stale_clients(&mut self) {
        use std::env;
        let front_version = env::var("FRONT_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
        let ping_msg = format!(
            "data: {{\"type\":\"ping\", \"payload\":{{\"version\":\"{}\"}} }}\n\n",
            front_version
        );

        let ids: Vec<i32> = self.stores.keys().map(|k| k.to_owned()).collect();
        for store_id in ids {
            let mut store = match self.take_store(store_id) {
                Some(r) => r,
                None => {
                    return;
                }
            };
            for mut client in store.drain(..) {
                match client.try_send(Bytes::from(ping_msg.clone())) {
                    Ok(_) => self.stores.get_mut(&store_id).unwrap().push(client),
                    Err(err) => log::error!("client disconnected {:?}", err),
                }
            }
        }
    }
}

// wrap Receiver in own type, with correct error type
pub struct Client(Receiver<Bytes>);

impl Stream for Client {
    type Item = Result<Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.0).poll_next(cx) {
            Poll::Ready(Some(v)) => Poll::Ready(Some(Ok(v))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
