FROM rust:1.51.0-alpine AS build

WORKDIR /src
RUN apk add --no-cache git
COPY . .
RUN cargo build --release

FROM alpine:3.13.2 AS alpine
COPY --from=build /src/target/release/shellharden /bin/shellharden
COPY "./docker-entrypoint.sh" "/init"
ENTRYPOINT ["/init"]

FROM scratch
COPY --from=build /src/target/release/shellharden  /bin/shellharden
ENTRYPOINT ["/bin/shellharden"]
CMD ["-h"]
