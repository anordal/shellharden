FROM alpine:3.8 as builder

WORKDIR /build
ADD https://api.github.com/repos/anordal/shellharden/compare/master...HEAD /dev/null
RUN apk add --no-cache cargo git
RUN git clone https://github.com/anordal/shellharden
RUN cd shellharden && cargo build --release


FROM alpine:3.8
RUN apk add --no-cache libgcc
COPY --from=builder /build/shellharden/target/shellharden /
ENTRYPOINT ["/shellharden"]
CMD ["--help"]
