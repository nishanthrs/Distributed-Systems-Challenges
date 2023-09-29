use rustengan::*;

use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::io::{StdoutLock, Write};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum UniqueIDPayload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {},
    Generate {},
    GenerateOk {
        id: String,
    },
}

struct UniqueIDNode {
    // Node in distributed system that handles unique ID generation
    id: usize,
}

impl UniqueIDNode {
    fn gen_unique_id(&self, dest_node_id: &str) -> String {
        /*
        ID will be generated as a string consisting of:
        1. Unix timestamp in seconds
        2. Node ID it's generated on
        3. Msg ID (auto-increment counter)
        This format ensures that even if there are multiple nodes generating this ID every second,
        these IDs are guaranteed to be unique and sortable

        NOTE[1]: The timestamp is not needed for the purposes of the challenge.
        However, it does allow the IDs to be sortable, which is a nice bonus feature.
        NOTE[2]: Inherent flaw in this design is that it's not a fixed amount of bits (due to node and message ID).
        We should use an integer of fixed size x bits for message ID.
        Note that this means we'd have to reset the ID back to 0 every second,
        meaning the limit of the system is generating 2^x IDs per second.
        We can increase this by a factor of 1000x by storing the unix timestamp in milliseconds.
        */
        let curr_ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards?")
            .as_secs();
        return format!(
            "{}_{}_{}",
            curr_ts.to_string(),
            dest_node_id,
            self.id.to_string()
        );
    }

    pub fn step(
        &mut self,
        input: Message<UniqueIDPayload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        match input.body.payload {
            UniqueIDPayload::Init { .. } => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: MessageBody {
                        msg_id: Some(self.id),
                        in_reply_to: match input.body.msg_id {
                            None => None,
                            Some(input_msg_id) => Some(input_msg_id),
                        },
                        payload: UniqueIDPayload::InitOk {},
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("Failed to write reply data to output: stdout.");
                output
                    .write_all(b"\n")
                    .context("Failed to write newline to output: stdout.")?;
                self.id += 1; // NOTE: If there are multiple threads calling EchoNode, might have to put a lock here
            }
            UniqueIDPayload::InitOk { .. } => {
                // Raise exception if receiving an InitOk message
                bail!("Received unexpected InitOk message!");
            }
            UniqueIDPayload::Generate { .. } => {
                let unique_id = self.gen_unique_id(&input.dest);
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: MessageBody {
                        msg_id: Some(self.id),
                        in_reply_to: match input.body.msg_id {
                            None => None,
                            Some(input_msg_id) => Some(input_msg_id),
                        },
                        payload: UniqueIDPayload::GenerateOk { id: unique_id },
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("Failed to write reply data to output: stdout.");
                output
                    .write_all(b"\n")
                    .context("Failed to write newline to output: stdout.")?;
                self.id += 1
            }
            UniqueIDPayload::GenerateOk { .. } => {
                // Raise exception if receiving an GenerateOk message
                bail!("Received unexpected GenerateOk message!");
            }
        };

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();
    let inputs = Deserializer::from_reader(stdin).into_iter::<Message<UniqueIDPayload>>();
    let mut stdout = std::io::stdout().lock();
    let mut unique_id_node = UniqueIDNode { id: 0 };
    for input in inputs {
        let input = input.context("Maelstrom input could not be deserialized!")?;
        unique_id_node
            .step(input, &mut stdout)
            .context("Node step function failed")?;
    }
    Ok(())
}
