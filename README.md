<h1 align="center">Simple Gossiping P2P Application</h1>

<p align="center">
  <a href="#summary">Summary</a> •
  <a href="#usage">Usage</a> • 
  <a href="#libraries">Using libraries</a>
</p>

---

> [!IMPORTANT]
>
> _without using libraries explicitly implement p2p logic (like libp2p)_


## Summary
The peer should have a cli interface to start it and connect itself to the other peers. Once connected, the peer should send a random gossip message to all the other peers every N seconds. The messaging period should also be specifiable in the command line. When a peer receives a message from the other peers, it should print it in the console.

---

## Usage

> run first participant on localhost with port 8080 and messaging period 5 seconds
>
> ```sh
> cargo run -- --period=5 --port=8080 
> ```

> run second participant on localhost with port 8081 and messaging period 6 seconds, which will connect to port 8080 on localhost
>
>```sh
>cargo run -- --period=6 --port=8081 --connect=127.0.0.1:8080 
>```

> run third participant on localhost with port 8082 and messaging period 7 seconds, which will connect to port 8080 on localhost
>
>```sh
>cargo run -- --period=7 --port=8082 --connect=127.0.0.1:8080
>```

---
<h4>with <code>make</code> command:</h4>
<details>

> run first participant on localhost with port 8080 and messaging period 5 seconds
>
```sh
make run TICK=5 FROM=8080
```

> run second participant on localhost with port 8081 and messaging period 6 seconds, which will connect to port 8080 on localhost
>
```sh
make run TICK=6 FROM=8081 TO=8080
```

> run third participant on localhost with port 8082 and messaging period 7 seconds, which will connect to port 8080 on localhost
>
```sh
make run TICK=7 FROM=8082 TO=8080
```

</details>

---

<h2 id="libraries">Using libraries</h2>

- [![message-io](https://shields.io/badge/message_io-0.18.1-darkgreen)](https://docs.rs/message_io/0.18.1/message_io/index.html) is a library designed to simplify network programming. It offers easy-to-use abstractions for handling asynchronous message-based communication across various protocols like TCP, UDP, and WebSockets. The library aims to minimize the boilerplate code required for network operations, allowing developers to focus on the logic of their applications. It supports event-driven architecture, enabling efficient handling of incoming messages and network events through a single event loop.

- [![bincode](https://shields.io/badge/bincode-1.3.3-darkgreen)](https://docs.rs/bincode/1.3.3/bincode/index.html) is a library used for serializing and deserializing Rust data structures efficiently and compactly using a binary representation. It's commonly used when you need fast and compact serialization for purposes like saving to a file, sending data over the network, or for any other scenario where you want to convert Rust structures to a byte format and back. bincode works by automatically generating the serialization and deserialization code for you, requiring minimal manual intervention. This makes it very convenient for quickly implementing binary serialization without needing to worry about the specifics of the binary format.

- [![serde](https://shields.io/badge/serde-1.0.197-darkgreen)](https://docs.rs/serde/1.0.197/serde/index.html) is a library framework for serializing and deserializing data structures efficiently and generically. It supports various data formats, such as JSON, YAML, Bincode, and others, through extensible data format traits. serde is known for its high performance and strong type safety, making it a standard choice for handling data interchange in Rust applications.

- [![rand](https://shields.io/badge/rand-0.8.5-darkgreen)](https://docs.rs/rand/0.8.5/rand/index.html) is a library that provides utilities to generate random numbers, derive random values from various distributions, and perform other randomness-related tasks. It's a comprehensive solution for all needs related to randomness in Rust applications, offering both ease of use for common tasks and flexibility for more complex requirements.

