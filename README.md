## IRC Link Informant
IRC bot providing link titles for URLs pasted in a channel.   
Written in [Rust](https://www.rust-lang.org/).
### Build:
```sh
git clone git@github.com:afreakk/IRC-Link-Informant.git
cd IRC-Link-Informant
cargo build --release
```

### Usage:
```sh
cp Settings.toml.example Settings.toml
vim Settings.toml #set nick, channel, server
target/release/link_informant #working directory needs to contain Settings.toml
```
