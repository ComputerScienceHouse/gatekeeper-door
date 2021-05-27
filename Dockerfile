FROM docker.io/rustembedded/cross:armv7-unknown-linux-gnueabihf-0.2.1

# We need to clone up here because installing armhf openssl breaks ca-certificates (seriously...)
RUN curl -L https://github.com/nfc-tools/libnfc/releases/download/libnfc-1.8.0/libnfc-1.8.0.tar.bz2 | tar xvj && \
    curl -L https://github.com/nfc-tools/libfreefare/releases/download/libfreefare-0.4.0/libfreefare-0.4.0.tar.bz2 | tar xvj

# Make sure we can use pkg-config...
RUN ln -s /usr/share/pkg-config-crosswrapper /usr/local/bin/arm-linux-gnueabihf-pkg-config

RUN dpkg --add-architecture armhf && \
    apt-get update && \
    apt-get install -y wget autoconf automake git libtool libssl-dev:armhf pkg-config libusb-dev:armhf uuid-dev:armhf openssl:armhf

COPY armv7-toolchain.cmake /

RUN cd libnfc-1.8.0 && \
    mkdir build && \
    cd build && \
    cmake .. -DCMAKE_TOOLCHAIN_FILE=/armv7-toolchain.cmake \
       -DCMAKE_INSTALL_PREFIX=/usr/arm-linux-gnueabihf/ \
       -DBUILD_EXAMPLES=OFF \
       -DBUILD_UTILS=OFF && \
    make && \
    make install

# I wish libfreefare 0.4.0's cmake files actually worked... Anyways, automake is
# A fake build system, so we have to tell it that cross compiled targets might actually
# support standard libc malloc... Ugh.
RUN cd libfreefare-0.4.0 && \  
    sed -i 's/ac_cv_func_malloc_0_nonnull=no/ac_cv_func_malloc_0_nonnull=yes/' configure && \
    sed -i 's/ac_cv_func_realloc_0_nonnull=no/ac_cv_func_realloc_0_nonnull=yes/' configure && \
    ./configure --host=arm-linux-gnueabihf --prefix=/usr/arm-linux-gnueabihf && \
    make && \
    make install

COPY libgatekeeper /libgatekeeper

RUN cd libgatekeeper && \
    mkdir build && \
    cd build && \
    cmake .. -DCMAKE_TOOLCHAIN_FILE=/armv7-toolchain.cmake \
        -DCMAKE_INSTALL_PREFIX=/usr/arm-linux-gnueabihf/ && \
    make && \
    make install
