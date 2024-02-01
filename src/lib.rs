#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use reqwest;

use reqwest::StatusCode;
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};
use time::{serde::rfc3339, Date, Month, OffsetDateTime};
use url::Url;

const BASE_URL: &str = "https://www.tagesschau.de/api2u/news";

/// The german federal states.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Region {
    #[allow(missing_docs)]
    BadenWürttemberg = 1,
    #[allow(missing_docs)]
    Bayern = 2,
    #[allow(missing_docs)]
    Berlin = 3,
    #[allow(missing_docs)]
    Brandenburg = 4,
    #[allow(missing_docs)]
    Bremen = 5,
    #[allow(missing_docs)]
    Hamburg = 6,
    #[allow(missing_docs)]
    Hessen = 7,
    #[allow(missing_docs)]
    MecklenburgVorpommern = 8,
    #[allow(missing_docs)]
    Niedersachsen = 9,
    #[allow(missing_docs)]
    NordrheinWestfalen = 10,
    #[allow(missing_docs)]
    RheinlandPfalz = 11,
    #[allow(missing_docs)]
    Saarland = 12,
    #[allow(missing_docs)]
    Sachsen = 13,
    #[allow(missing_docs)]
    SachsenAnhalt = 14,
    #[allow(missing_docs)]
    SchleswigHolstein = 15,
    #[allow(missing_docs)]
    Thüringen = 16,
}

/// Different news categorys
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Ressort {
    /// With this option, the ressort will not be specified and all results will be shown
    None,
    /// Only news from Germany
    Inland,
    /// Only news from outside of Germany
    Ausland,
    /// Economic news
    Wirtschaft,
    /// Sports news
    Sport,
    /// Different kinds of videos
    Video,
    /// Investigative journalism
    Investigativ,
    // Faktenfinder,
}

impl Display for Ressort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ressort::None => f.write_str(""),
            Ressort::Inland => f.write_str("inland"),
            Ressort::Ausland => f.write_str("ausland"),
            Ressort::Wirtschaft => f.write_str("wirtschaft"),
            Ressort::Sport => f.write_str("sport"),
            Ressort::Video => f.write_str("video"),
            Ressort::Investigativ => f.write_str("investigativ"),
            // Ressort::Faktenfinder => f.write_str("faktenfinder"),
        }
    }
}

//TODO - Add support for multiple Ressorts

pub enum Timeframe {
    Now,
    Date(TDate),
    DateRange(DateRange),
}

#[derive(Clone, Copy)]
pub struct TDate {
    day: u8,
    month: Month,
    year: i32,
}

impl TDate {
    pub fn from_calendar_date(
        year: i32,
        month: Month,
        day: u8,
    ) -> Result<Self, TagesschauApiError> {
        let date = Date::from_calendar_date(year, month, day)?;
        Ok(TDate::from_time_date(date))
    }

    pub fn from_time_date(d: Date) -> Self {
        TDate {
            day: d.day(),
            month: d.month(),
            year: d.year(),
        }
    }

    pub fn format(&self) -> String {
        format!(
            "{}{}{}",
            self.year % 100,
            format!("{:0>2}", self.month as u8),
            format!("{:0>2}", self.day)
        )
    }
}

#[derive(Clone)]
pub struct DateRange {
    dates: Vec<TDate>,
}

impl DateRange {
    pub fn new(start: TDate, end: TDate) -> Result<Self, TagesschauApiError> {
        let mut dates: Vec<TDate> = Vec::new();

        let mut s = Date::from_calendar_date(start.year, start.month, start.day)?;

        let e = Date::from_calendar_date(end.year, end.month, end.day)?;

        while s <= e {
            dates.push(TDate::from_time_date(s));
            s = s.next_day().unwrap();
        }

        Ok(Self { dates })
    }

    pub fn from_dates(dates: Vec<TDate>) -> Self {
        Self { dates }
    }
}

pub struct TagesschauAPI {
    ressort: Ressort,
    regions: HashSet<Region>,
    timeframe: Timeframe,
}

impl TagesschauAPI {
    pub fn new() -> Self {
        Self {
            ressort: Ressort::None,
            regions: HashSet::new(),
            timeframe: Timeframe::Now,
        }
    }

    pub fn ressort(&mut self, res: Ressort) -> &mut TagesschauAPI {
        self.ressort = res;
        self
    }

    pub fn regions(&mut self, reg: HashSet<Region>) -> &mut TagesschauAPI {
        self.regions = reg;
        self
    }

    pub fn timeframe(&mut self, timeframe: Timeframe) -> &mut TagesschauAPI {
        self.timeframe = timeframe;
        self
    }

    fn prepare_url(&self, date: TDate) -> Result<String, TagesschauApiError> {
        // TODO - Support multiple ressorts
        let mut url = Url::parse(BASE_URL)?;

        url.query_pairs_mut().append_pair("date", &date.format());

        if !self.regions.is_empty() {
            let mut r = String::new();
            for region in &self.regions {
                r.push_str(&format!("{},", *region as u8));
            }

            url.query_pairs_mut().append_pair("regions", &r);
        }

        if self.ressort != Ressort::None {
            url.query_pairs_mut()
                .append_pair("ressort", &self.ressort.to_string());
        }

        Ok(url.to_string())
    }

    async fn fetch(&self, date: TDate) -> Result<Articles, TagesschauApiError> {
        let url = self.prepare_url(date)?;

        println!("{}", url);

        let response = reqwest::get(url)
            .await
            .map_err(|e| TagesschauApiError::BadRequest(e))?;

        let text = match response.status() {
            StatusCode::OK => response
                .text()
                .await
                .map_err(|e| TagesschauApiError::ParsingError(e))?,
            _ => Err(TagesschauApiError::InvalidResponse(
                response.status().as_u16(),
            ))?,
        };

        let articles: Articles = serde_json::from_str(&text)?;

        Ok(articles)
    }

    pub async fn get_all_articles(&self) -> Result<Vec<Content>, TagesschauApiError> {
        let dates: Vec<TDate> = match &self.timeframe {
            Timeframe::Now => {
                let now = OffsetDateTime::now_local()?;

                vec![TDate::from_time_date(now.date())]
            }
            Timeframe::Date(date) => {
                vec![*date]
            }
            Timeframe::DateRange(date_range) => date_range.dates.clone(),
        };

        let mut content: Vec<Content> = Vec::new();

        for date in dates {
            let mut art = self.fetch(date).await?;

            content.append(&mut art.news)
        }

        Ok(content)
    }

    pub async fn get_text_articles(&self) -> Result<Vec<Text>, TagesschauApiError> {
        let articles = self.get_all_articles().await;

        match articles {
            Ok(mut a) => {
                a.retain(|x| x.is_text());
                let mut t: Vec<Text> = Vec::new();

                for content in a {
                    t.push(content.to_text().unwrap())
                }

                Ok(t)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn get_video_articles(&self) -> Result<Vec<Video>, TagesschauApiError> {
        let articles = self.get_all_articles().await;

        match articles {
            Ok(mut a) => {
                a.retain(|x| x.is_video());
                let mut t: Vec<Video> = Vec::new();

                for content in a {
                    t.push(content.to_video().unwrap())
                }

                Ok(t)
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Articles {
    pub news: Vec<Content>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Content {
    Text(Text),
    Video(Video),
}

impl PartialEq for Content {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Content {
    pub fn is_text(&self) -> bool {
        match self {
            Content::Text(_) => true,
            Content::Video(_) => false,
        }
    }

    pub fn is_video(&self) -> bool {
        match self {
            Content::Text(_) => false,
            Content::Video(_) => true,
        }
    }

    pub fn to_text(self) -> Result<Text, TagesschauApiError> {
        match self {
            Content::Text(text) => Ok(text),
            Content::Video(_) => Err(TagesschauApiError::ConversionError),
        }
    }

    pub fn to_video(self) -> Result<Video, TagesschauApiError> {
        match self {
            Content::Video(video) => Ok(video),
            Content::Text(_) => Err(TagesschauApiError::ConversionError),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Text {
    pub title: String,
    #[serde(with = "rfc3339")]
    pub date: OffsetDateTime,
    #[serde(rename(deserialize = "detailsweb"))]
    pub url: String,
    pub tags: Vec<Tag>,
    pub ressort: Option<String>,
    #[serde(rename(deserialize = "type"))]
    pub kind: String,
    #[serde(rename(deserialize = "breakingNews"))]
    pub breaking_news: bool,
    #[serde(rename(deserialize = "teaserImage"))]
    pub image: Images,
}

#[derive(Deserialize, Debug)]
pub struct Video {
    pub title: String,
    #[serde(with = "rfc3339")]
    pub date: OffsetDateTime,
    pub streams: HashMap<String, String>,
    pub tags: Vec<Tag>,
    pub ressort: Option<String>,
    #[serde(rename(deserialize = "type"))]
    pub kind: String,
    #[serde(rename(deserialize = "breakingNews"))]
    pub breaking_news: bool,
    #[serde(rename(deserialize = "teaserImage"))]
    pub image: Images,
}

#[derive(Deserialize, Debug)]
pub struct Tag {
    pub tag: String,
}

#[derive(Deserialize, Debug)]
pub struct Images {
    pub title: Option<String>,
    pub copyright: Option<String>,
    pub alttext: String,
    #[serde(rename(deserialize = "imageVariants"))]
    pub image_variants: HashMap<String, String>,
    #[serde(rename(deserialize = "type"))]
    pub kind: String,
}

#[derive(thiserror::Error, Debug)]
pub enum TagesschauApiError {
    #[error("Fetching articles failed")]
    BadRequest(reqwest::Error),
    #[error("Failed to parse response")]
    ParsingError(reqwest::Error),
    #[error("Invalid Response: HTTP Response Code {0}")]
    InvalidResponse(u16),
    #[error("Failed to deserialize response")]
    DeserializationError(#[from] serde_json::Error),
    #[error("Tried to extract wrong type")]
    ConversionError,
    #[error("Unable to retrieve current date")]
    DateError(#[from] time::error::IndeterminateOffset),
    // DateRangeError,
    #[error("Unable parse date")]
    DateParsingError(#[from] time::error::ComponentRange),
    #[error("Url parsing failed")]
    UrlParsing(#[from] url::ParseError),
}
