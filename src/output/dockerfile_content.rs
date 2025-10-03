pub fn dockerfile_content(project_name: &str) -> String {
    format!(
        r#"
FROM rust:1.70 AS builder
WORKDIR /usr/src/{}
COPY . .
RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder /usr/src/{}/target/release/{} /usr/local/bin/app
CMD ["app"]
"#,
        project_name, project_name, project_name
    )
}
