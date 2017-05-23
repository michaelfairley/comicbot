extern crate reqwest;
extern crate rss;
extern crate xml;
extern crate csv;
#[macro_use]
extern crate serde_json;

use std::{fs,io};
use std::collections::HashSet;

#[derive(Copy,Clone)]
enum Comic {
  PoorlyDrawnLines,
  WebcomicName,
}

impl Comic {
  fn username(self) -> &'static str {
    use Comic::*;
    match self {
      PoorlyDrawnLines => "poorlydrawnlines",
      WebcomicName => "webcomicname",
    }
  }

  fn icon_url(self) -> &'static str {
    use Comic::*;
    match self {
      PoorlyDrawnLines => "https://pbs.twimg.com/profile_images/785967542380617728/Iy0lhx2T.jpg",
      WebcomicName => "http://68.media.tumblr.com/avatar_44d7cb4c7049_128.png",
    }
  }
}

fn main() {
  pdl();
  webcomicname();
}


fn post_to_slack(comic: Comic, message: String) {
  let json = json!({
    "username": comic.username(),
    "icon_url": comic.icon_url(),
    "text": message,
  });

  let client = reqwest::Client::new().unwrap();
  client.post(&std::env::var("SLACK_WEBHOOK_URL").expect("SLACK_WEBHOOK_URL must be set"))
    .json(&json)
    .send()
    .expect("Posting to Slack failed");
}

fn webcomicname() {
  let existing = existing_wcns();
  let current = get_wcns();

  let new = current.iter().filter(|&&(ref guid, _)| !existing.contains(guid)).collect::<Vec<_>>();

  let f = fs::OpenOptions::new()
    .append(true)
    .create(true)
    .open("wcn").unwrap();

  for &(ref guid, ref url) in new {
    use std::io::Write;

    post_to_slack(Comic::WebcomicName, format!("{}", url));

    writeln!(&f, "{}", guid).unwrap();
    println!("{}", guid);
  }
}

fn existing_wcns() -> HashSet<String> {
  let mut set = HashSet::new();

  if let Ok(f) = fs::File::open("wcn") {
    use std::io::BufRead;

    let f = io::BufReader::new(f);
    set.extend(f.lines().map(Result::unwrap));
  }

  set
}

fn get_wcns() -> Vec<(String, String)> {
  let response = reqwest::get("http://webcomicname.com/rss").expect("Bad response");

  let buf_response = io::BufReader::new(response);
  let channel = rss::Channel::read_from(buf_response).expect("Couldn't read channel");

  channel.items.into_iter().filter_map(|item| {
    let guid = item.guid.unwrap().value;
    let image = item.description.expect("No description").parse::<xml::Element>().expect("Couldn't parse description");

    if image.name == "img" {
      let image_url = image.get_attribute("src", None).expect("No src");
      let image_url = image_url.replace("_500.png", "_1280.png");
      Some((guid, image_url.to_string()))
    } else {
      None
    }
  }).collect()
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

    post_to_slack(Comic::PoorlyDrawnLines, format!("{}\n{}", title, url));

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

  channel.items.into_iter().filter_map(|item| {
    let title = item.title.unwrap().to_lowercase();
    let guid = item.guid.unwrap().value;
    let body = item.content.expect("No content").parse::<xml::Element>().expect("Couldn't parse content");

    if let Some(image) = body.get_child("img", None) {
      let image_url = image.get_attribute("src", None).expect("No src");
      Some((guid, title, image_url.to_string()))
    } else {
      None
    }
  }).collect()
}
