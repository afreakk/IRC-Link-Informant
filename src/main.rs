extern crate bufstream;
extern crate config;
extern crate linkify;
extern crate reqwest;
extern crate select;
extern crate serde;

use bufstream::BufStream;
use serde::Deserialize;
use std::io::prelude::*;
use std::net::TcpStream;

use select::document::Document;
use select::predicate::Name;

fn get_title_from_link(link: linkify::Link) -> Result<String, String> {
    reqwest::blocking::get(link.as_str())
        .map_err(|e| e.to_string())
        .and_then(|res| Document::from_read(res).map_err(|e| e.to_string()))
        .and_then(|doc| {
            doc.find(Name("title"))
                .next()
                .ok_or(format!("No title found on url {}", link.as_str()))
                .map(|elem| elem.text())
        })
}

fn print_and_discard(r: &Result<String, String>) {
    match r {
        Ok(v) => println!("{}", v),
        Err(e) => println!("error: {}", e),
    }
}

#[derive(Debug, Deserialize)]
struct Settings {
    channel: String,
    server: String,
    nick: String,
    name: String,
    user: String,
}
fn get_settings() -> Result<Settings, config::ConfigError> {
    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("Settings"))?;
    settings.try_into()
}

fn main() -> Result<(), String> {
    get_settings().map_err(|e| e.to_string()).and_then(|s| {
        TcpStream::connect(&s.server)
            .map_err(|e| e.to_string())
            .map(|tcp_stream| BufStream::new(tcp_stream))
            .and_then(|mut bufstream| {
                send_raw_msg_to_stream(&mut bufstream, &format!("NICK {}", &s.nick))
                    .and(send_raw_msg_to_stream(
                        &mut bufstream,
                        &format!("USER {} 0 * :{}", &s.user, &s.name),
                    ))
                    .and(send_raw_msg_to_stream(
                        &mut bufstream,
                        &format!("JOIN {}", &s.channel),
                    ))
                    .map(|_| irc_loop(bufstream, s))
            })
    })
}

fn irc_loop(mut bufstream: BufStream<TcpStream>, s: Settings) {
    let split_by_channel = format!("PRIVMSG {}", &s.channel);
    let mut buffer = String::new();
    while let Ok(_) = bufstream.read_line(&mut buffer) {
        print!(">> {}", buffer);
        if buffer.starts_with("PING") {
            print_and_discard(&send_raw_msg_to_stream(
                &mut bufstream,
                &buffer.replace("PING", "PONG").trim_end(),
            ));
        } else {
            buffer.split(&split_by_channel).nth(1).map(|chan_msg| {
                linkify::LinkFinder::new().links(chan_msg).for_each(|link| {
                    print_and_discard(&get_title_from_link(link).and_then(|title| {
                        send_raw_msg_to_stream(&mut bufstream, &as_channel_msg(&s.channel, &title))
                    }))
                })
            });
        }
        buffer.clear();
    }
}

fn as_channel_msg(channel: &str, msg: &str) -> String {
    format!("PRIVMSG {} :{}", channel, msg)
}

fn send_raw_msg_to_stream<W: Write>(w: &mut W, msg: &str) -> Result<String, String> {
    let to_write = format!("{}\r\n", msg);
    w.write(to_write.as_bytes())
        .and(w.flush())
        .map_err(|e| e.to_string())
        .map(|_| format!("<< {}", to_write))
}
