FROM frolvlad/alpine-glibc AS glibc

# install libstdc++ until rocksdb dep is removed
FROM frolvlad/alpine-gxx
COPY --from=glibc / /
RUN cd /tmp && \
    apk add --no-cache libunwind-dev icu-static binutils-gold libuuid && \
    apk add --no-cache --virtual=.build-dependencies wget ca-certificates tar xz && \
    wget "http://mirrors.kernel.org/ubuntu/pool/main/g/gcc-8/libstdc++6_8-20180414-1ubuntu2_amd64.deb" -O "libstdc++6.deb" && \
    ar x libstdc++6.deb && \
    tar --extract --xz --file="data.tar.xz" --strip=4 --directory=/usr/glibc-compat/lib/ ./usr/lib/x86_64-linux-gnu/libstdc++.so.6.0.25 && \
    ln -s libstdc++.so.6.0.25 /usr/glibc-compat/lib/libstdc++.so.6

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
