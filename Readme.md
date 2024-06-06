# prkserver

`prkserver` is a CLI tool that helps create a backend server in Rust using Axum and SQLx. It configures everything based on a provided `config.toml` file.

## Features

- Generates a Rust backend project using Axum for HTTP handling.
- Configures SQLx for database interactions.
- Supports PostgreSQL and MySQL databases.
- Creates models and endpoints as specified in the `config.toml` file.

## Installation

To install `prkserver`, use cargo:

```sh
cargo install prkserver
```

## Usage

To use `prkserver`, create a `config.toml` file that defines the project configuration. Here is an example `config.toml` file:

```toml
project_name = "backend_project"
database_url = "mysql://user:password@localhost/database_name"
database_type = "mysql"

[[models]]
name = "User"
table_name = "users"
fields = [
    { name = "id", type = "i32" },
    { name = "username", type = "String" },
    { name = "email", type = "String" },
]
endpoints = [
    { method = "GET", path = "/users" },
    { method = "POST", path = "/users" },
]


[[models]]
name = "Todo"
table_name = "todos"
fields = [
    { name = "id", type = "i32" },
    { name = "task", type = "String" },
    { name = "description", type = "Option<String>" },
]
endpoints = [
    { method = "GET", path = "/todos" },
    { method = "GET", path = "/todos/:id" },
]
```

Once you have your config.toml file, run prkserver at the path of this config file:
```sh
prkserver
```

This will generate a new project in a directory named after project_name specified in the config.toml.