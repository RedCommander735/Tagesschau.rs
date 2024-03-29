#![warn(missing_docs)]
// only enables the `doc_cfg` feature when
// the `docsrs` configuration attribute is defined
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use reqwest;

use reqwest::StatusCode;
use serde::{de, Deserialize, Deserializer};
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

/// The different available news categorys
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone)]
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
    /// Informative news that refutes false reports, explain the background and encourage reflection.
    Wissen,
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
            Ressort::Wissen => f.write_str("wissen"),
        }
    }
}

impl<'de> Deserialize<'de> for Ressort {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "" => Ok(Ressort::None),
            "inland" => Ok(Ressort::Inland),
            "ausland" => Ok(Ressort::Ausland),
            "wirtschaft" => Ok(Ressort::Wirtschaft),
            "sport" => Ok(Ressort::Sport),
            "video" => Ok(Ressort::Video),
            "investigativ" => Ok(Ressort::Investigativ),
            "wissen" => Ok(Ressort::Wissen),
            _ => Err(de::Error::custom(format!(
                "String does not contain expected value: {}",
                s
            ))),
        }
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
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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
}

impl Display for TDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.year % 100,
            format!("{:0>2}", self.month as u8),
            format!("{:0>2}", self.day)
        )
    }
}

/// A collection of unique [`TDates`](TDate).
#[derive(Clone, Debug)]
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

        url.query_pairs_mut().append_pair("date", &date.to_string());

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
                Content::TextArticle(t) => t.date,
                Content::Video(v) => v.date,
            };

            let date_next = match next {
                Content::TextArticle(t) => t.date,
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

    /// Query only [`TextArticle`] articles that match the parameters currently specified on the `TRequestBuilder` Object.
    pub async fn get_text_articles(&self) -> Result<Vec<TextArticle>, Error> {
        let articles = self.get_all_articles().await;

        match articles {
            Ok(mut a) => {
                a.retain(|x| x.is_text());
                let mut t: Vec<TextArticle> = Vec::new();

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

#[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
#[cfg(feature = "blocking")]
mod blocking;

#[derive(Deserialize, Debug)]
struct Articles {
    news: Vec<Content>,
}

/// A value returned by the [TRequestBuilder] that can be either a text article or a video.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Content {
    #[allow(missing_docs)]
    TextArticle(TextArticle),
    #[allow(missing_docs)]
    Video(Video),
}

impl Content {
    /// Checks if the `Content` is a [`TextArticle`].
    pub fn is_text(&self) -> bool {
        match self {
            Content::TextArticle(_) => true,
            Content::Video(_) => false,
        }
    }

    /// Checks if the `Content` is a [`Video`].
    pub fn is_video(&self) -> bool {
        match self {
            Content::TextArticle(_) => false,
            Content::Video(_) => true,
        }
    }

    /// Unpacks a and returns a [`TextArticle`].
    pub fn to_text(self) -> Result<TextArticle, Error> {
        match self {
            Content::TextArticle(text) => Ok(text),
            Content::Video(_) => Err(Error::ConversionError),
        }
    }

    /// Unpacks a and returns a [`Video`].
    pub fn to_video(self) -> Result<Video, Error> {
        match self {
            Content::Video(video) => Ok(video),
            Content::TextArticle(_) => Err(Error::ConversionError),
        }
    }
}

/// A text article returned by the API.
#[derive(Deserialize, Debug)]
pub struct TextArticle {
    title: String,
    #[serde(rename(deserialize = "firstSentence"))]
    first_sentence: String,
    #[serde(with = "rfc3339")]
    date: OffsetDateTime,
    #[serde(rename(deserialize = "detailsweb"))]
    url: String,
    tags: Option<Vec<Tag>>,
    ressort: Option<Ressort>,
    #[serde(rename(deserialize = "type"))]
    kind: String,
    #[serde(rename(deserialize = "breakingNews"))]
    breaking_news: Option<bool>,
    #[serde(rename(deserialize = "teaserImage"))]
    image: Option<Image>,
}

impl TextArticle {
    /// Get the title of this `TextArticle`.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get the first sentence of this `TextArticle`.
    pub fn first_sentence(&self) -> &str {
        &self.first_sentence
    }

    /// Get the publishing time of this `TextArticle` as [OffsetDateTime].
    pub fn date(&self) -> OffsetDateTime {
        self.date
    }

    /// Get the URL to this `TextArticle`.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the tags of this `TextArticle`.
    pub fn tags(&self) -> Option<Vec<&str>> {
        match &self.tags {
            Some(t) => {
                let mut tags: Vec<&str> = Vec::new();
                for tag in t {
                    tags.push(&tag.tag)
                }
                Some(tags)
            }
            None => None,
        }
    }

    /// Get the [`Ressort`] of this `TextArticle`.
    pub fn ressort(&self) -> Option<Ressort> {
        self.ressort
    }

    /// Get the type of `TextArticle` this is.
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Get if this `TextArticle` is breaking news or not.
    pub fn breaking_news(&self) -> Option<bool> {
        self.breaking_news
    }

    /// Get the image attached to this `TextArticle`.
    pub fn image(&self) -> Option<&Image> {
        self.image.as_ref()
    }
}

/// A video returned by the API.
#[derive(Deserialize, Debug)]
pub struct Video {
    title: String,
    #[serde(with = "rfc3339")]
    date: OffsetDateTime,
    streams: HashMap<String, String>,
    tags: Option<Vec<Tag>>,
    ressort: Option<Ressort>,
    #[serde(rename(deserialize = "type"))]
    kind: String,
    #[serde(rename(deserialize = "breakingNews"))]
    breaking_news: Option<bool>,
    #[serde(rename(deserialize = "teaserImage"))]
    image: Option<Image>,
}

impl Video {
    /// Get the title of this `Video`.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get the publishing time of this `Video` as [OffsetDateTime].
    pub fn date(&self) -> OffsetDateTime {
        self.date
    }

    /// Get the [`HashMap`] consisting of (stream-type, URL) (key, value) pairs of this `Video`.
    pub fn streams(&self) -> HashMap<&str, &str> {
        let mut streams: HashMap<&str, &str> = HashMap::new();
        for (key, value) in &self.streams {
            streams.insert(&key, &value);
        }
        streams
    }

    /// Get the tags of this `Video`.
    pub fn tags(&self) -> Option<Vec<&str>> {
        match &self.tags {
            Some(t) => {
                let mut tags: Vec<&str> = Vec::new();
                for tag in t {
                    tags.push(&tag.tag)
                }
                Some(tags)
            }
            None => None,
        }
    }

    /// Get the [`Ressort`] of this `Video`.
    pub fn ressort(&self) -> Option<Ressort> {
        self.ressort
    }

    /// Get the type of `Video` this is.
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Get if this `Video` is breaking news or not.
    pub fn breaking_news(&self) -> Option<bool> {
        self.breaking_news
    }

    /// Get the image attached to this `Video`.
    pub fn image(&self) -> Option<&Image> {
        match &self.image {
            Some(img) => match &img.image_variants {
                Some(variants) => {
                    let var = if variants.is_empty() { None } else { Some(img) };
                    var
                }
                None => None,
            },
            None => None,
        };
        self.image.as_ref()
    }
}

#[derive(Deserialize, Debug)]
struct Tag {
    tag: String,
}

/// A struct that contains an images metadata and variants.
#[derive(Deserialize, Debug, Clone)]
pub struct Image {
    title: Option<String>,
    copyright: Option<String>,
    alttext: Option<String>,
    #[serde(rename(deserialize = "imageVariants"))]
    image_variants: Option<HashMap<String, String>>,
    #[serde(rename(deserialize = "type"))]
    kind: String,
}

impl Image {
    /// Get the title of this `Image`.
    pub fn title(&self) -> Option<&str> {
        match &self.title {
            Some(title) => Some(&title),
            None => None,
        }
    }

    /// Get the copyright of this `Image`.
    pub fn copyright(&self) -> Option<&str> {
        match &self.copyright {
            Some(copyright) => Some(&copyright),
            None => None,
        }
    }

    /// Get the alt-text of this `Image`.
    pub fn alttext(&self) -> Option<&str> {
        match &self.alttext {
            Some(alttext) => Some(&alttext),
            None => None,
        }
    }

    /// Get the [`HashMap`] consisting of (image-resolution, URL) (key, value) pairs of this `Image`.
    pub fn image_variants(&self) -> HashMap<&str, &str> {
        let variants = self.image_variants.as_ref().unwrap();
        let mut image_variants: HashMap<&str, &str> = HashMap::new();
        for (key, value) in variants {
            image_variants.insert(&key, &value);
        }
        image_variants
    }

    /// Get the type of `Image` this is.
    pub fn kind(&self) -> &str {
        &self.kind
    }
}

/// The Errors that might occur when using the API.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Fetching articles failed.
    #[error("Fetching articles failed")]
    BadRequest(reqwest::Error),
    /// Failed to parse http response.
    #[error("Failed to parse response")]
    ParsingError(reqwest::Error),
    /// Invalid HTTP Response, contains HTTP response code.
    #[error("Invalid Response: HTTP Response Code {0}")]
    InvalidResponse(u16),
    /// Failed to deserialize response.
    #[error("Failed to deserialize response")]
    DeserializationError(#[from] serde_json::Error),
    /// Tried to extract wrong type from [Content].
    #[error("Tried to extract wrong type")]
    ConversionError,
    /// Unable to retrieve current date.
    #[error("Unable to retrieve current date")]
    DateError(#[from] time::error::IndeterminateOffset),
    /// Unable parse date.
    #[error("Unable parse date")]
    DateParsingError(#[from] time::error::ComponentRange),
    /// URL parsing failed.
    #[error("URL parsing failed")]
    UrlParsing(#[from] url::ParseError),
}
