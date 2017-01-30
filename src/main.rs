extern crate reqwest;
extern crate rss;
extern crate xml;
extern crate csv;
extern crate serde_json;

use std::{fs,io};
use std::collections::HashSet;

fn main() {
  pdl()
}


fn post_to_slack(message: String) {
  // TODO: replace with json! once reqwest upgrades its serde dependency
  let json = serde_json::builder::ObjectBuilder::new()
    .insert("text", message)
    .build();

  let client = reqwest::Client::new().unwrap();
  client.post(&std::env::var("SLACK_WEBHOOK_URL").expect("SLACK_WEBHOOK_URL must be set"))
    .json(&json)
    .send()
    .expect("Posting to Slack failed");
}

fn pdl() {
  let existing = existing_pdls();
  let pdls = get_pdls();

  let new = pdls.iter().filter(|p| !existing.contains(p)).collect::<Vec<_>>();

  let f = fs::OpenOptions::new()
    .append(true)
    .create(true)
    .open("pdl").unwrap();

  let mut csv = csv::Writer::from_writer(f)
    .delimiter(b':');

  for comic in new {
    post_to_slack(format!("{}\n{}", comic.0, comic.1));
    csv.encode(comic).unwrap();
  }
}

fn existing_pdls() -> HashSet<(String, String)> {
  let mut set = HashSet::new();

  if let Ok(f) = fs::File::open("pdl") {
    let mut csv = csv::Reader::from_reader(f)
      .has_headers(false)
      .delimiter(b':');

    for row in csv.decode() {
      set.insert(row.unwrap());
    }
  }

  set
}

fn get_pdls() -> Vec<(String, String)> {
  let response = reqwest::get("http://feeds.feedburner.com/PoorlyDrawnLines?format=xml").expect("Bad response");

  let buf_response = io::BufReader::new(response);
  let channel = rss::Channel::read_from(buf_response).expect("Couldn't read channel");

  channel.items.into_iter().map(|item| {
    let title = item.title.unwrap().to_lowercase();
    let body = item.content.expect("No content").parse::<xml::Element>().expect("Couldn't parse content");
    let image = body.get_child("img", None).expect("No img");
    let image_url = image.get_attribute("src", None).expect("No src");


    (title, image_url.to_string())
  }).collect()
}
