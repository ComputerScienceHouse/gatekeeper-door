FROM rustembedded/cross:armv7-unknown-linux-gnueabihf

RUN apt-get install -y wget autoconf automake git libtool libssl-dev pkg-config libusb-dev uuid-dev openssl

RUN wget https://github.com/nfc-tools/libnfc/releases/download/libnfc-1.7.1/libnfc-1.7.1.tar.bz2 && \
    tar xf libnfc-1.7.1.tar.bz2 && \
    cd libnfc-1.7.1 && \
    mkdir build && \
    cd build && \
    cmake .. && \
    make && \
    make install

RUN wget https://github.com/nfc-tools/libfreefare/releases/download/libfreefare-0.4.0/libfreefare-0.4.0.tar.bz2 && \
    tar xf libfreefare-0.4.0.tar.bz2 && \
    cd libfreefare-0.4.0 && \
    ./configure && \
    make && \
    make install

RUN git clone https://github.com/nicholastmosher/libgatekeeper && \
    cd libgatekeeper && \
    mkdir build && \
    cd build && \
    cmake .. && \
    make && \
    make install && \
    chmod +x /usr/local/lib/libgatekeeper.so
