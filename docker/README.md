# Docker 运维

- `docker-compose.yml` 用于本地联调。
- `docker-compose.prod.yml` 用于带 Caddy 的生产拓扑。
- `backup.sh` 使用 `pg_dump --format=custom` 输出可恢复备份。
- `restore.sh` 使用 `pg_restore --clean --if-exists` 恢复。

生产环境请通过 `.env` 注入密钥，不要把真实 `.env` 放进镜像或仓库。
