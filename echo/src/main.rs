use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Serializer};
use std::io::{self, StdoutLock, Write};

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    src: String,

    #[serde(rename = "dest")]
    dst: String,

    body: Body,
}

#[derive(Serialize, Deserialize, Debug)]
struct Body {
    #[serde(rename = "msg_id")]
    id: Option<i32>,

    in_reply_to: Option<i32>,

    #[serde(flatten)]
    payload: Payload,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case", tag = "type")]
enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
    Init { node_id: String, node_ids: Vec<String> },
    InitOk,
}

struct EchoNode {
    id: String
}

impl EchoNode {
    pub fn handle(&mut self, msg: Message, output: &mut StdoutLock) {
        match msg.body.payload {
            Payload::EchoOk { echo: _ } => (),
            Payload::Echo { echo } => {
                let response = Message {
                    dst: msg.src,
                    src: self.id.clone(),
                    body: Body {
                        id: msg.body.id,
                        in_reply_to: msg.body.id,
                        payload: Payload::EchoOk { echo },
                    },
                };
                serde_json::to_writer(&mut *output, &response);
                output.write_all(b"\n");
            }
            Payload::Init { node_id, node_ids: _ } => {
                self.id = node_id;
                let response = Message {
                    dst: msg.src,
                    src: self.id.clone(),
                    body: Body {
                        id: msg.body.id,
                        in_reply_to: msg.body.id,
                        payload: Payload::InitOk,
                    },
                };
                serde_json::to_writer(&mut *output, &response);
                output.write_all(b"\n");
            }
            Payload::InitOk => (),
        }
    }
}

fn main() -> Result<(), io::Error> {
    let stdin = io::stdin().lock();

    let stdout = io::stdout();
    let mut output = stdout.lock();

    let mut node = EchoNode { id: String::new() };

    for msg in Deserializer::from_reader(stdin).into_iter::<Message>() {
        let msg = msg?;

        node.handle(msg, &mut output);
    }

    Ok(())
}
