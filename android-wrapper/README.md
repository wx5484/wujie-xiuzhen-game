# Android Web 容器

这个目录只保留轻量 WebView 外壳。游戏 UI、GM UI、响应式布局和 PWA 离线能力都由 Web 负责。

后续发布到生产时建议：

- 将 `default_base_url` 改为正式域名
- 关闭 `usesCleartextTraffic`
- 增加安全域名白名单
- 根据 PWA 成熟度评估切换为 Trusted Web Activity
