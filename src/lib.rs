use chrono::{self, Datelike};
use gtk::prelude::*;

use url::Url;

use roxmltree::{self, Node};

use reqwest::{self, header::{LAST_MODIFIED, DATE}};

pub fn curr_time() -> String {
    let now = chrono::offset::Local::now();

    format!("{}/{}/{}", now.day(), now.month(), now.year())
}

pub fn show_screen(b: &gtk::Builder, screen_name: &str) {
    let w: gtk::Window = b.get_object(screen_name).unwrap();
    w.show_all();
}

pub fn set_window_close(window: &gtk::Window) {
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
}

#[derive(Debug, Clone)]
enum WhoisError {
    ParseXML,
    ParseRegistrant,
    ParseOrganisation,
    NoOrganisation
}

fn descend_xml_tree<'a, 'b, 'c, 'd>(node: Node<'c, 'd>, child_tagname: &'b str) -> Option<Node<'c, 'd>> {
    let descendants: Vec<Node> = node.descendants().filter(|node| { node.tag_name().name() == child_tagname }).collect();

    match descendants.len() {
        1 => Some(descendants[0]),
        _ => None
    }
}

fn get_domain_owner(curr_url: Url) -> Result<String, WhoisError> {
    let _host = curr_url.host_str().unwrap_or("");

    //https://www.whoisxmlapi.com/whoisserver/WhoisService?apiKey=&domainName=google.com
    // Todo, perform web request
    let xml_res = include_str!("tmp");

    let doc = roxmltree::Document::parse(xml_res).or(Err(WhoisError::ParseXML))?;

    let registrant = descend_xml_tree(doc.root(), "registrant").ok_or(WhoisError::ParseRegistrant)?;
    let organisation = descend_xml_tree(registrant, "organization").ok_or(WhoisError::ParseOrganisation)?;

    organisation.text().map(|s| String::from(s)).ok_or(WhoisError::NoOrganisation)
}


#[derive(Debug, Clone)]
pub enum GuessError {
    UrlParse,
    RequestError,
    DecodingError
}

pub struct Guesses {
    pub author: String,
    pub year: String,
    pub page: String,
    pub site: String
}

pub fn guess_from_url(raw_url: &str) -> Result<Guesses, GuessError> {
    let url = Url::parse(raw_url).or(Err(GuessError::UrlParse))?;

    let res = reqwest::blocking::get(raw_url).or(Err(GuessError::RequestError))?;

    // Year
    let headers = res.headers();

    let last_modified = headers.get(LAST_MODIFIED);
    let date = headers.get(DATE);

    let year = last_modified.or(date).and_then(|raw_date| {
        raw_date.to_str().ok().and_then(|date_str| {
            date_str.split(' ').skip(3).next()
        })
    });
    let year = year.map(|y| String::from(y)).unwrap_or(chrono::Local::now().year().to_string());

    // Page title
    // let txt = res.text().or(Err(GuessError::DecodingError))?;


    // Site title

    // Author
    //todo implement backup using site title if whois fails
    let domain_owner = get_domain_owner(url).unwrap_or(String::from("fail"));

    Ok(Guesses {
        author: domain_owner,
        year: year,
        page: String::from("TODO"),
        site: String::from("TODO")
    })
}
