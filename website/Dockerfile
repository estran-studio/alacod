# BUILDING STAGE
FROM node:alpine AS builder


ARG APP_VERSION

WORKDIR /app

COPY package*.json ./
COPY vendor ./vendor

RUN npm ci

COPY . .

RUN npm run build



# SERVING STAGE
FROM nginx:alpine

WORKDIR /usr/share/nginx/html

RUN rm index.html

COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/build /usr/share/nginx/html

EXPOSE 80

LABEL org.opencontainers.image.source=https://github.com/bascanada/alacod
LABEL org.opencontainers.image.description="Container image for the website with the wasm"

