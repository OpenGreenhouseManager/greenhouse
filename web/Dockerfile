FROM --platform=$BUILDPLATFORM node:23.6.1-bullseye-slim as builder

RUN mkdir /greenhouse_web
WORKDIR /greenhouse_web

RUN npm install -g @angular/cli@18

COPY package.json package-lock.json ./
RUN npm ci

COPY . .
CMD ["ng", "serve", "--host", "0.0.0.0"]

FROM builder as dev-envs

RUN apt-get update && \
    apt-get install -y --no-install-recommends git

RUN useradd -s /bin/bash -m vscode && \
groupadd docker && \
usermod -aG docker vscode 

# install Docker tools (cli, buildx, compose)
COPY --from=builder / /
EXPOSE 4200

CMD ["ng", "serve", "--host", "0.0.0.0"]