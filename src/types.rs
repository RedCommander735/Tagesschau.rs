use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use time::{Month, OffsetDateTime};

const BASE_URL: &str = "https://www.tagesschau.de/api2u/news?";

#[derive(Clone, Copy)]
pub enum Region {
    BadenWürttemberg,
    Bayern,
    Berlin,
    Brandenburg,
    Bremen,
    Hamburg,
    Hessen,
    MecklenburgVorpommern,
    Niedersachsen,
    NordrheinWestfalen,
    RheinlandPfalz,
    Saarland,
    Sachsen,
    SachsenAnhalt,
    SchleswigHolstein,
    Thüringen,
}

impl Region {
    pub fn index(&self) -> usize {
        (*self as usize) + 1
    }
}

pub enum Ressort {
    None,
    Inland,
    Ausland,
    Wirtschaft,
    Sport,
    Video,
    Investigativ,
    Wissen,
}

impl Ressort {
    pub fn as_text(self) -> String {
        match self {
            Ressort::None => "".to_owned(),
            Ressort::Inland => "inland".to_owned(),
            Ressort::Ausland => "ausland".to_owned(),
            Ressort::Wirtschaft => "wirtschaft".to_owned(),
            Ressort::Sport => "sport".to_owned(),
            Ressort::Video => "video".to_owned(),
            Ressort::Investigativ => "investigativ".to_owned(),
            Ressort::Wissen => "wissen".to_owned(),
        }
    }
}

pub enum Timeframe {
    Now,
    Date(Date),
    DateRange(DateRange),
}

pub struct Date {
    day: u8,
    month: Month,
    year: u16,
}

#[derive(Clone)]
pub struct DateRange {
    dates: Vec<OffsetDateTime>,
}

impl DateRange {
    fn new(start: Date, end: Date) -> DateRange {
        let dates: Vec<OffsetDateTime> = Vec::new();

        let raw_dates: Vec<Date> = Vec::new();

        // TODO Parse start and end and generate range in between, return as DateRange object
        todo!()
    }
}

struct TagesschauAPI {
    ressort: Ressort,
    regions: HashSet<Region>,
    timeframe: Timeframe,
}

impl TagesschauAPI {
    fn new() -> TagesschauAPI {
        TagesschauAPI {
            ressort: Ressort::None,
            regions: HashSet::new(),
            timeframe: Timeframe::Now,
        }
    }

    fn ressort(&mut self, res: Ressort) -> &mut TagesschauAPI {
        self.ressort = res;
        self
    }

    fn regions(&mut self, reg: HashSet<Region>) -> &mut TagesschauAPI {
        self.regions = reg;
        self
    }

    fn date(&mut self, timeframe: Timeframe) -> &mut TagesschauAPI {
        self.timeframe = timeframe;
        self
    }

    fn prepare_url(&self) -> Result<Vec<String>, TagesschauApiError> {
        let dates: Vec<OffsetDateTime> = match &self.timeframe {
            Timeframe::Now => {
                let now =
                    OffsetDateTime::now_local().map_err(|e| TagesschauApiError::DateError(e))?;

                vec![now]
            }
            Timeframe::Date(date) => {
                // TODO Parse date string here and return vec with date
                todo!()
            }
            Timeframe::DateRange(date_range) => date_range.dates.clone(),
        };

        // TODO for each date build the relevant string from date, region and ressort
        todo!()
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
    #[serde(rename(deserialize = "detailsweb"))]
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct Video {
    pub title: String,
    pub streams: HashMap<String, String>,
}

#[derive(thiserror::Error, Debug)]
pub enum TagesschauApiError {
    #[error("Fetching articles failed")]
    RequestFailed(reqwest::Error),
    #[error("Failed to parse response")]
    ParsingError(reqwest::Error),
    #[error("Failed to deserialize response")]
    DeserializationError(serde_json::Error),
    #[error("Tried to extract wrong type")]
    ConversionError,
    #[error("Unable to retrieve current date")]
    DateError(time::error::IndeterminateOffset),
    // DateRangeError,
    // DateParsingError,
}
