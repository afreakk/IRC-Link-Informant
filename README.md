## IRC Link Informant
IRC bot that post title of links posted in a channel.   
Written in [Rust](https://www.rust-lang.org/).
### Installation:
```sh
git clone git@github.com:afreakk/IRC-Link-Informant.git
cd IRC-Link-Informant
cargo build --release
```

### Usage:
```sh
cp Settings.toml.examle Settings.toml
vim Settings.toml #set nick, channel, server
target/release/titlebot #working directory needs to contain Settings.toml
```
