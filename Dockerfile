FROM docker.io/rustembedded/cross:armv7-unknown-linux-gnueabihf-0.2.1

# Force rebuild when our versions change...
RUN LIBNFC_VERSION=2b5ad9ce0be19fbca5abc04b4ee0b59fb612e590 && \
    LIBFREEFARE_VERSION=e95406c0d1b417ff6db7ff8ee95df1b5981ec7b5 && \
    LIBGATEKEEPER_VERSION=dc021430a68c66878bfe363fb85ce277e5a62501

# We need to clone up here because installing armhf openssl breaks ca-certificates (seriously...)
RUN git clone https://github.com/nfc-tools/libnfc && \
    git clone https://github.com/nfc-tools/libfreefare && \
    git clone https://github.com/Mstrodl/libgatekeeper

# Make sure we can use pkg-config...
RUN ln -s /usr/share/pkg-config-crosswrapper /usr/local/bin/arm-linux-gnueabihf-pkg-config

RUN dpkg --add-architecture armhf && \
    apt-get update && \
    apt-get install -y wget autoconf automake git libtool libssl-dev:armhf pkg-config libusb-dev:armhf uuid-dev:armhf openssl:armhf

COPY armv7-toolchain.cmake /

RUN cd libnfc && \
    git checkout $LIBNFC_VERSION && \
    mkdir build && \
    cd build && \
    cmake .. -DCMAKE_TOOLCHAIN_FILE=/armv7-toolchain.cmake \
       -DCMAKE_INSTALL_PREFIX=/usr/arm-linux-gnueabihf/ \
       -DBUILD_EXAMPLES=OFF \
       -DBUILD_UTILS=OFF && \
    make && \
    make install

#./configure --host=arm-linux-gnueabihf --prefix=/usr/arm-linux-gnueabihf/ && \
RUN cd libfreefare && \
    git checkout $LIBFREEFARE_VERSION && \
    mkdir build && \
    cd build && \
    cmake .. -DCMAKE_TOOLCHAIN_FILE=/armv7-toolchain.cmake \
       -DCMAKE_INSTALL_PREFIX=/usr/arm-linux-gnueabihf/ && \
    make && \
    make install

RUN cd libgatekeeper && \
    git checkout $LIBGATEKEEPER_VERSION && \
    mkdir build && \
    cd build && \
    cmake .. -DCMAKE_TOOLCHAIN_FILE=/armv7-toolchain.cmake \
        -DCMAKE_INSTALL_PREFIX=/usr/arm-linux-gnueabihf/ && \
    make && \
    make install
