# Ark file system storage

File system storage implementation for writing key value pairs to disk.

## Steps to use CLI
- Create a test.json file of key:values pairs you want to store.
```json
{
  "key1": "value1",
  "key2": "value2",
  "key3": "value3"
}
```

- Run Write Command
```bash
cargo run --example cli write /tmp/z test.json
```

- Run Read Command
```bash
cargo run --example cli read /tmp/z key1,key2
```

- Get Output
```bash
key1: value1
key2: value2
```

- To get all key value pairs
```bash
cargo run --example cli read /tmp/z
```
