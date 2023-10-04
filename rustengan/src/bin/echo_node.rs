use rustengan::*;

use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum EchoPayload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {},
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
}

struct EchoNode {
    // Node in distributed system that handles echo functionality
    id: usize,
}

impl Node<EchoPayload> for EchoNode {
    fn step(&mut self, input: Message<EchoPayload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.payload {
            EchoPayload::Init { .. } => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: MessageBody {
                        msg_id: Some(self.id),
                        in_reply_to: match input.body.msg_id {
                            None => None,
                            Some(input_msg_id) => Some(input_msg_id),
                        },
                        payload: EchoPayload::InitOk {},
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("Failed to write reply data to output: stdout.");
                output
                    .write_all(b"\n")
                    .context("Failed to write newline to output: stdout.")?;
                self.id += 1; // NOTE: If there are multiple threads calling EchoNode, might have to put a lock here
            }
            EchoPayload::Echo { echo } => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: MessageBody {
                        msg_id: Some(self.id),
                        in_reply_to: match input.body.msg_id {
                            None => None,
                            Some(input_msg_id) => Some(input_msg_id),
                        },
                        payload: EchoPayload::EchoOk { echo },
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("Failed to write reply data to output: stdout.");
                output
                    .write_all(b"\n")
                    .context("Failed to write newline to output: stdout.")?;
                self.id += 1; // NOTE: If there are multiple threads calling EchoNode, might have to put a lock here
            }
            EchoPayload::EchoOk { .. } => {
                // Raise exception if receiving an EchoOk message
                bail!("Received unexpected EchoOk message!");
            }
            EchoPayload::InitOk { .. } => {
                // Raise exception if receiving an InitOk message
                bail!("Received unexpected InitOk message!");
            }
        };

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let echo_node = EchoNode { id: 0 };
    main_loop(echo_node)
}
