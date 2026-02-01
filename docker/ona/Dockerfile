FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    build-essential \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN git clone https://github.com/opennars/OpenNARS-for-Applications.git /ona && \
    cd /ona && \
    make -j4

WORKDIR /ona
CMD ["./ONA", "shell"]
