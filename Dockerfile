FROM alpine:3.18 as build

RUN apk add --no-cache libgcc cargo

# Copy, build, and install
COPY . /repo
RUN cd /repo && cargo build --release
RUN install -Dvm755 /repo/target/release/tg2h /usr/bin/tg2h

# Cleanup
RUN rm -rf /repo /root/.cargo
RUN apk del cargo

# Flatten
FROM scratch
COPY --from=build / /

USER nobody
CMD ["/usr/bin/tg2h"]