# Build stage
FROM rust:1.69-bullseye as builder

WORKDIR /app

# Accept the build argument
ARG DATABASE_URL

ENV DATABASE_URL=${DATABASE_URL}

COPY . .

RUN cargo build --release

# Production stage
FROM debian:bullseye-slim

WORKDIR /usr/local/bin

COPY --from=builder /app/target/release/newsapi .

# Ensure the environment variable is set in the final container
ENV DATABASE_URL=${DATABASE_URL}

CMD ["./newsapi"]
