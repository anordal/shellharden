FROM alpine:3.8 as builder

WORKDIR /build
ADD . .
RUN apk add --no-cache cargo
RUN cargo build --release


FROM alpine:3.8
RUN apk add --no-cache libgcc
COPY --from=builder /build/target/release/shellharden /
ENTRYPOINT ["/shellharden"]
CMD ["--help"]
