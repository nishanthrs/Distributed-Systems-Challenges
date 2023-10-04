use rustengan::*;

use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{StdoutLock, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum BroadcastPayload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {},
    Broadcast {
        message: i32,
    },
    BroadcastOk {},
    Read {},
    ReadOk {
        messages: Vec<i32>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk {},
}

/* Node in distributed system that handles broadcasting */
struct BroadcastNode {
    id: usize,
    node_id: String,
    messages: Vec<i32>,
    topology: HashMap<String, Vec<String>>,
}

impl Node<BroadcastPayload> for BroadcastNode {
    fn step(
        &mut self,
        input: Message<BroadcastPayload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        match input.body.payload {
            BroadcastPayload::Init { node_id, node_ids } => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: MessageBody {
                        msg_id: Some(self.id),
                        in_reply_to: match input.body.msg_id {
                            None => None,
                            Some(input_msg_id) => Some(input_msg_id),
                        },
                        payload: BroadcastPayload::InitOk {},
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("Failed to write reply data to output: stdout.");
                output
                    .write_all(b"\n")
                    .context("Failed to write newline to output: stdout.")?;

                self.node_id = node_id;
                self.id += 1; // NOTE: If there are multiple threads calling EchoNode, might have to put a lock here
            }
            BroadcastPayload::InitOk { .. } => {
                // Raise exception if receiving an InitOk message
                bail!("Received unexpected InitOk message!");
            }
            BroadcastPayload::Broadcast { message } => {
                self.messages.push(message);
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: MessageBody {
                        msg_id: Some(self.id),
                        in_reply_to: match input.body.msg_id {
                            None => None,
                            Some(input_msg_id) => Some(input_msg_id),
                        },
                        payload: BroadcastPayload::BroadcastOk {},
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("Failed to write reply data to output: stdout.");
                output
                    .write_all(b"\n")
                    .context("Failed to write newline to output: stdout.")?;
                self.id += 1
            }
            BroadcastPayload::BroadcastOk { .. } => {
                bail!("Received unexpected BroadcastOk message!");
            }
            BroadcastPayload::Read { .. } => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: MessageBody {
                        msg_id: Some(self.id),
                        in_reply_to: match input.body.msg_id {
                            None => None,
                            Some(input_msg_id) => Some(input_msg_id),
                        },
                        payload: BroadcastPayload::ReadOk {
                            messages: self.messages.clone(),
                        },
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("Failed to write reply data to output: stdout.");
                output
                    .write_all(b"\n")
                    .context("Failed to write newline to output: stdout.")?;
                self.id += 1;
            }
            BroadcastPayload::ReadOk { .. } => {
                bail!("Received unexpected ReadOk message!");
            }
            BroadcastPayload::Topology { topology } => {
                self.topology = topology; // topology ptr is invalidated as owner of hashmap data
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: MessageBody {
                        msg_id: Some(self.id),
                        in_reply_to: match input.body.msg_id {
                            None => None,
                            Some(input_msg_id) => Some(input_msg_id),
                        },
                        payload: BroadcastPayload::TopologyOk {},
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("Failed to write reply data to output: stdout.");
                output
                    .write_all(b"\n")
                    .context("Failed to write newline to output: stdout.")?;
                self.id += 1;
            }
            BroadcastPayload::TopologyOk { .. } => {
                bail!("Received unexpected TopologyOk message!");
            }
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let broadcast_node = BroadcastNode {
        id: 0,
        node_id: String::new(), // I know this is bad code; will refactor this to only init node once Init message is sent
        messages: Vec::new(),
        topology: HashMap::new(),
    };
    main_loop(broadcast_node)
}
