FROM rust:1-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release -p cq-server

FROM node:24-bookworm AS web
WORKDIR /web
COPY web/player/package*.json ./
RUN npm install
COPY web/player ./
RUN npm run build

FROM node:24-bookworm AS admin-web
WORKDIR /admin-web
COPY web/admin/package*.json ./
RUN npm install
COPY web/admin ./
RUN npm run build

FROM debian:bookworm-slim
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates postgresql-client tzdata wget \
  && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/cq-server /app/cq-server
COPY --from=web /web/dist /app/public
COPY --from=admin-web /admin-web/dist /app/public/admin
COPY docker /app/docker
COPY migrations /app/migrations
ENV PORT=3000
ENV PUBLIC_DIR=/app/public
ENV TZ=Asia/Shanghai
EXPOSE 3000
CMD ["/app/cq-server"]
