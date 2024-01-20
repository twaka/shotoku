# shotoku

tool for sending concurrent requests and observing stream outputs

## usage

```
Usage: main [OPTIONS] <URL> <PAYLOAD_FILE>

Arguments:
  <URL>           URL of the server
  <PAYLOAD_FILE>  Path to the payload file

Options:
  -u, --vus <VUS>                Number of virtual users [default: 1]
  -d, --duration <DURATION>      Duration of the test [default: 30]
  -s, --spawn-rate <SPAWN_RATE>  Spawn rate of virtual users [default: 1]
  -h, --help                     Print help
  -V, --version                  Print version
```

example: request to llama.cpp server with 5 virtual users
```
cargo run -r -- http://localhost:8080/completion examples/llamacpp.jsonl -u 5
```

## demo

- llama.cpp server configuration 1
  ```
  ./server -m tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf
  ```

  https://github.com/twaka/shotoku/assets/8081197/803aebc2-7ba5-45de-b397-dfe7cbb9d499

- llama.cpp server configuration 2
  ```
  ./server -m tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf -np 10
  ```

  https://github.com/twaka/shotoku/assets/8081197/b93d5279-f24c-46a0-952e-f4523684efa8

- llama.cpp server configuration 3
  ```
  ./server -m tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf -np 10 -cb
  ```

  https://github.com/twaka/shotoku/assets/8081197/f0fa75ad-7753-40d4-a138-27c3300d1743
