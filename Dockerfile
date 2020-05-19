FROM rustlang/rust:nightly

RUN cd / && USER=root cargo new app
WORKDIR /app 
COPY ./image_server/Cargo.toml /app
RUN cargo build
COPY ./image_server .
RUN cargo build --release 

ENTRYPOINT ./target/release/image_server
