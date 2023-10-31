<div align="center">
<h1>Silent</h1>
<p>
<a href="https://github.com/hubertshelley/silent/actions">
    <img alt="build status" src="https://github.com/hubertshelley/silent/actions/workflows/build.yml/badge.svg" />
</a>
<br/>
<a href="https://crates.io/crates/silent"><img alt="crates.io" src="https://img.shields.io/crates/v/silent" /></a>
<a href="https://docs.rs/silent"><img alt="Documentation" src="https://docs.rs/silent/badge.svg" /></a>
<a href="https://github.com/rust-secure-code/safety-dance/"><img alt="unsafe forbidden" src="https://img.shields.io/badge/unsafe-forbidden-success.svg" /></a>
<a href="https://blog.rust-lang.org/2022/09/22/Rust-1.71.0.html"><img alt="Rust Version" src="https://img.shields.io/badge/rust-1.74%2B-blue" /></a>
<br/>
<a href="https://crates.io/crates/silent"><img alt="Download" src="https://img.shields.io/crates/d/silent.svg" /></a>
<img alt="License" src="https://img.shields.io/crates/l/silent.svg" />
</p>
</div>

### 概要

Silent 是一个简单的基于Hyper的Web框架，它的目标是提供一个简单的、高效的、易于使用的Web框架。

### 目标

- [x] 路由
- [x] 中间件
- [x] 静态文件
- [x] WebSocket
- [x] 模板
- [ ] 数据库
- [x] 日志 (使用了tracing)
- [x] 配置
- [x] 会话
- [x] 安全
- [ ] 测试
- [ ] 文档

## security

### argon2

add make_password and verify_password function

### pbkdf2

add make_password and verify_password function

### aes

re-export aes/aes_gcm

### rsa

re-export rsa

## configs

### setting

```rust
use silent::Configs;
let mut configs = Configs::default();
configs.insert(1i32);
```

### usage

```rust
async fn call(req: Request) -> Result<i32> {
    let num = req.configs().get::<i32>().unwrap();
    Ok(*num)
}
```
