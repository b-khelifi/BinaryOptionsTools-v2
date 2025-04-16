FROM rust:1.76-slim

# Install build dependencies
RUN apt-get update && apt-get install -y \
    curl unzip build-essential pkg-config cmake \
    python3 python3-pip python3-venv \
    clang git libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install maturin
RUN pip install maturin

# Set environment variables
ENV ANDROID_NDK_VERSION=r26d
ENV ANDROID_SDK_ROOT=/opt/android-sdk
ENV ANDROID_NDK_HOME=$ANDROID_SDK_ROOT/ndk/$ANDROID_NDK_VERSION
ENV PATH="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH"

# Download Android NDK
RUN mkdir -p $ANDROID_SDK_ROOT/ndk && \
    curl -L https://dl.google.com/android/repository/android-ndk-${ANDROID_NDK_VERSION}-linux.zip -o ndk.zip && \
    unzip ndk.zip -d $ANDROID_SDK_ROOT/ndk && \
    rm ndk.zip

# Add Rust target (e.g., aarch64 for 64-bit ARM Android)
RUN rustup target add aarch64-linux-android

# Set working dir
WORKDIR /app

# Copy project
COPY . .

# Set linker for Android cross-compilation
ENV CC_aarch64_linux_android=aarch64-linux-android21-clang
ENV CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=aarch64-linux-android21-clang

# Default command to build .whl
CMD ["maturin", "build", "--release", "--target", "aarch64-linux-android", "--interpreter", "python3"]
