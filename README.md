<h1 align="center">Simple Gossiping P2P Application</h1>

<p align="center">
  <a href="#summary">Summary</a> •
  <a href="#usage">Usage</a> •
  <a href="#libraries">Production replacements</a>
</p>

---

> [!IMPORTANT]
>
> _without using libraries explicitly implement p2p logic (like libp2p)_


## Summary
The peer has a CLI interface to start it and connect to other peers. Once connected, the peer sends a pseudo-random gossip message to all known peers every `N` seconds. When a peer receives a message from another peer it prints it to the console.

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

<h2 id="libraries">Production replacements</h2>

This project intentionally avoids third‑party crates to serve as an educational
example. For production systems consider the following crates:

- [`tokio`](https://docs.rs/tokio) or [`mio`](https://docs.rs/mio) for
  asynchronous networking.
- [`serde`](https://docs.rs/serde) and [`bincode`](https://docs.rs/bincode) for
  efficient serialization.
- [`rand`](https://docs.rs/rand) for robust random number generation.

These libraries provide battle‑tested implementations that are more efficient
and feature rich than the minimal versions included here.

