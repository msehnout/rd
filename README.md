## How to build:

Install rust from package:
```
# dnf install cargo rust
```
or upstream:
```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

and compile it:
```
$ cargo build --release
```

you can run the resulting executable using cargo:
```
$ cargo run -- --help
```
or directly:
```
$ target/release/rd --help
```
