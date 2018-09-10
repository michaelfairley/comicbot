extern crate reqwest;
extern crate rss;
extern crate xml;
extern crate csv;
#[macro_use]
extern crate serde_json;

use std::{fs,io};
use std::collections::HashSet;

fn main() {
  go::<PDL>();
  go::<WCN>();
  go::<JLO>();
  go::<SMBC>();
}

fn go<C: Comic>() {
  let existing = existing::<C>();
  let mut current = C::get_current();
  current.reverse();

  let new = current.iter().filter(|i| !existing.contains(&i.guid)).collect::<Vec<_>>();

  let f = fs::OpenOptions::new()
    .append(true)
    .create(true)
    .open(C::EXISTING_FILE_NAME)
    .unwrap();

  for instance in new {
    use std::io::Write;

    let mut message = String::new();

    if let Some(title) = &instance.title {
      message.push_str("<");
      message.push_str(&instance.link);
      message.push_str("|");
      message.push_str(title);
      message.push_str(">");
    } else {
      message.push_str(&instance.link);
    }
    message.push_str("\n");
    message.push_str(&instance.image_url);
    if let Some(alt_text) = &instance.alt_text {
      message.push_str("\n");
      message.push_str(alt_text);
    }

    post_to_slack::<C>(&message);

    writeln!(&f, "{}", instance.guid).unwrap();
    println!("{}", instance.guid);
  }
}

fn existing<C: Comic>() -> HashSet<String> {
  let mut set = HashSet::new();

  if let Ok(f) = fs::File::open(C::EXISTING_FILE_NAME) {
    use std::io::BufRead;

    let f = io::BufReader::new(f);
    set.extend(f.lines().map(Result::unwrap));
  }

  set
}



struct Instance {
  guid: String,
  title: Option<String>,
  alt_text: Option<String>,
  image_url: String,
  link: String,
}

trait Comic {
  const USERNAME: &'static str;
  const ICON_URL: &'static str;
  const EXISTING_FILE_NAME: &'static str;

  fn get_current() -> Vec<Instance>;
}

struct PDL;
impl Comic for PDL {
  const USERNAME: &'static str = "poorlydrawnlines";
  const ICON_URL: &'static str = "https://pbs.twimg.com/profile_images/785967542380617728/Iy0lhx2T.jpg";
  const EXISTING_FILE_NAME: &'static str = "pdl";

  fn get_current() -> Vec<Instance> {
    let response = reqwest::get("http://feeds.feedburner.com/PoorlyDrawnLines?format=xml").expect("Bad response");

    let buf_response = io::BufReader::new(response);
    let channel = rss::Channel::read_from(buf_response).expect("Couldn't read channel");

    channel.items.into_iter().filter_map(|item| {
      let title = item.title.unwrap().to_lowercase();
      let guid = item.guid.unwrap().value;
      let body = item.content.expect("No content").parse::<xml::Element>().expect("Couldn't parse content");

      let link = item.extensions["feedburner"]["origLink"][0].value.as_ref().unwrap().to_string();

      if let Some(image) = body.get_child("img", None) {
        let image_url = image.get_attribute("src", None).expect("No src");
        Some(Instance{
          guid,
          link,
          title: Some(title),
          alt_text: None,
          image_url: image_url.to_string(),
        })
      } else {
        None
      }
    }).collect()
  }
}

struct WCN;
impl Comic for WCN {
  const USERNAME: &'static str = "webcomicname";
  const ICON_URL: &'static str = "http://68.media.tumblr.com/avatar_44d7cb4c7049_128.png";
  const EXISTING_FILE_NAME: &'static str = "wcn";


  fn get_current() -> Vec<Instance> {
    let response = reqwest::get("http://webcomicname.com/rss").expect("Bad response");

    let buf_response = io::BufReader::new(response);
    let channel = rss::Channel::read_from(buf_response).expect("Couldn't read channel");

    channel.items.into_iter().filter_map(|item| {
      let guid = item.guid.unwrap().value;
      let image = item.description.expect("No description").parse::<xml::Element>().expect("Couldn't parse description");
      let link = item.link.unwrap();

      let img = if image.name == "img" {
        Some(&image)
      } else if let Some(img) = image.get_child("img", None) {
        Some(img)
      } else { None };

      img.map(|img| {
        let image_url = img.get_attribute("src", None).expect("No src");
        let image_url = image_url.replace("_500.png", "_1280.png");
        Instance{
          guid,
          link,
          title: None,
          alt_text: None,
          image_url: image_url.to_string(),
        }
      })
    }).collect()
  }
}

struct JLO;
impl Comic for JLO {
  const USERNAME: &'static str = "jakelikesonions";
  const ICON_URL: &'static str = "https://pbs.twimg.com/profile_images/915262151270572032/FW9GE1_O_400x400.jpg";
  const EXISTING_FILE_NAME: &'static str = "jlo";


  fn get_current() -> Vec<Instance> {
    let response = reqwest::get("http://jakelikesonions.com/rss").expect("Bad response");

    let buf_response = io::BufReader::new(response);
    let channel = rss::Channel::read_from(buf_response).expect("Couldn't read channel");

    channel.items.into_iter().filter_map(|item| {
      let guid = item.guid.unwrap().value;
      let image = item.description.expect("No description").parse::<xml::Element>().expect("Couldn't parse description");
      let link = item.link.unwrap();

      let img = if image.name == "img" {
        Some(&image)
      } else if let Some(img) = image.get_child("img", None) {
        Some(img)
      } else { None };

      img.map(|img| {
        let image_url = img.get_attribute("src", None).expect("No src")
          .replace("_540.jpg", "_1280.jpg")
          .replace("_500.jpg", "_1280.jpg");
        Instance{
          guid,
          link,
          title: None,
          alt_text: None,
          image_url: image_url.to_string(),
        }
      })
    }).collect()
  }
}

struct SMBC;
impl Comic for SMBC {
  const USERNAME: &'static str = "saturdaymorningbreakfastcereal";
  const ICON_URL: &'static str = "https://pbs.twimg.com/profile_images/1733661436/41813_104538479599168_2496_n_400x400.jpg";
  const EXISTING_FILE_NAME: &'static str = "smbc";

  fn get_current() -> Vec<Instance> {
    let response = reqwest::get("https://www.smbc-comics.com/comic/rss").expect("Bad response");

    let buf_response = io::BufReader::new(response);
    let channel = rss::Channel::read_from(buf_response).expect("Couldn't read channel");

    channel.items.into_iter().filter_map(|item| {
      let guid = item.guid.unwrap().value;

      let title = item.title.unwrap().trim_left_matches("Saturday Morning Breakfast Cereal - ").to_string();
      let link = item.link.unwrap();


      let mut p = xml::Parser::new();
      let mut e = xml::ElementBuilder::new();
      p.feed_str(&item.description.expect("No description"));
      let nodes = p.filter_map(|x| e.handle_event(x)).map(Result::unwrap).collect::<Vec<_>>();

      if let Some(image) = nodes.get(0).and_then(|n| n.get_child("img", None)) {
        let image_url = image.get_attribute("src", None).expect("No src")
          .replace(" ", "%20");

        let text = nodes.get(1).map(|n| n.content_str().trim_left_matches("Hovertext:").to_string());

        Some(Instance{
          guid,
          link,
          title: Some(title),
          alt_text: text,
          image_url: image_url.to_string(),
        })
      } else {
        None
      }
    }).collect()
  }
}


fn post_to_slack<C: Comic>(message: &str) {
  let json = json!({
    "username": C::USERNAME,
    "icon_url": C::ICON_URL,
    "text": message,
    "unfurl_links": false,
  });

  if let Ok(slack_webhook_url) = std::env::var("SLACK_WEBHOOK_URL") {
    let client = reqwest::Client::new().unwrap();
    client.post(&slack_webhook_url)
      .json(&json)
      .send()
      .expect("Posting to Slack failed");
  } else {
    println!("{}", json);
  }
}
