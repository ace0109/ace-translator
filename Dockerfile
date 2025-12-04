# 使用官方 Rust 镜像
FROM rust:1.88

# 设置环境变量避免交互式安装
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=Asia/Shanghai
ENV XWIN_CACHE_DIR=/opt/xwin-cache
ENV XWIN_ARCH=x86_64,x86,aarch64

# 配置镜像源并安装依赖
RUN apt-get update && apt-get install -y --fix-missing \
    curl \
    ca-certificates \
    nsis \
    clang \
    lld \
    llvm \
    && rm -rf /var/lib/apt/lists/*

# 创建缓存目录
RUN mkdir -p /opt/xwin-cache

# 安装 cargo-xwin 和添加 Rust Windows 交叉编译目标
RUN cargo install --locked cargo-xwin && \
    rustup target add x86_64-pc-windows-msvc && \
    rustup target add i686-pc-windows-msvc && \
    rustup target add aarch64-pc-windows-msvc

# 安装 Node.js 和 pnpm
RUN curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - \
    && apt-get install -y nodejs \
    && npm install -g pnpm

# 设置工作目录
WORKDIR /app

# 暴露 target 目录
VOLUME ["/app/src-tauri/target"]

# 默认命令，保持容器运行
CMD ["bash"]