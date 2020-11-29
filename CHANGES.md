# Changes

## Unreleased - 2020-12-xx
### Changed
* `QaQuery` now has a public representation: `QaQuery(pub T)` that enables
  destructuring.
  
  From now on, instead of:
  ```
  fn index(info: QsQuery<Info>) -> Result<String> {
      Ok(format!("Welcome {}!", info.username))
  }
  ```
  You can use:
  ```
  fn index(QaQuery(info): QsQuery<Info>) -> Result<String> {
      Ok(format!("Welcome {}!", info.username))
  }
  ```
* Update `serde_urlencoded` to `0.7.0`
