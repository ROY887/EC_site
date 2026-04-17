FROM node:current-alpine AS builder

WORKDIR /app

COPY package*.json ./

RUN npm ci

COPY . .

FROM node:current-alpine

WORKDIR /app

COPY --from=builder /app/node_modules ./node_modules

COPY . .

EXPOSE 5173

CMD ["npm","run", "dev", "--", "--host", "0.0.0.0"]