FROM alpine:3.17 as build

# These from the stable branch
RUN apk add --no-cache git libgcc
# This from edge. TODO: Change this once Rust 1.65 gets to stable in Alpine Linux
RUN apk add --no-cache --repository=https://dl-cdn.alpinelinux.org/alpine/edge/community cargo

# Clone, build, and install
RUN git clone https://github.com/jlxip/tg2h ~/tg2h
RUN cd ~/tg2h && cargo build --release
RUN install -Dvm755 ~/tg2h/target/release/tg2h /usr/bin/tg2h

# Cleanup
RUN rm -rf ~/tg2h ~/.cargo
RUN apk del git cargo

# Flatten
FROM scratch
COPY --from=build / /

USER nobody
CMD ["/usr/bin/tg2h"]