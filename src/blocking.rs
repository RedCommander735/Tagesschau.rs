use std::cmp::Ordering;

use reqwest::StatusCode;
use time::OffsetDateTime;

use crate::{Articles, Content, Error, TDate, TRequestBuilder, TextArticle, Timeframe, Video};

impl TRequestBuilder {
    fn fetch_blocking(&self, date: TDate) -> Result<Articles, Error> {
        let url = self.prepare_url(date)?;

        let response = reqwest::blocking::get(url).map_err(|e| Error::BadRequest(e))?;

        let text = match response.status() {
            StatusCode::OK => response.text().map_err(|e| Error::ParsingError(e))?,
            _ => Err(Error::InvalidResponse(response.status().as_u16()))?,
        };

        let articles: Articles = serde_json::from_str(&text)?;

        Ok(articles)
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
    /// Query all articles that match the parameters currently specified on the `TRequestBuilder` Object in form of [Content] as a blocking request.
    pub fn get_all_articles_blocking(&self) -> Result<Vec<Content>, Error> {
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
            let mut art = self.fetch_blocking(date)?;

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

    #[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
    /// Query only [`TextArticle`] articles that match the parameters currently specified on the `TRequestBuilder` Object as a blocking request.
    pub fn get_text_articles_blocking(&self) -> Result<Vec<TextArticle>, Error> {
        let articles = self.get_all_articles_blocking();

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

    #[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
    /// Query only [`Videos`](Video) that match the parameters currently specified on the `TRequestBuilder` Object as a blocking request.
    pub fn get_video_articles_blocking(&self) -> Result<Vec<Video>, Error> {
        let articles = self.get_all_articles_blocking();

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
