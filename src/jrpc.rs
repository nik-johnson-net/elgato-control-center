use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use anyhow::Result;
use futures::{Sink, SinkExt, Stream, StreamExt};
use reqwest_websocket::Message;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use tokio::{
    select,
    sync::{
        mpsc,
        oneshot::{self, Sender},
    },
};

const JSONRPC20: &str = "2.0";

#[derive(Serialize, Deserialize)]
struct SendContainer<'a> {
    jsonrpc: &'a str,
    id: Option<u64>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RecvContainer {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<Value>,
    id: Option<u64>,
}

#[derive(Clone)]
pub struct Jrpc {
    requests: Arc<Mutex<BTreeMap<u64, Sender<Value>>>>,
    counter: Arc<AtomicU64>,
    sender: mpsc::Sender<String>,
}

impl Jrpc {
    pub fn handle<
        T: Stream<Item = Result<Message, reqwest_websocket::Error>>
            + Sink<Message>
            + std::marker::Unpin
            + Send
            + 'static,
    >(
        transport: T,
    ) -> Jrpc {
        let (tx, rx) = mpsc::channel(1);
        let jrpc = Jrpc {
            requests: Arc::default(),
            counter: Arc::default(),
            sender: tx,
        };

        tokio::spawn(run(transport, jrpc.clone(), rx));

        jrpc
    }

    pub async fn send<M: Into<String>, V: DeserializeOwned>(
        &self,
        method: M,
        params: Option<Value>,
    ) -> Result<V> {
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();

        {
            let mut map = self.requests.lock().unwrap();
            map.insert(id, tx);
        }

        let message = SendContainer {
            jsonrpc: JSONRPC20,
            id: Some(id),
            method: method.into(),
            params,
        };

        self.sender.send(serde_json::to_string(&message)?).await?;

        let response: Value = rx.await.unwrap();
        let deserialized: V = serde_json::from_value(response)?;
        Ok(deserialized)
    }

    async fn receive(&self, container: RecvContainer) {
        if let Some(id) = container.id {
            {
                let mut map = self.requests.lock().unwrap();
                let parked_task = map.remove(&id);
                if let Some(task) = parked_task {
                    if let Some(result) = container.result {
                        task.send(result).unwrap();
                    } else if let Some(error) = container.error {
                        println!("Got an error: {:?}", error);
                        task.send(error).unwrap();
                    } else {
                        task.send(Value::Null).unwrap();
                    }
                } else {
                    // Got a response for an already completed task.
                }
            }
        } else {
            // broadcast message
            println!("Got broadcast message {:#?}", container)
        }
    }
}

async fn run<
    T: Stream<Item = Result<Message, reqwest_websocket::Error>> + Sink<Message> + std::marker::Unpin,
>(
    mut transport: T,
    jrpc: Jrpc,
    mut send_queue: mpsc::Receiver<String>,
) {
    loop {
        select! {
          opt = transport.next() => {
            match opt {
              Some(Ok(Message::Text(data))) => {
                let deserialized: RecvContainer = serde_json::from_str(&data).unwrap();
                jrpc.receive(deserialized).await;
              },
              Some(Ok(Message::Binary(_))) => {
                eprintln!("Error: Control Center returned binary data. This is a bug or intentional breaking change from elgato.");
                return;
              },
              Some(Err(e)) => {
                eprintln!("Error: {:?}", e);
                return;
              }
              None => {
                return;
              }
            }
          },
          opt = send_queue.recv() => {
            if let Some(data) = opt {
              let _ = transport.send(Message::Text(data)).await;
            } else {
              return;
            }
          }
        }
    }
}
