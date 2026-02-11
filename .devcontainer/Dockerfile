FROM rust:1.93-bookworm

ARG USERNAME
ARG USER_UID
ARG USER_GID=$USER_UID

RUN groupadd -g ${USER_GID} ${USERNAME} \
  && useradd -u ${USER_UID} -g ${USER_GID} -m -s /bin/bash ${USERNAME}

RUN apt-get update \
  && export DEBIAN_FRONTEND=noninteractive \
  && apt-get install -y --no-install-recommends git openssh-client

RUN mkdir /workspaces && chown ${USERNAME}:${USERNAME} /workspaces

USER ${USERNAME}

RUN rustup component add rustfmt clippy
