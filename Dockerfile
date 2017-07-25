FROM scratch

ADD ./target/x86_64-unknown-linux-musl/release/fblog /fblog

ENTRYPOINT ["/fblog"]
CMD []
