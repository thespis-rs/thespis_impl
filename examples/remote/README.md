# Remote Actor Example

This example has 3 peers talking to eachother over tcp connections. To run the example, it's advised to open 3 different terminals and run them in the given order:

1. cargo run --bin peera
2. cargo run --bin peerb
3. cargo run --bin peerc

You can watch the interaction between **peera** and **peerb** before launching **peerc**

- **peera** provides 2 services: ServiceA returns "pong" and ServiceB which does not return anything.
- **peerb** will send and call **peera**, as well as relay for **peerc** which will talk to **peera** thanks to the relaying of **peerb**.
