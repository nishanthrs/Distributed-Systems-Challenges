# Distributed Systems Challenges

Set of learning exercises to implement and better understand concepts in distributed systems:
* rustengan: Gossip Glomers, [distributed system challenges from Fly.io](https://fly.io/blog/gossip-glomers/)
* [Installing Maelstrom](https://github.com/jepsen-io/maelstrom/blob/main/doc/01-getting-ready/index.md)
* [Maelstrom Protocol](https://github.com/jepsen-io/maelstrom/blob/main/doc/protocol.md)

Running Echo Executable:
```bash
# cd to maelstrom repo
# Locate Rust binary
./maelstrom test -w echo --bin ../gossip_glomers/rustengan/target/debug/echo_node --node-count 1 --time-limit 10
```
Running Unique IDs Executable:
```bash
# cd to maelstrom repo
# Locate Rust binary
./maelstrom test -w unique-ids --bin ../gossip_glomers/rustengan/target/debug/unique_id_node --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition
```
