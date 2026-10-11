# Step 1: Build the Vue frontend
FROM node:20-alpine AS frontend-builder
WORKDIR /app
# 版本号由构建参数注入（来自 git commit），会编译进前端
ARG APP_VERSION=dev
ARG APP_COMMIT_MSG=
ENV APP_VERSION=$APP_VERSION
ENV APP_COMMIT_MSG=$APP_COMMIT_MSG
COPY package*.json ./
RUN npm install --no-audit --no-fund --fetch-retries=5 --fetch-retry-mintimeout=20000 --fetch-retry-maxtimeout=120000
COPY . .
RUN npm run build

# Step 2: Build the Golang backend
FROM golang:1.26-alpine AS backend-builder
WORKDIR /app
COPY backend/go.mod backend/go.sum ./
RUN go mod download
COPY backend/ .
RUN CGO_ENABLED=0 GOOS=linux go build -o server main.go

# Step 3: Final production image
FROM alpine:latest
RUN apk --no-cache add ca-certificates

WORKDIR /app/backend
# Copy backend binary from builder
COPY --from=backend-builder /app/server .
# Copy frontend dist folder from builder to the expected location
COPY --from=frontend-builder /app/dist ../dist

# Set working directory to where the binary is, so "../dist" paths work
WORKDIR /app/backend

EXPOSE 8080

# Run the backend
CMD ["./server"]
