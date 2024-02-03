#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use reqwest;

use reqwest::StatusCode;
use serde::{
    de::{self, Visitor},
    Deserialize,
};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fmt::{self, Display},
};
use time::{serde::rfc3339, Date, OffsetDateTime};
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

/// Months of the year.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Month {
    #[allow(missing_docs)]
    January = 1,
    #[allow(missing_docs)]
    February = 2,
    #[allow(missing_docs)]
    March = 3,
    #[allow(missing_docs)]
    April = 4,
    #[allow(missing_docs)]
    May = 5,
    #[allow(missing_docs)]
    June = 6,
    #[allow(missing_docs)]
    July = 7,
    #[allow(missing_docs)]
    August = 8,
    #[allow(missing_docs)]
    September = 9,
    #[allow(missing_docs)]
    October = 10,
    #[allow(missing_docs)]
    November = 11,
    #[allow(missing_docs)]
    December = 12,
}

impl Month {
    fn to_time_month(&self) -> time::Month {
        match self {
            Month::January => time::Month::January,
            Month::February => time::Month::February,
            Month::March => time::Month::March,
            Month::April => time::Month::April,
            Month::May => time::Month::May,
            Month::June => time::Month::June,
            Month::July => time::Month::July,
            Month::August => time::Month::August,
            Month::September => time::Month::September,
            Month::October => time::Month::October,
            Month::November => time::Month::November,
            Month::December => time::Month::December,
        }
    }

    fn from_time_month(m: time::Month) -> Self {
        match m {
            time::Month::January => Month::January,
            time::Month::February => Month::February,
            time::Month::March => Month::March,
            time::Month::April => Month::April,
            time::Month::May => Month::May,
            time::Month::June => Month::June,
            time::Month::July => Month::July,
            time::Month::August => Month::August,
            time::Month::September => Month::September,
            time::Month::October => Month::October,
            time::Month::November => Month::November,
            time::Month::December => Month::December,
        }
    }
}

struct RessortVisitor;

impl<'de> Visitor<'de> for RessortVisitor {
    type Value = Ressort;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string with one of the following values: inland, ausland, wirtschaft, sport, video, investigativ")
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value.as_str() {
            "" => Ok(Ressort::None),
            "inland" => Ok(Ressort::Inland),
            "ausland" => Ok(Ressort::Ausland),
            "wirtschaft" => Ok(Ressort::Wirtschaft),
            "sport" => Ok(Ressort::Sport),
            "video" => Ok(Ressort::Video),
            "investigativ" => Ok(Ressort::Investigativ),
            _ => Err(E::custom(format!(
                "String does not contain expected value: {}",
                value
            ))),
        }
    }
}

/// The different available news categorys
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Ressort {
    /// With this option, the ressort will not be specified and all results will be shown.
    None,
    /// Only news from Germany.
    Inland,
    /// Only news from outside of Germany.
    Ausland,
    /// Economic news.
    Wirtschaft,
    /// Sports news.
    Sport,
    /// Different kinds of videos.
    Video,
    /// Investigative journalism.
    Investigativ,
}

impl Display for Ressort {
    /// Formats the ressort value in a way that is usable by the underlying API.
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

impl<'de> Deserialize<'de> for Ressort {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(RessortVisitor)
    }
}

/// A timeframe for which the news should be fetched.
pub enum Timeframe {
    /// The current date.
    Now,
    /// A specific singular date.
    Date(TDate),
    /// A range of dates.
    DateRange(DateRange),
}

/// A date format for usage in [`Timeframes`](Timeframe).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TDate {
    day: u8,
    month: Month,
    year: i32,
}

impl TDate {
    /// Creates a `TDate` from the year, month, and day.
    pub fn from_calendar_date(year: i32, month: Month, day: u8) -> Result<Self, Error> {
        let date = Date::from_calendar_date(year, month.to_time_month(), day)?;
        Ok(TDate::from_time_date(date))
    }

    /// Creates a `TDate` from a [Date].
    pub fn from_time_date(d: Date) -> Self {
        TDate {
            day: d.day(),
            month: Month::from_time_month(d.month()),
            year: d.year(),
        }
    }

    /// Formats the date in a way that is usable by the underlying API (yymmdd).
    fn format(&self) -> String {
        format!(
            "{}{}{}",
            self.year % 100,
            format!("{:0>2}", self.month as u8),
            format!("{:0>2}", self.day)
        )
    }
}

/// A collection of unique [`TDates`](TDate).
#[derive(Clone)]
pub struct DateRange {
    dates: HashSet<TDate>,
}

impl DateRange {
    /// Generates a `DateRange` by encompassing dates within the range defined by two specified [`TDates`](TDate).
    pub fn new(start: TDate, end: TDate) -> Result<Self, Error> {
        let mut dates: Vec<TDate> = Vec::new();

        let mut s = Date::from_calendar_date(start.year, start.month.to_time_month(), start.day)?;

        let e = Date::from_calendar_date(end.year, end.month.to_time_month(), end.day)?;

        while s <= e {
            dates.push(TDate::from_time_date(s));
            s = s.next_day().unwrap();
        }

        Ok(Self {
            dates: HashSet::from_iter(dates.into_iter()),
        })
    }

    /// Creates a `DateRange` from a collection of [`TDates`](TDate).
    pub fn from_dates(dates: Vec<TDate>) -> Self {
        Self {
            dates: HashSet::from_iter(dates.into_iter()),
        }
    }
}

/// A client for the [Tagesschau](https://www.tagesschau.de) `/api2/news` endpoint.
pub struct TRequestBuilder {
    ressort: Ressort,
    regions: HashSet<Region>,
    timeframe: Timeframe,
}

impl TRequestBuilder {
    /// Creates a `TRequestBuilder` with no specified ressort, region and the current date as timeframe.
    pub fn new() -> Self {
        Self {
            ressort: Ressort::None,
            regions: HashSet::new(),
            timeframe: Timeframe::Now,
        }
    }

    /// Sets an existing `TRequestBuilder`'s selected ressort.
    pub fn ressort(&mut self, res: Ressort) -> &mut TRequestBuilder {
        self.ressort = res;
        self
    }

    /// Sets an existing `TRequestBuilder`'s selected regions.
    pub fn regions(&mut self, reg: HashSet<Region>) -> &mut TRequestBuilder {
        self.regions = reg;
        self
    }

    /// Sets an existing `TRequestBuilder`'s selected timeframe.
    pub fn timeframe(&mut self, timeframe: Timeframe) -> &mut TRequestBuilder {
        self.timeframe = timeframe;
        self
    }

    /// Creates the queryable URL for the `fetch` method.
    fn prepare_url(&self, date: TDate) -> Result<String, Error> {
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

    /// Processes the URLs created by `prepare_url`.
    async fn fetch(&self, date: TDate) -> Result<Articles, Error> {
        let url = self.prepare_url(date)?;

        let response = reqwest::get(url).await.map_err(|e| Error::BadRequest(e))?;

        let text = match response.status() {
            StatusCode::OK => response.text().await.map_err(|e| Error::ParsingError(e))?,
            _ => Err(Error::InvalidResponse(response.status().as_u16()))?,
        };

        let articles: Articles = serde_json::from_str(&text)?;

        Ok(articles)
    }

    /// Query all articles that match the parameters currently specified on the `TRequestBuilder` Object in form of [Content].
    pub async fn get_all_articles(&self) -> Result<Vec<Content>, Error> {
        let dates: Vec<TDate> = match &self.timeframe {
            Timeframe::Now => {
                let now = OffsetDateTime::now_local()?;

                vec![TDate::from_time_date(now.date())]
            }
            Timeframe::Date(date) => {
                vec![*date]
            }
            Timeframe::DateRange(date_range) => {
                Vec::from_iter(date_range.dates.clone().into_iter())
            }
        };

        let mut content: Vec<Content> = Vec::new();

        for date in dates {
            let mut art = self.fetch(date).await?;

            content.append(&mut art.news)
        }

        content.sort_by(|element, next| {
            let date_element = match element {
                Content::Text(t) => t.date,
                Content::Video(v) => v.date,
            };

            let date_next = match next {
                Content::Text(t) => t.date,
                Content::Video(v) => v.date,
            };

            if date_element > date_next {
                Ordering::Greater
            } else if date_element < date_next {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });

        Ok(content)
    }

    /// Query only [`Text`] articles that match the parameters currently specified on the `TRequestBuilder` Object.
    pub async fn get_text_articles(&self) -> Result<Vec<Text>, Error> {
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

    /// Query only [`Videos`](Video) that match the parameters currently specified on the `TRequestBuilder` Object.
    pub async fn get_video_articles(&self) -> Result<Vec<Video>, Error> {
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
struct Articles {
    news: Vec<Content>,
}

/// A value returned by the [RequestBuilder] that can be either a text article or a video.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Content {
    #[allow(missing_docs)]
    Text(Text),
    #[allow(missing_docs)]
    Video(Video),
}

impl PartialEq for Content {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Content {
    /// Checks if the `Content` is a [`Text`].
    pub fn is_text(&self) -> bool {
        match self {
            Content::Text(_) => true,
            Content::Video(_) => false,
        }
    }

    /// Checks if the `Content` is a [`Video`].
    pub fn is_video(&self) -> bool {
        match self {
            Content::Text(_) => false,
            Content::Video(_) => true,
        }
    }

    /// Unpacks a and returns a [`Text`].
    pub fn to_text(self) -> Result<Text, Error> {
        match self {
            Content::Text(text) => Ok(text),
            Content::Video(_) => Err(Error::ConversionError),
        }
    }

    /// Unpacks a and returns a [`Video`].
    pub fn to_video(self) -> Result<Video, Error> {
        match self {
            Content::Video(video) => Ok(video),
            Content::Text(_) => Err(Error::ConversionError),
        }
    }
}

/// A text article returned by the API.
#[derive(Deserialize, Debug)]
pub struct Text {
    #[allow(missing_docs)]
    pub title: String,
    #[allow(missing_docs)]
    #[serde(with = "rfc3339")]
    pub date: OffsetDateTime,
    #[allow(missing_docs)]
    #[serde(rename(deserialize = "detailsweb"))]
    pub url: String,
    #[allow(missing_docs)]
    #[serde(default = "default_tag_vec")]
    pub tags: Vec<Tag>,
    #[allow(missing_docs)]
    #[serde(default = "default_ressort")]
    pub ressort: Ressort,
    #[allow(missing_docs)]
    #[serde(rename(deserialize = "type"))]
    pub kind: String,
    #[allow(missing_docs)]
    #[serde(rename(deserialize = "breakingNews"), default = "default_bool")]
    pub breaking_news: bool,
    #[allow(missing_docs)]
    #[serde(rename(deserialize = "teaserImage"), default = "default_images")]
    pub image: Image,
}

/// A video returned by the API.
#[derive(Deserialize, Debug)]
pub struct Video {
    #[allow(missing_docs)]
    pub title: String,
    #[allow(missing_docs)]
    #[serde(with = "rfc3339")]
    pub date: OffsetDateTime,
    #[allow(missing_docs)]
    /// A [`HashMap`] consisting of (stream type, URL) (key, value) pairs.
    pub streams: HashMap<String, String>,
    #[allow(missing_docs)]
    #[serde(default = "default_tag_vec")]
    pub tags: Vec<Tag>,
    #[allow(missing_docs)]
    #[serde(default = "default_string")]
    pub ressort: String,
    #[allow(missing_docs)]
    #[serde(rename(deserialize = "type"))]
    pub kind: String,
    #[allow(missing_docs)]
    #[serde(rename(deserialize = "breakingNews"), default = "default_bool")]
    pub breaking_news: bool,
    #[allow(missing_docs)]
    #[serde(rename(deserialize = "teaserImage"), default = "default_images")]
    pub image: Image,
}

/// A tag value for a [`Text`] or a [`Video`].
#[derive(Deserialize, Debug)]
pub struct Tag {
    #[allow(missing_docs)]
    pub tag: String,
}

/// A struct that contains an images metadata and variants.
#[derive(Deserialize, Debug)]
pub struct Image {
    #[allow(missing_docs)]
    #[serde(default = "default_string")]
    pub title: String,
    #[serde(default = "default_string")]
    #[allow(missing_docs)]
    pub copyright: String,
    #[serde(default = "default_string")]
    #[allow(missing_docs)]
    pub alttext: String,
    /// A [`HashMap`] consisting of (image size, URL) (key, value) pairs
    #[serde(rename(deserialize = "imageVariants"), default = "default_hash_map")]
    pub image_variants: HashMap<String, String>,
    #[allow(missing_docs)]
    #[serde(rename(deserialize = "type"))]
    pub kind: String,
}

fn default_string() -> String {
    "".to_string()
}

fn default_hash_map() -> HashMap<String, String> {
    HashMap::new()
}

fn default_tag_vec() -> Vec<Tag> {
    Vec::new()
}

fn default_bool() -> bool {
    false
}

fn default_images() -> Image {
    Image {
        title: "".to_string(),
        copyright: "".to_string(),
        alttext: "".to_string(),
        image_variants: HashMap::new(),
        kind: "".to_string(),
    }
}

fn default_ressort() -> Ressort {
    Ressort::None
}

/// The Errors that might occur when using the API
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[allow(missing_docs)]
    #[error("Fetching articles failed")]
    BadRequest(reqwest::Error),
    #[allow(missing_docs)]
    #[error("Failed to parse response")]
    ParsingError(reqwest::Error),
    #[allow(missing_docs)]
    #[error("Invalid Response: HTTP Response Code {0}")]
    InvalidResponse(u16),
    #[allow(missing_docs)]
    #[error("Failed to deserialize response")]
    DeserializationError(#[from] serde_json::Error),
    #[allow(missing_docs)]
    #[error("Tried to extract wrong type")]
    ConversionError,
    #[allow(missing_docs)]
    #[error("Unable to retrieve current date")]
    DateError(#[from] time::error::IndeterminateOffset),
    #[allow(missing_docs)]
    #[error("Unable parse date")]
    DateParsingError(#[from] time::error::ComponentRange),
    #[allow(missing_docs)]
    #[error("Url parsing failed")]
    UrlParsing(#[from] url::ParseError),
}
