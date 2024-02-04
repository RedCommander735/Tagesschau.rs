# tagesschau-rs

[![Crates.io](https://img.shields.io/crates/v/tagesschau)](https://crates.io/crates/tagesschau)
[![Documentation](https://img.shields.io/docsrs/tagesschau)](https://docs.rs/tagesschau/)
[![License: MIT](https://img.shields.io/crates/l/tagesschau)](LICENSE)

<!-- cargo-rdme start -->

A client library for interacting with the [Tagesschau](https://www.tagesschau.de)'s `/api2/news` endpoint.

## Example
```rust
let start = TDate::from_calendar_date(2024, Month::January, 20)?;
let end = TDate::from_calendar_date(2024, Month::January, 31)?;

let mut builder = TRequestBuilder::new();

builder
    .ressort(Ressort::Wirtschaft)
    .timeframe(tagesschau::Timeframe::DateRange(
        DateRange::new(start, end)?,
    ));

let articles: Vec<TextArticle> = builder.get_text_articles().await?;

for article in articles {
    println!("{} - {}", article.title(), article.date().time());
}

```
<details><summary>Results in something like</summary>

```
Gesetzlicher Mindestlohn zeigt positive Wirkung - 14:52:03.304
E-Autos werden beliebter – nur nicht in Deutschland - 17:07:02.836
Fed lässt Leitzins erneut unverändert - 20:50:58.427
Fed enttäuscht Zinshoffnungen - 22:16:27.875
...
```
</details>

<!-- cargo-rdme end -->

## License

This project is licensed under the MIT License.

See [LICENSE](LICENSE) for more information.


## TODO:
- [ ] Support for multiple ressorts
- [ ] Support for Datetime timeframes (limit to 12.00h to 13.00h e.g.)
- [ ] Support for a limited amount of articles