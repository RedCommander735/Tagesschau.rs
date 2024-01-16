use reqwest;
use time::{format_description, OffsetDateTime};
mod types;
use types::{Articles, TagesschauApiError, Text, Video};

pub fn get_all(date: &str) -> Result<Articles, TagesschauApiError> {
    let url = format!("https://www.tagesschau.de/api2u/news?date={date}");
    let response = reqwest::blocking::get(url)
        .map_err(|e| TagesschauApiError::RequestFailed(e))?
        .text()
        .map_err(|e| TagesschauApiError::ParsingError(e))?;

    let articles: Articles =
        serde_json::from_str(&response).map_err(|e| TagesschauApiError::DeserializationError(e))?;

    Ok(articles)
}

pub fn get_text_articles(date: &str) -> Result<Vec<Text>, TagesschauApiError> {
    let articles = get_all(date);

    match articles {
        Ok(mut a) => {
            a.news.retain(|x| x.is_text());
            let mut t: Vec<Text> = Vec::new();

            for content in a.news {
                t.push(content.to_text().unwrap())
            }

            Ok(t)
        }
        Err(e) => Err(e),
    }
}

pub fn get_video_articles(date: &str) -> Result<Vec<Video>, TagesschauApiError> {
    let articles = get_all(date);

    match articles {
        Ok(mut a) => {
            a.news.retain(|x| x.is_video());
            let mut t: Vec<Video> = Vec::new();

            for content in a.news {
                t.push(content.to_video().unwrap())
            }

            Ok(t)
        }
        Err(e) => Err(e),
    }
}
