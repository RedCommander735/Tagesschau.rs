use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use time::{Date, Month, OffsetDateTime};

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
    fn from_time_date(d: Date) -> TDate {
        TDate {
            day: d.day(),
            month: d.month(),
            year: d.year(),
        }
    }

    fn format(&self) -> String {
        format!(
            "{}{}{}",
            self.year,
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
    fn new(start: TDate, end: TDate) -> Result<DateRange, TagesschauApiError> {
        let mut dates: Vec<TDate> = Vec::new();

        let mut s = Date::from_calendar_date(start.year, start.month, start.day)
            .map_err(|e| TagesschauApiError::DateParsingError(e))?;

        let e = Date::from_calendar_date(end.year, end.month, end.day)
            .map_err(|e| TagesschauApiError::DateParsingError(e))?;

        while s <= e {
            dates.push(TDate::from_time_date(s));
            s = s.next_day().unwrap();
        }

        Ok(DateRange { dates })
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
        let dates: Vec<TDate> = match &self.timeframe {
            Timeframe::Now => {
                let now =
                    OffsetDateTime::now_local().map_err(|e| TagesschauApiError::DateError(e))?;

                vec![TDate::from_time_date(now.date())]
            }
            Timeframe::Date(date) => {
                vec![*date]
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
    #[error("Unable parse date")]
    DateParsingError(time::error::ComponentRange),
}
