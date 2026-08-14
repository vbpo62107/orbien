---
sidebar_position: 4
sidebar_label: Docker 安装
title: Docker 安装
---

# Docker 安装

推荐在生产环境中使用 Docker 或 Docker Compose 方式部署 Orbien，便于管理生命周期与自动重启。

镜像托管于 GitHub Container Registry：
- 服务端：`ghcr.io/orbien-org/orbien-server:latest`
- 客户端：`ghcr.io/orbien-org/orbien:latest`

---

## 服务端

### 准备配置文件

创建 `orbien-server.toml`：

```toml
# orbien-server.toml
bindAddr = "0.0.0.0"
bindPort = 9527

# 可选：HTTP 虚拟主机
# vhostHTTPPort = 80

# 可选：HTTPS 虚拟主机（SNI 透传）
# vhostHTTPSPort = 443

# 可选：Token 鉴权
# [auth]
# token = "your-secret-token"

# 可选：Web 管理面板
[webServer]
addr     = "0.0.0.0"   # 必须为 0.0.0.0，宿主机才能通过端口映射访问
port     = 8020
user     = "admin"
password = "changeme"
```

:::warning 注意
`webServer.addr` **必须**设置为 `0.0.0.0`，否则管理面板只监听容器内部，宿主机无法访问。
:::

### 方式一：docker run

```shell
docker run -d \
  --name orbien-server \
  --restart unless-stopped \
  -p 9527:9527 \
  -p 8020:8020 \
  -v "$PWD/orbien-server.toml:/etc/orbien/orbien-server.toml:ro" \
  ghcr.io/orbien-org/orbien-server:latest
```

若同时需要 HTTP/HTTPS 虚拟主机，追加端口映射：

```shell
  -p 80:80 \
  -p 443:443 \
```

### 方式二：Docker Compose

```yaml
# docker-compose.yaml
services:
  orbien-server:
    image: ghcr.io/orbien-org/orbien-server:latest
    container_name: orbien-server
    restart: unless-stopped
    ports:
      - "9527:9527"
      - "8020:8020"
      # - "80:80"
      # - "443:443"
    volumes:
      - ./orbien-server.toml:/etc/orbien/orbien-server.toml:ro
```

```shell
docker compose up -d
```

启动后，访问 `http://YOUR_SERVER_IP:8020` 打开 Web 管理面板。

---

## 客户端

### 准备配置文件

创建 `orbien.toml`：

```toml
# orbien.toml
serverAddr = "YOUR_SERVER_IP"
serverPort = 9527

# 若服务端开启了 Token 鉴权，需保持一致
# [auth]
# token = "your-secret-token"

[[proxies]]
name       = "mysql"
type       = "tcp"
localIP    = "127.0.0.1"
localPort  = 3306
remotePort = 6050
```

:::tip 容器内 127.0.0.1 的含义
容器内的 `127.0.0.1` 指向容器自身，**不是宿主机**。  
若要穿透宿主机上的服务，请将 `localIP` 改为宿主机 IP，或改用 **host 网络模式**（见下文）。
:::

### 方式一：docker run（桥接网络）

```shell
docker run -d \
  --name orbien \
  --restart unless-stopped \
  -v "$PWD/orbien.toml:/etc/orbien/orbien.toml:ro" \
  ghcr.io/orbien-org/orbien:latest
```

### 方式一变体：host 网络模式

使用 `--network host` 后，容器与宿主机共享网络栈，配置文件中可直接填写 `127.0.0.1` 访问宿主机本地服务：

```shell
docker run -d \
  --name orbien \
  --restart unless-stopped \
  --network host \
  -v "$PWD/orbien.toml:/etc/orbien/orbien.toml:ro" \
  ghcr.io/orbien-org/orbien:latest
```

> ⚠️ host 网络模式仅在 Linux 上有效；macOS / Windows 的 Docker Desktop 不支持此模式。

### 方式二：Docker Compose

```yaml
# docker-compose.yaml
services:
  orbien:
    image: ghcr.io/orbien-org/orbien:latest
    container_name: orbien
    restart: unless-stopped
    # 若需 host 网络，取消下行注释并删除 ports 配置
    # network_mode: host
    volumes:
      - ./orbien.toml:/etc/orbien/orbien.toml:ro
```

```shell
docker compose up -d
```

---

## 查看日志

```shell
# 实时查看服务端日志
docker logs -f orbien-server

# 实时查看客户端日志
docker logs -f orbien
```

## 升级镜像

```shell
docker compose pull && docker compose up -d
```

或使用 `docker run` 方式时：

```shell
docker stop orbien-server && docker rm orbien-server
docker pull ghcr.io/orbien-org/orbien-server:latest
# 重新执行 docker run 命令
```
