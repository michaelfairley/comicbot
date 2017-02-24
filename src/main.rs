extern crate reqwest;
extern crate rss;
extern crate xml;
extern crate csv;
#[macro_use]
extern crate serde_json;

use std::{fs,io};
use std::collections::HashSet;

fn main() {
  pdl()
}


fn post_to_slack(message: String) {
  let json = json!({
    "text": message,
  });

  let client = reqwest::Client::new().unwrap();
  client.post(&std::env::var("SLACK_WEBHOOK_URL").expect("SLACK_WEBHOOK_URL must be set"))
    .json(&json)
    .send()
    .expect("Posting to Slack failed");
}

fn pdl() {
  let existing = existing_pdls();
  let pdls = get_pdls();

  let new = pdls.iter().filter(|&&(ref guid, _, _)| !existing.contains(guid)).collect::<Vec<_>>();

  let f = fs::OpenOptions::new()
    .append(true)
    .create(true)
    .open("pdl").unwrap();

  for &(ref guid, ref title, ref url) in new {
    use std::io::Write;

    post_to_slack(format!("{}\n{}", title, url));

    writeln!(&f, "{}", guid).unwrap();
    println!("{}", guid);
  }
}

fn existing_pdls() -> HashSet<String> {
  let mut set = HashSet::new();

  if let Ok(f) = fs::File::open("pdl") {
    use std::io::BufRead;

    let f = io::BufReader::new(f);
    set.extend(f.lines().map(Result::unwrap));
  }

  set
}

fn get_pdls() -> Vec<(String, String, String)> {
  let response = reqwest::get("http://feeds.feedburner.com/PoorlyDrawnLines?format=xml").expect("Bad response");

  let buf_response = io::BufReader::new(response);
  let channel = rss::Channel::read_from(buf_response).expect("Couldn't read channel");

  channel.items.into_iter().map(|item| {
    let title = item.title.unwrap().to_lowercase();
    let guid = item.guid.unwrap().value;
    let body = item.content.expect("No content").parse::<xml::Element>().expect("Couldn't parse content");
    let image = body.get_child("img", None).expect("No img");
    let image_url = image.get_attribute("src", None).expect("No src");


    (guid, title, image_url.to_string())
  }).collect()
}
