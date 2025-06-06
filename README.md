<https://github.com/tokio-rs/axum>
# Axum Web Service

V1 : This is a simple Todo application built using the Axum framework, which is a web framework for Rust (Top microservices framework for Rust, As of 2024).

## Features

- RESTful API for managing todos
- PostgreSQL integration using [sqlx](https://github.com/launchbadge/sqlx)
- Lightweight DI via Axum's with_state() and extractors — supports the DI pattern without a dedicated container.
- [Tokio](https://github.com/tokio-rs/tokio) runtime
- sqlx-cli is used to manage database, schema migrations.
- CORS (Cross-Origin Resource Sharing) support

## Looking to add

- [Auth](https://github.com/tokio-rs/axum/tree/master/examples/auth)
- Change from simple todos to be Personal blog : Because Static site generation (SSG) site does not support realtime post updates. Sometimes, I need to update my blog away from my computer. Also, I want to create a full stack web as a project to demonstrate my skills.

## Getting Started

### Prerequisites

- Rust and Cargo installed
- PostgreSQL installed and running

![image](https://github.com/boraxpr/rust_axum_service/assets/43258373/83b5ddd3-bd5a-484b-81d5-7e17551b56ea)

