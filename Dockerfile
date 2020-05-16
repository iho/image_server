FROM rustlang/rust:nightly

RUN cd / && USER=root cargo new app
WORKDIR /app 
COPY ./image_server/Cargo.toml /app
RUN cargo build
RUN rm -rf /app/*

COPY ./image_server .

RUN cargo install --path .

CMD ["image_server"]