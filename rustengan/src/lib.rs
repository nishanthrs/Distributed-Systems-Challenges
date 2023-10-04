use anyhow::Context;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Deserializer;
use std::io::StdoutLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<Payload> {
    pub src: String,
    pub dest: String,
    pub body: MessageBody<Payload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBody<Payload> {
    pub msg_id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}

pub trait Node<T> {
    fn step(&mut self, input: Message<T>, output: &mut StdoutLock) -> anyhow::Result<()>;
}

pub fn main_loop<S, Payload>(mut state: S) -> anyhow::Result<()>
where
    S: Node<Payload>,
    Payload: DeserializeOwned,
{
    let stdin = std::io::stdin().lock();
    let inputs = Deserializer::from_reader(stdin).into_iter::<Message<Payload>>();
    let mut stdout = std::io::stdout().lock();
    for input in inputs {
        let input = input.context("Maelstrom input could not be deserialized!")?;
        state
            .step(input, &mut stdout)
            .context("Node step function failed")?;
    }
    Ok(())
}
