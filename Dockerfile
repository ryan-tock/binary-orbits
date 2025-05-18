FROM amazonlinux:2

# Install build dependencies
RUN yum -y update && \
    yum -y install perl gcc openssl-devel bzip2-devel libffi-devel zlib-devel make wget tar gzip zip

# Set Python version
ENV PYTHON_VERSION=3.13.3
ENV SSL_VERSION=1.1.1

# Download and extract SSL
WORKDIR /usr/src
RUN wget https://www.openssl.org/source/openssl-${SSL_VERSION}v.tar.gz && \
    tar -xzvf openssl-${SSL_VERSION}v.tar.gz
WORKDIR /usr/src/openssl-${SSL_VERSION}v
RUN ./config --prefix=/usr/local/ssl --openssldir=/usr/local/ssl shared zlib
RUN make install

# Download and extract Python source
WORKDIR /usr/src
RUN wget https://www.python.org/ftp/python/$PYTHON_VERSION/Python-$PYTHON_VERSION.tgz && \
    tar xzf Python-$PYTHON_VERSION.tgz

ENV LD_LIBRARY_PATH="/usr/local/ssl/lib"

# Build and install Python
WORKDIR /usr/src/Python-$PYTHON_VERSION
RUN ./configure --enable-optimizations --with-ensurepip=install --with-openssl=/usr/local/ssl --with-openssl-rpath=/usr/local/ssl/lib
RUN make altinstall

# Cleanup
RUN rm -rf /usr/src/Python-$PYTHON_VERSION* /urs/src/openssl-${SSL_VERSION}v*

# Install scipy and clean unused modules
WORKDIR /root
RUN mkdir python
RUN pip3.13 install --upgrade pip
RUN python3.13 -m pip install --target python scipy
WORKDIR /root/python/scipy
RUN rm -r stats signal io interpolate integrate
WORKDIR /root
RUN zip -r scipy-layer.zip python