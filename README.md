Redrust
===

Rewrite of [DiceDB (redis-compatible KV store)](https://github.com/DiceDB/dice) in [Rust](https://www.rust-lang.org/).

## Pipelining Example

Individual Command:
```
PING:       *1\r\n$4\r\nPING\r\n
SET key value:    *3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n
GET key:      *2\r\n$3\r\nGET\r\n$3\r\nkey\r\n
```

Pipelined:
```
$ (printf '*1\r\n$4\r\nPING\r\n*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n';) | nc localhost 7379
```

Expected result:
```
+PONG
+OK
$5
value
```
