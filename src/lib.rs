pub mod tui;

use clap::{Parser};
use url::{Url};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use serde::{Serialize, Deserialize};
use select::document::Document;
use select::predicate::{Attr, Name, Predicate};
use std::io::Write;
use tui::PRZZITUI;


const CLI_HELP: &str = "A TUI to partially view/download research papers.
Search Results are taken from Google Scholar.";

#[derive(Parser)]
#[clap(version = "0.1.0", author = "lucasace", about = CLI_HELP)]
pub struct PRZZIConfig {
    /// Query to search for
    #[clap()]
    pub query: Option<String>,

    /// number of results to show
    #[clap(short='r', long="num_results", default_value_t = 10, requires = "query")]
    pub num_results: usize,

    /// download the paper via the DOI url, cannot be used when query is mentioned

    #[clap(short='d', long = "download", conflicts_with = "query")]
    pub download: Option<Url>,
    
}

pub struct PRZZIError {
    pub msg: String,
}

impl From<std::io::Error> for PRZZIError {
    fn from(err: std::io::Error) -> Self {
        PRZZIError {
            msg: err.to_string(),
        }
    }
}   

impl From<String> for PRZZIError {
    fn from(err: String) -> Self {
        PRZZIError {
            msg: err,
        }
    }
}


impl From<url::ParseError> for PRZZIError {
    fn from(err: url::ParseError) -> Self {
        PRZZIError {
            msg: err.to_string(),
        }
    }
}
impl From<reqwest::Error> for PRZZIError {
    fn from(err: reqwest::Error) -> Self {
        PRZZIError {
            msg: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for PRZZIError {
    fn from(err: serde_json::Error) -> Self {
        PRZZIError {
            msg: err.to_string(),
        }
    }
}


impl std::fmt::Debug for PRZZIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("PRZZIError: {}", self.msg).as_str())
    }
}



#[derive(Serialize)]
pub struct Query {
    pub query: Option<String>,
    pub limit : usize,
    pub fields: String,
}


#[derive(Deserialize)]
pub struct PRZZIResult {
    #[serde(rename = "DOI")]
    pub doi: Url,
    pub title: String,
    #[serde(rename = "abstract")]
    pub abs: String,
    pub year: usize,
    pub authors: Vec<String>
}

impl PRZZIResult {
    pub fn new(papers:serde_json::Value) -> Self {
        let mut doi = papers["externalIds"]["DOI"].to_string().replace("\"", "");
        doi.insert_str(0, "https://doi.org/");
        let doi = Url::parse(&doi).unwrap();
        let title = papers["title"].to_string().replace("\"", "");
        let abs = papers["abstract"].to_string().replace("\"", "");
        let year = papers["year"].to_string().parse::<usize>().unwrap();
        let authors = papers["authors"].as_array().unwrap().iter().map(|x| x["name"].to_string().replace("\"", "")).collect();
        PRZZIResult {
            doi,
            title,
            abs,
            year,
            authors,
        }
    }
}

pub struct PRZZI {
    tui: PRZZITUI,
    search_url: Url,
    download_url: Url,
    query: Option<String>,
    num_results: usize,
    download: Option<Url>,
}

impl PRZZI {
    pub fn new(config: PRZZIConfig) -> Result<Self, PRZZIError> {
        if config.query.is_none() && config.download.is_none() {
            return Err(PRZZIError {
                msg: "Either query or download must be specified".to_string(),
            });
        }
        Ok(PRZZI {
            tui: PRZZITUI::new(),
            search_url: Url::parse("https://api.semanticscholar.org/graph/v1/paper/search")?,
            download_url: Url::parse("https://sci-hub.wf/")?,
            query: config.query,
            num_results: config.num_results,
            download: config.download,
        })
    }

    pub fn run(& mut self) -> Result<(), PRZZIError> {
        if self.query.is_some() {
            enable_raw_mode().unwrap();
            let results : Vec<PRZZIResult> = self.search()?;
            self.tui.set_results(results);
            self.tui.print_results();
            loop {
                if let Ok(false) = self.tui.handle_input() {
                    disable_raw_mode().unwrap();
                    break;
                }
            }
        } else {
            if let Err(_e) = self.download() {
                return Err(PRZZIError {
                    msg: "Download failed! :-( \n Please Check the DOI or raise an issue on github".to_string(),
                });
            }
        }
       Ok(())
    }

    pub fn search(&self) -> Result<Vec<PRZZIResult>, PRZZIError> {
        let query = Query {
            query: self.query.clone(),  
            limit: self.num_results,
            fields: "title,abstract,authors,year,url,externalIds".to_string(),
        };

        let client = reqwest::blocking::Client::new();
        let res = client.get(self.search_url.clone())
            .query(&query)
            .send()?;
        let response: serde_json::Value = serde_json::from_str(res.text()?.as_str())?;
        let papers : Vec<serde_json::Value> = serde_json::from_value(response["data"].clone())?;
        let results : Vec<PRZZIResult> = papers.iter().map(|x| PRZZIResult::new(x.clone())).collect();
        Ok(results)
    }

    pub fn download(&self) -> Result<(), PRZZIError> {
        let client = reqwest::blocking::Client::new();
        let doi_url = self.download_url.join(self.download.as_ref().unwrap().path())?;
        let res = client.get(doi_url)
            .send()?;
        let document = Document::from(res.text()?.as_str());
        // Need better way to do this
        let link = document.find(Attr("id", "buttons").descendant(Name("button")));
        if link.count() == 0 {
            return Err(PRZZIError {
                msg: "Download failed! :-( \n Please Check the DOI or raise an issue on github".to_string(),
            });
        }
        println!("Downloading...");
        let link = document.find(Attr("id", "buttons").descendant(Name("button"))).next().unwrap();
        let link = link.attr("onclick").unwrap().replace("location.href='", "").replace("'", "");
        let down_url = self.download_url.join(&link)?;
        let res = client.get(down_url)
            .send()?;
        if res.headers().get("Content-Type").unwrap() != "application/pdf" {
            return Err(PRZZIError {
                msg: "Download failed! :-( \n Please Check the DOI or raise an issue on github".to_string(),
            });
        }
        // get filename
        let filename = res.url().path_segments().unwrap().last().unwrap();
        // save file
        let mut file = std::fs::File::create(filename)?;
        file.write_all(res.bytes()?.as_ref())?;
        Ok(())
    }
}