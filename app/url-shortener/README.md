# Geektime Rust 语言训练营第四周：url-shortener

一个简单的 url-shortener 实现。

- requirement
  - http post api to create new shortened record
  - http get api to retrive original url from shortened id, and redirect

- technical requirement
  - use thiserror to define error message
  - auto-regen shortened id if conflict
  - use sqlx macro
