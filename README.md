
TODO:
- [ ] More documentation with (code) examples
- [ ] Support for multiple ressorts


# tagesschau-rs

[![Build Status](https://github.com/d-k-bo/mediathekviewweb-rs/workflows/CI/badge.svg)](https://github.com/d-k-bo/mediathekviewweb-rs/actions?query=workflow%3ACI)
[![Crates.io](https://img.shields.io/crates/v/mediathekviewweb)](https://lib.rs/crates/mediathekviewweb)
[![Documentation](https://img.shields.io/docsrs/mediathekviewweb)](https://docs.rs/mediathekviewweb)
[![License: MIT](https://img.shields.io/crates/l/mediathekviewweb)](LICENSE)

<!-- cargo-rdme start -->

A client library for interacting with the [Tagesschau](https://www.tagesschau.de)'s `/api2/news` endpoint.

## Example
```rust
#[tokio::main]
async fn main() {
    let start = TDate::from_calendar_date(2024, Month::January, 20).unwrap();
    let end = TDate::from_calendar_date(2024, Month::January, 31).unwrap();

    let mut builder = TRequestBuilder::new();

    builder
        .ressort(Ressort::Wirtschaft)
        .timeframe(tagesschau::Timeframe::DateRange(
            DateRange::new(start, end).unwrap(),
        ));

    let articles: Vec<TextArticle> = builder.get_text_articles().await.unwrap();

    for article in articles {
        println!("{}", article);
    }
}
```
<details><summary>Results in something like</summary>

```rust
```
</details>

<!-- cargo-rdme end -->

## License

This project is licensed under the MIT License.

See [LICENSE](LICENSE) for more information.