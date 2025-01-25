FROM python:3.10-bullseye

WORKDIR /home

RUN apt update && apt upgrade -y
RUN apt install -y --no-install-recommends \
        ccache \
        cmake \
        device-tree-compiler \
        dfu-util \
        file \
        g++-multilib \
        gcc \
        gcc-multilib \
        git \
        gperf \
        libmagic1 \
        libsdl2-dev \
        make \
        ninja-build \
        wget \
        xz-utils

# CMake 3.20 or newer required
RUN apt remove -y --purge --auto-remove cmake
RUN cd /tmp && \
    wget https://cmake.org/files/v3.28/cmake-3.28.1.tar.gz && \
    tar -zxvf cmake-3.28.1.tar.gz && \
    cd cmake-3.28.1 && \
    ./bootstrap && \
    make -j$(nproc) && \
    make install

# Zephyr-specifics
RUN pip install west

## Zephyr SDK
RUN wget https://github.com/zephyrproject-rtos/sdk-ng/releases/download/v0.16.8/zephyr-sdk-0.16.8_linux-x86_64.tar.xz && \
    wget -O - https://github.com/zephyrproject-rtos/sdk-ng/releases/download/v0.16.8/sha256.sum | shasum --check --ignore-missing && \
    tar xvf zephyr-sdk-0.16.8_linux-x86_64.tar.xz

RUN cp -r zephyr-sdk-0.16.8 /opt && \
    /opt/zephyr-sdk-0.16.8/setup.sh -c

COPY zephyr/scripts/requirements* .
RUN pip install -r requirements.txt
