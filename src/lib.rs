use chrono::{self, Datelike};
use gtk::prelude::*;

use url::Url;

use roxmltree;

use reqwest::{self, header::{LAST_MODIFIED, DATE}};
use html_parser::{Dom, Element, Node};

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
enum GetOwnerError {
    QueryWhois,
    ReadWhoisText,
    ParseXML,
    ParseRegistrant,
    ParseOrganisation,
    NoOrganisation
}

fn descend_xml_tree<'a, 'b, 'c, 'd>(node: roxmltree::Node<'c, 'd>, child_tagname: &'b str) -> Option<roxmltree::Node<'c, 'd>> {
    let descendants: Vec<roxmltree::Node> = node.descendants().filter(|node| { node.tag_name().name() == child_tagname }).collect();

    match descendants.len() {
        1 => Some(descendants[0]),
        _ => None
    }
}

fn get_domain_owner(curr_url: &Url) -> Result<String, GetOwnerError> {
    let host = curr_url.host_str().unwrap_or("");

    let query_url = format!("https://www.whoisxmlapi.com/whoisserver/WhoisService?apiKey={}&domainName={}", include_str!("whoisapi.key"), host);

    let res = reqwest::blocking::get(&query_url).or(Err(GetOwnerError::QueryWhois))?;
    let xml_data = res.text().or(Err(GetOwnerError::ReadWhoisText))?;

    let doc = roxmltree::Document::parse(&xml_data).or(Err(GetOwnerError::ParseXML))?;

    let registrant = descend_xml_tree(doc.root(), "registrant").ok_or(GetOwnerError::ParseRegistrant)?;
    let organisation = descend_xml_tree(registrant, "organization").ok_or(GetOwnerError::ParseOrganisation)?;

    organisation.text().map(|s| String::from(s)).ok_or(GetOwnerError::NoOrganisation)
}


#[derive(Debug, Clone)]
pub enum GuessError {
    UrlParse,
    Request,
    Decoding,
    DomParsing,
    NoDomain
}

pub struct Guesses {
    pub author: String,
    pub year: String,
    pub page: String,
    pub site: String
}

fn search_for_title_element(el: &Element) -> Option<String> {
    if el.name.to_lowercase() == "title" {
        let title_txt = el.children.iter().filter_map(|c| {
            if let Node::Text(txt) = c {
                Some(txt)
            } else {
                None
            }
        }).map(|s| s.as_str());
        let title_txt = title_txt.collect::<Vec<&str>>().join(" ");
        Some(title_txt)
    } else {
        el.children.iter().filter_map(|node| {
            if let Node::Element(el) = node {
                search_for_title_element(el)
            } else {
                None
            }
        }).next()
    }
}

fn search_for_title(dom: Dom) -> Option<String> {
    dom.children.iter().filter_map(|node| {
        if let Node::Element(el) = node {
            search_for_title_element(el)
        } else {
            None
        }
    }).next()
}

fn upper_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

pub fn guess_from_url(raw_url: &str) -> Result<Guesses, GuessError> {
    let url = Url::parse(raw_url).or(Err(GuessError::UrlParse))?;

    let res = reqwest::blocking::get(raw_url).or(Err(GuessError::Request))?;

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
    let txt = res.text().or(Err(GuessError::Decoding))?;
    let dom = Dom::parse(&txt).or(Err(GuessError::DomParsing))?;
    let page_title = search_for_title(dom).unwrap_or(String::from("unknown title"));

    // Site title
    let domain = url.domain().ok_or(GuessError::NoDomain)?;
    let site_title = domain.split('.').filter(|domain_part| {
        !include_str!("subdomains.txt").split('\n').collect::<Vec<_>>().contains(domain_part)
    }).next().unwrap_or("");
    let site_title = upper_first_letter(site_title);

    // Author
    let domain_owner = get_domain_owner(&url).unwrap_or(site_title.clone());

    Ok(Guesses {
        author: domain_owner,
        year: year,
        page: page_title,
        site: site_title
    })
}
