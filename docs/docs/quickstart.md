---
sidebar_position: 3
title: 快速开始
---

# 快速开始

本文介绍如何在 5 分钟内完成 Orbien 的基础部署，将内网的 MySQL（端口 `3306`）暴露到公网服务器的 `6050` 端口。

## 前提条件

- 一台**公网服务器**（云主机、VPS 等），用于运行 `orbien-server`
- 一台**内网机器**，运行需要穿透的服务（本例为 MySQL）
- 在 [下载页面](download.mdx) 下载对应平台的二进制压缩包并解压

:::tip 图形界面
如果不习惯命令行操作，可以使用 [Orbien Desktop](download.mdx) 桌面客户端完成客户端配置。注意：桌面端内存占用较高，内存受限的环境建议使用 CLI。
:::

---

## 第一步：部署服务端

在**公网服务器**上创建配置文件 `orbien-server.toml`：

```toml
# orbien-server.toml

# 监听所有网卡
bindAddr = "0.0.0.0"
# 客户端连接端口
bindPort = 9527

# 可选：开启 Token 鉴权，防止非授权客户端连接
# [auth]
# token = "your-secret-token"

# 可选：开启 Web 管理面板
# [webServer]
# addr = "0.0.0.0"
# port = 8020
# user = "admin"
# password = "changeme"
```

启动服务端：

```shell
./orbien-server -c orbien-server.toml
```

确认控制台输出类似以下内容，说明服务端已就绪：

```
[INFO] orbien-server started, listening on 0.0.0.0:9527
```

:::tip 防火墙
确保公网服务器的安全组 / 防火墙已放行 `9527` 端口（TCP）；若开启了 Web 面板，同时放行 `8020`。
:::

---

## 第二步：配置并启动客户端

在**内网机器**上创建配置文件 `orbien.toml`：

```toml
# orbien.toml

# 公网服务器地址
serverAddr = "YOUR_SERVER_IP"
serverPort = 9527

# 若服务端开启了 Token 鉴权，需保持一致
# [auth]
# token = "your-secret-token"

# 穿透规则示例：将本机 MySQL 映射到服务器的 6050 端口
[[proxies]]
name = "mysql"
type = "tcp"
localIP  = "127.0.0.1"
localPort  = 3306
remotePort = 6050
```

启动客户端：

```shell
./orbien -c orbien.toml
```

---

## 第三步：验证连接

在任意外网机器上，使用 MySQL 客户端连接公网服务器的 `6050` 端口：

```shell
mysql -h YOUR_SERVER_IP -P 6050 -u root -p
```

连接成功即说明穿透已生效。

---

## 常用配置示例

### 穿透 SSH

```toml
[[proxies]]
name = "ssh"
type = "tcp"
localIP   = "127.0.0.1"
localPort  = 22
remotePort = 6022
```

连接方式：

```shell
ssh -p 6022 user@YOUR_SERVER_IP
```

### 穿透 HTTP 服务

```toml
[[proxies]]
name       = "web"
type       = "http"
localIP    = "127.0.0.1"
localPort  = 8080
customDomains = ["example.com"]
```

> 需要服务端配置 `vhostHTTPPort = 80` 并将域名解析到公网服务器。

---

## 下一步

- [Docker 安装](docker.md)：推荐在生产环境中使用容器部署
- [代理类型](features/)：了解 TCP / UDP / HTTP / HTTPS 代理的详细配置
- [传输层](transport/)：选择适合网络环境的传输协议（QUIC / KCP / WebSocket）
