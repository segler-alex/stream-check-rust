FROM rust:1
ADD . /root
WORKDIR /root
RUN cargo install

EXPOSE 8080
CMD [ "stream-check-rust" ]
