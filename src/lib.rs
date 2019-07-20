#[macro_use]
extern crate clap;

use colored::{ColoredString, Colorize};
use reqwest::r#async::Response;
use reqwest::{Url, StatusCode};

use select::document::Document;
use select::predicate::Name;
use std::collections::HashSet;
use std::fmt::Display;

use clap::{App, Arg};
pub fn print_error<T: Display>(x: T) {
    let formatted_str = format!("{}", x).bold_red();
    println!("{}", formatted_str);
}
fn is_valid_status_code(x:StatusCode)->bool{
    x.is_success()|x.is_redirection()
}
pub fn print_response(x: Response) {
    if is_valid_status_code(x.status()){

            let formatted_str =
                format!("{} is valid ({})", x.url().as_str(), x.status().as_str()).bold_green();
            println!("{}", formatted_str);
        }
        else {
            let formatted_str =
                format!("{} failed ({})", x.url().as_str(), x.status().as_str()).bold_red();
            println!("{}", formatted_str);
        }

}
pub trait ColorsExt {
    fn bold_red(&self) -> ColoredString;
    fn bold_green(&self) -> ColoredString;
}
impl ColorsExt for str {
    fn bold_red(self: &str) -> ColoredString {
        self.bold().red()
    }
    fn bold_green(self: &str) -> ColoredString {
        self.bold().green()
    }
}
pub fn make_app<'a, 'b>() -> App<'a, 'b> {
    App::new("Rusty Links")
        .version(crate_version!())
        .author("Jose Luis Ricon <jose@ricon.xyz>")
        .about("Finds dead links in websites")
        .arg(
            Arg::with_name("INPUT")
                .help("URL to check links for (e.g. http://www.google.com)")
                .index(1),
        )
}
#[derive(Debug)]
pub enum RustyLinksError {
    MalformedUrl,
    RequestError,
}
fn add_http(url_string: &str) -> String {
    if !url_string.starts_with("http://") {
        ["http://", url_string].concat()
    } else {
        url_string.to_owned()
    }
}
pub fn get_links_for_website(url_string: String) -> Result<HashSet<String>, RustyLinksError> {
    let links = Url::parse(&add_http(&url_string)).map(|url| {
        reqwest::get(url)
            .map(|doc| if is_valid_status_code(doc.status()) {
                Document::from_read(doc)
                    .unwrap()
                    .find(Name("a"))
                    .filter_map(|n| n.attr("href"))
                    .map(|x| {
                        if x.starts_with("/") {
                            Option::Some([&url_string, &x[1..]].concat())
                        } else if x.starts_with("http") {
                            Option::Some(x.to_owned())
                        } else {
                            Option::None
                        }
                    })
                    .filter(|elem| elem.is_some())
                    .map(|elem| match elem {
                        Some(e) => e,
                        _ => panic!("This can't happen"),
                    })
                    .collect()}
                else {
                    let err = format!("Could not reach website {}: {}", url_string, doc.status());
                    print_error(err);
                    HashSet::new()

            })
            .map_err(|e| println!("{:?}", e))
    });

    match links {
        Ok(e) => match e {
            Ok(e) => Ok(e),
            Err(_) => Err(RustyLinksError::RequestError),
        },
        Err(e) => {
            println!("{:?}", e);
            Err(RustyLinksError::MalformedUrl)
        }
    }
}
