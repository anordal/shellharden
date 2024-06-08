# docker build . -f docker/Dockerfile --tag shellharden:latest


FROM rust:1.53.0-alpine AS build

WORKDIR /src
COPY . .
RUN cargo build --release


FROM scratch
COPY --from=build /src/target/release/shellharden /shellharden
ENTRYPOINT ["/shellharden"]
