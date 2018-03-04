// Exhaustively search iTunes search suggestions.
//
// Search suggestions typically work by matching a prefix
// of your query. This makes it possible to build up a
// tree of results, expanding where the search hints did
// not fill up an entire page.
//
// https://search.itunes.apple.com/WebObjects/MZSearchHints.woa/wa/hints?media=music&term=foo

extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate xml;

use std::io::{Cursor};

use futures::{Future, Stream};
use hyper::Client;
use tokio_core::reactor::Core;
use xml::reader::{EventReader, XmlEvent};

// TODO: figure out why HTTPS didn't work here.
static SEARCH_PAGE: &str = "http://search.itunes.apple.com/WebObjects/MZSearchHints.woa/wa/hints?media=music&term=";

fn main() {
    let mut core = Core::new().expect("Failed to create reactor core.");
    let client = Client::new(&core.handle());

    let xml = query_result_xml(&mut core, &client, "poop").expect("uhoh");
    println!("foo: {:?}", parse_result_xml(xml).expect("oh no!");
}

fn parse_result_xml(raw: String) -> Result<Vec<String>, xml::reader::Error> {
    let mut results = Vec::new();
    let mut last_key = String::new();
    let mut inside_key = false;
    let mut current_value = String::new();
    let mut inside_value = false;
    let reader = EventReader::new(Cursor::new(raw.into_bytes()));
    for event in reader {
        match event? {
            XmlEvent::StartElement{name, ..} => {
                match name.local_name.as_ref() {
                    "key" => {
                        inside_key = true;
                        inside_value = false;
                        last_key.clear();
                    },
                    "string" => {
                        inside_key = false;
                        inside_value = true;
                        current_value.clear();
                    },
                    _ => ()
                }
            },
            XmlEvent::Characters(data) => {
                if inside_key {
                    last_key.push_str(&data);
                } else if inside_value {
                    current_value.push_str(&data);
                }
            },
            XmlEvent::EndElement{..} => {
                if inside_value && last_key == "term" {
                    results.push(current_value.clone());
                }
                inside_key = false;
                inside_value = false;
            },
            _ => ()
        }
    }
    Ok(results)
}

fn query_result_xml<T: hyper::client::Connect>(core: &mut Core, client: &Client<T>, query: &str) ->
    Result<String, hyper::error::Error>
{
    let url = format!("{}{}", SEARCH_PAGE, escape_query(query));
    // The parse() method + type inference triggers
    // hyper::Uri::from_str().
    let res_future = client.get(url.parse()?).and_then(|res| {
        // res is a hyper::Response
        // res.body() is a Stream<Item=hyper::Chunk, Error=>
        // res.body().concat2() makes a unified hyper::Chunk
        res.body().concat2().map(|x| {
            String::from_utf8(x.into_iter().collect()).map_err(From::from)
        })
    });
    core.run(res_future)?
}

fn escape_query(query: &str) -> String {
    let mut res = String::new();
    for ch in query.bytes() {
        res.push_str(&format!("%{:02x}", ch));
    }
    res
}
