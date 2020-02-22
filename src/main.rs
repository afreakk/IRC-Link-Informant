extern crate bufstream;
extern crate config;
extern crate linkify;
extern crate reqwest; // 0.9.18
extern crate select;
extern crate serde;

use bufstream::BufStream;
use serde::Deserialize;
use std::io::prelude::*;
use std::net::TcpStream;

use select::document::Document;
use select::predicate::Name;

fn get_title_from_link(link: &linkify::Link) -> Result<String, String> {
    reqwest::blocking::get(link.as_str())
        .map_err(|e| e.to_string())
        .and_then(|res| Document::from_read(res).map_err(|e| e.to_string()))
        .and_then(|x| {
            x.find(Name("title"))
                .next()
                .ok_or(format!("No title found on url {}", link.as_str()))
                .map(|n| n.text())
        })
}

fn extract_links(s: &str) -> Vec<linkify::Link> {
    linkify::LinkFinder::new().links(s).collect()
}

fn print_and_discard(r: &Result<String, String>) {
    match r {
        Ok(v) => println!("{}", v),
        Err(e) => println!("error: {}", e),
    }
}
fn print_and_return(r: &Result<String, String>) -> Result<String, String> {
    let x = r.clone();
    print_and_discard(&r);
    x
}
#[derive(Debug, Deserialize)]
struct Settings {
    channel: String,
    server: String,
    nick: String,
    name: String,
}
fn get_settings() -> Result<Settings, config::ConfigError> {
    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("Settings"))?;
    settings.try_into()
}

fn main() -> Result<(), String> {
    let s = get_settings().unwrap();
    let split_by_channel = format!("PRIVMSG #{}", &s.channel);
    TcpStream::connect(&s.server)
        .map_err(|e| e.to_string())
        .map(|tcp_stream| BufStream::new(tcp_stream))
        .and_then(|mut stream| {
            print_and_return(&send_raw_msg_to_stream(
                &mut stream,
                &format!("NICK {}", &s.nick),
            ))
            .and_then(|_| {
                print_and_return(&send_raw_msg_to_stream(
                    &mut stream,
                    &format!("USER rrr 0 * :irc {}", &s.name),
                ))
            })
            .and_then(|_| {
                print_and_return(&send_raw_msg_to_stream(
                    &mut stream,
                    &format!("JOIN #{}", &s.channel),
                ))
            })
            .map(|_| stream)
        })
        .map(|mut stream| {
            let mut buffer = String::new();
            while let Ok(_) = stream.read_line(&mut buffer) {
                print!(">> {}", buffer);
                if buffer.starts_with("PING") {
                    print_and_discard(&send_raw_msg_to_stream(
                        &mut stream,
                        &buffer.replace("PING", "PONG").trim_end(),
                    ));
                } else {
                    let print_url_title_result: Result<Vec<Result<String, String>>, String> =
                        buffer
                            .split(&split_by_channel)
                            .nth(1)
                            .ok_or("Not a channel msg".to_owned())
                            .map(extract_links)
                            .map(|links| {
                                links
                                    .iter()
                                    .map(get_title_from_link)
                                    .map(|r| {
                                        r.and_then(|title| {
                                            send_raw_msg_to_stream(
                                                &mut stream,
                                                &as_channel_msg(&s.channel, &title),
                                            )
                                        })
                                    })
                                    .collect()
                            });
                    match print_url_title_result {
                        Ok(v) => v.iter().for_each(print_and_discard),
                        Err(e) => println!("{}", e),
                    }
                }
                buffer.clear();
            }
        })
}

fn as_channel_msg(channel: &str, msg: &str) -> String {
    format!("PRIVMSG #{} :{}", channel, msg)
}

fn send_raw_msg_to_stream<W: Write>(w: &mut W, msg: &str) -> Result<String, String> {
    let to_write = format!("{}\r\n", msg);
    w.write(to_write.as_bytes())
        .and_then(|_| w.flush())
        .and_then(|_| Ok(format!("<< {}", to_write)))
        .map_err(|e| e.to_string())
}
