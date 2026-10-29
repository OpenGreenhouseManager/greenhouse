FROM --platform=$BUILDPLATFORM node:24-trixie-slim AS builder

ARG API_BASE_URL
ARG ANGULAR_CONFIGURATION=staging
ENV env=${ANGULAR_CONFIGURATION}

RUN mkdir /greenhouse_web
WORKDIR /greenhouse_web

COPY package.json package-lock.json ./
RUN npm ci

COPY . .
CMD ["sh", "-c", "npx ng serve --host 0.0.0.0 --configuration \"$env\""]

FROM builder AS dev-envs

RUN apt-get update && \
    apt-get install -y --no-install-recommends git

RUN useradd -s /bin/bash -m vscode && \
groupadd docker && \
usermod -aG docker vscode

# install Docker tools (cli, buildx, compose)
COPY --from=builder / /
EXPOSE 4200

CMD ["sh", "-c", "npx ng serve --host 0.0.0.0 --configuration \"$env\""]
