FROM frolvlad/alpine-glibc AS glibc

ARG VCS_REF
ARG BUILD_DATE

ENV RUST_BACKTRACE 1

RUN adduser --uid 1000 --shell /bin/sh --home /sunshine sunshine --disabled-password

COPY target/release/sunshine-node /usr/local/bin

USER sunshine

# check if the executable works in this container
RUN /usr/local/bin/sunshine-node --version

EXPOSE 30333 9933 9944
VOLUME ["/sunshine"]

ENTRYPOINT ["/usr/local/bin/sunshine-node"]
