FROM alpine:latest

WORKDIR /app

RUN apk add --no-cache curl
RUN curl https://git.4rs.nl/awiteb/forgejo-guardian/releases/download/v0.4.1/forgejo-guardian-v0.4.1-x86_64-linux-musl --output forgejo-guardian
RUN chmod +x forgejo-guardian

ENTRYPOINT ["./forgejo-guardian"]
