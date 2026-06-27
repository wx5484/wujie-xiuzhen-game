# 无界修真：斩妖录

一个 Rust + Vue 实现的网页文字修真游戏，包含玩家端、GM 后台、PostgreSQL 数据库迁移、Docker 部署和 Android WebView 外壳骨架。它不兼容旧账号、旧角色、旧数据库，也不保留旧 Socket.IO 协议。

当前版本：**0.2.13**。每个版本的功能变化、数据库迁移和验证状态记录在 [CHANGELOG.md](./CHANGELOG.md)，机器可读版本号记录在 [VERSION](./VERSION)。

## 当前落地范围

- Rust workspace：`cq-server`、`cq-domain`、`cq-game`、`cq-db`、`cq-protocol`、`cq-admin`
- 后端：Axum HTTP、标准 WebSocket JSON 协议、PostgreSQL/sqlx、健康检查、静态文件服务
- 数据库：全新 PostgreSQL schema，核心账号、角色、背包、怪物、邮件、行会、交易、活动、技能、任务、宠物、法宝、修炼、会员自动用药、PK 设置、仿玩家 bot、审计、备份表
- 玩家闭环：注册/登录、建角、数据库地图移动、WebSocket 认证、打怪、经验金币、掉落入包、每日体力与疲劳结算、消耗品使用、补给商店、元宝商城、会员自动用药与自动提取设置、装备与技能独立页、装备/卸下、仓库、职业技能学习、主动技能自动释放开关、技能书/杂项拆解、任务领取、行会申请/审批/捐献、Boss 解锁法宝/境界/宠物培养、万象铸体、凡尘界/修真界/飞升界/终极探索区域 1-500 级地图网络、奇遇弹窗、同屏玩家 PK、沙巴克状态、活动积分、挂机结算、邮件附件领取、关于页、角色位置和在线状态持久化
- 经济与安全：寄售在元宝玩法页展示并作为全服公开市场，支持金币/元宝任选标价、寄售上架费、成交税、卖家税后实收、绑定资源保护、元宝商品独立扣费、PK 默认关闭、探索当前区域不会自动攻击玩家
- 行会成长：默认行会作为阵营入口，成员每日可做巡猎、补给建设、首领演武，获得个人贡献并推进行会资金、等级和当日目标
- 数值平衡：任务、怪物、装备、技能和掉落按 1-500 级阶段推进；普通怪、精英怪、地图 Boss、世界 Boss 使用可通关的分段曲线，野外装备掉落统一走 N-2/N-1/N/N+1 的全局三步判定；材料以绑定日常/行会任务和挑战副本稳定产出；Bot PK 按等级差提供合理反击压力
- 装备体系：1-17 阶装备按单套模板重构，只有 T3/T6/T9/T12/T15 拥有套装效果，其余为高面板散件；T6+ 装备开放受上限约束的高级百分比词条；终极世界首领产出主宰套装，8 件套可无视疲劳收益衰减、移除探索间隔并触发天罚
- GM 可见性与操作：环境变量账号密码登录、仪表盘、注册账号、角色资产、邮件概览、物品模板、怪物模板、仿玩家 bot 批量调度、Bot 增删、自动/手动脚本推进、测试工具、展示设置、审计日志、健康检查、GM 发邮件、角色资产调整、活动开关
- Web：Vue 3 + TypeScript + Pinia + Vite PWA，玩家端与 GM 后台分离
- 部署：Dockerfile、Docker Compose、Caddy 反代、一次性迁移服务、备份脚本
- Android：轻量 WebView 外壳骨架，可在正式域名稳定后替换为 TWA

## 版本信息

- 当前版本：`0.2.13`
- 版本号来源：根目录 [VERSION](./VERSION)、Rust crate `Cargo.toml`、玩家端 `web/player/package.json`、GM 后台 `web/admin/package.json`
- 更新记录：[CHANGELOG.md](./CHANGELOG.md)

发布或合并新版本时需要同步更新：

1. `VERSION`
2. `CHANGELOG.md`
3. Rust crate 的 `Cargo.toml` 版本
4. `web/player/package.json` 与 `web/player/package-lock.json`
5. `web/admin/package.json` 与 `web/admin/package-lock.json`

## 技术栈与依赖

- Rust 1.78+，Axum、Tokio、sqlx、PostgreSQL
- Vue 3、TypeScript、Pinia、Vite、PWA
- Node.js 24 推荐，生产镜像使用 `node:24-bookworm`
- PostgreSQL 16，开发和生产 compose 默认使用 `postgres:16-alpine`
- Docker 与 Docker Compose v2，用于本地联调、迁移和生产部署

## 目录结构

```text
.
├── crates/              # Rust workspace：领域、游戏逻辑、数据库、协议、服务和 GM 能力
├── migrations/          # PostgreSQL schema 与数据迁移
├── web/player/          # 玩家端 Vue/Vite 应用
├── web/admin/           # GM 后台 Vue/Vite 应用
├── docker/              # Caddy、迁移、备份和恢复脚本
├── scripts/             # 部署与验证脚本
├── android-wrapper/     # Android WebView 外壳骨架
├── docker-compose.yml   # 本地 compose
└── docker-compose.prod.yml
```

## 玩家玩法要点

- 主线成长上限为 500 级，地图按凡尘界、修真界、飞升界、终极探索区域推进；青牛城、天水古城、破冰前哨站、虚空要塞、混沌庇护所和太初远征营地是安全枢纽。卡图时优先检查等级、装备强化、技能等级、宠物、法宝、境界、万象铸体和药品。
- 职业体系为剑修、法修、魂修三系，每系 10 个技能，保留 100 级技能上限和累计熟练度升级方式；主动技能会在扫荡和 PK 中自动释放并消耗魔法，玩家可以在技能详情里关闭单个主动技能的自动释放。
- 每日体力上限为 5000 点，每天 0 点按 Asia/Shanghai 自然日自动回满。体力大于 0 时每次击杀怪物扣 1 点并获得 100% 经验、100% 金币和正常掉落；体力为 0 后仍可继续探索或挂机，但经验降至 5%，金币降至 2%，装备与材料掉率为 0%。
- 技能获取按阶段推进：1-4 技能在天水古城的天水书院购买，5-6 技能在虚空要塞的虚空市集购买，7-8 技能由塔顶封印的怨魂聚合体中概率掉落，9-10 技能由判官殿的阎罗判官以 0.01% 概率掉落；通用特殊被动在混沌庇护所寻找不动冥王，消耗技能书残页逐级提升。
- 打坐修炼已迁移到新地图节点：破冰前哨站的炼狱用于等级修炼，混沌庇护所的虚境用于指定已学习技能研修。
- 背包支持装备一键拆解和杂项拆解：装备拆解主要获得炼器石，高阶装备可能额外获得鸿蒙石；杂项拆解用于处理技能书、材料或消耗品，使用前应确认物品不再需要。
- 法宝升级金币为 `10000 + 当前等级^3 * 90`，宠物升级金币为 `15000 + 当前等级^3 * 100`，境界突破金币为 `100000 + 当前层级^3 * 12000`；材料消耗仍按长期成长曲线计算。
- 三大挑战区域独立于主线挂机：探索秘境可在任意城市或安全区开启，最高 40 层；无尽塔位于天水古城，每 1 小时连续扫荡一次，从 1 层挑战到失败或当前版本 100 层边界；世界首领“万古渊魔”位于虚空要塞，胜利后 4 小时刷新并必定掉落 1 件主宰套装随机部件。
- 万象铸体需要第一次进入星际观测台后开启；开启后可在界限突破页批量提取背包中 1 至所选阶级的装备获得灵韵，主宰装备提取为 100 点灵韵；Lv.500 后升级可能失败，Lv.1000 获得完整肉身属性。
- 野外装备掉落统一为三步判定：50% 触发装备掉落；再按当前地图最高掉落阶 N 的 50%/30%/19%/1% 权重产出 N-2/N-1/N/N+1 阶；最后按 50% 防具、49% 首饰、1% 武器抽取部位。
- 移动、打怪和挂机有极低概率触发奇遇，玩家可在弹窗中选择处理方式；奇遇每次只结算一个奖励或一个惩罚，角色达到 500 级后不再触发。
- 会员自动拆解低阶装备和自动提取灵韵只能任选一个；万象铸体开启后，自动提取命中时装备不会进入背包，而是直接转入万象铸体灵韵。

## 本地开发

```bash
cp .env.example .env
docker compose up -d postgres migrate
cargo run -p cq-server
```

玩家端：

```bash
cd web/player
npm ci
npm run dev
```

GM 后台：

```bash
cd web/admin
npm ci
npm run dev
```

## 验证命令

提交前建议至少跑一次：

```bash
cargo fmt --all --check
cargo check --workspace --all-targets
cargo test --workspace
```

前端构建：

```bash
cd web/player
npm ci
npm run build

cd ../admin
npm ci
npm run build
```

仓库包含 GitHub Actions 工作流，会在 push 到 `main` 和 pull request 时执行 Rust 格式检查、`cargo check` 和 `cargo test`。

## 环境变量与安全

- 只提交 `.env.example`，不要提交真实 `.env`、数据库密码、GM 密码或生产域名密钥。
- 首次生产部署必须修改 `POSTGRES_PASSWORD`、`ADMIN_BOOTSTRAP_USER` 和 `ADMIN_BOOTSTRAP_PASSWORD`。
- 公开服务建议使用 HTTPS，并按需设置 `ADMIN_IP_ALLOWLIST` 限制 GM 后台访问来源。
- `data/`、`logs/`、`tmp/`、前端 `node_modules/` 和 `dist/` 都是运行或构建产物，已在 `.gitignore` 中排除。

## 后端部署到 VPS

下面以一台 Ubuntu/Debian VPS 为例。生产推荐使用 `docker-compose.prod.yml`，它包含 `postgres + migrate + server + caddy`：数据库、迁移、后端、玩家端、GM 后台都在同一台机器上运行。

### 1. 准备服务器

```bash
sudo apt update
sudo apt install -y ca-certificates curl git
curl -fsSL https://get.docker.com | sudo sh
sudo usermod -aG docker "$USER"
```

重新登录 SSH 后确认：

```bash
docker version
docker compose version
```

如果 VPS 内存小于 4 GB，建议加 swap：

```bash
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
```

### 2. 上传代码并进入项目

```bash
git clone <your-repo-url> cq-rs
cd cq-rs
```

如果不是 Git 部署，也可以把仓库根目录整个上传到 VPS，例如 `/opt/cq-rs`。

Ubuntu 24 服务器也可以在项目根目录直接执行一键部署脚本：

```bash
DOMAIN=game.example.com bash scripts/deploy-ubuntu24.sh
```

脚本会检查 Docker/Compose，缺少 `.env` 时复制模板并生成随机数据库密码和 GM 密码，随后构建 `cq-rs:latest` 并启动 `docker-compose.prod.yml`。如果暂时没有域名，也可以不传 `DOMAIN`，之后手动修改 `docker/Caddyfile` 和 `.env` 里的 `PUBLIC_BASE_URL`。

### 3. 配置环境变量

```bash
cp .env.example .env
```

编辑 `.env`，生产至少改这些值：

```env
APP_ENV=production
PORT=3000
PUBLIC_BASE_URL=https://game.example.com
DATABASE_URL=postgres://cq:strong_postgres_password@postgres:5432/cq
POSTGRES_PASSWORD=strong_postgres_password
ADMIN_PATH=admin
ADMIN_BOOTSTRAP_USER=admin
ADMIN_BOOTSTRAP_PASSWORD=strong_admin_password
SESSION_TTL_MIN=1440
RUST_LOG=info,cq_server=info
PUBLIC_DIR=/app/public
BACKUP_DIR=/app/data/backup
JSON_BODY_LIMIT_BYTES=262144
```

注意：

- `DATABASE_URL` 里的主机名使用 `postgres`，这是 compose 内部服务名；`docker-compose.prod.yml` 会按 `POSTGRES_PASSWORD` 显式覆盖容器内后端的 `DATABASE_URL`，避免误连容器内 `localhost`。
- `POSTGRES_PASSWORD` 要和 `DATABASE_URL` 中的密码一致。
- `ADMIN_BOOTSTRAP_USER` 和 `ADMIN_BOOTSTRAP_PASSWORD` 用于 GM 后台登录，不要使用默认值。
- `.env` 不要提交到仓库。

### 4. 配置域名和 HTTPS

把域名 A 记录指向 VPS 公网 IP，然后改 [docker/Caddyfile](./docker/Caddyfile)：

```caddyfile
game.example.com {
  encode zstd gzip
  reverse_proxy server:3000
}
```

开放 VPS 防火墙端口：

```bash
sudo ufw allow OpenSSH
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable
```

### 5. 构建镜像并启动

```bash
docker build -t cq-rs:latest .
docker compose -f docker-compose.prod.yml up -d
```

启动过程会自动执行：

1. 启动 PostgreSQL
2. `migrate` 服务按顺序执行 `migrations/*.sql`
3. 启动 Rust 后端
4. Caddy 反代并自动处理 HTTPS

当前 Dockerfile 会把玩家端打包到 `/app/public`，把 GM 后台打包到 `/app/public/admin`。部署后访问：

- 玩家端：`https://game.example.com/`
- GM 后台：`https://game.example.com/admin/`
- 健康检查：`https://game.example.com/api/healthz`
- 就绪检查：`https://game.example.com/api/readyz`

### 6. 查看状态和日志

```bash
docker compose -f docker-compose.prod.yml ps
docker compose -f docker-compose.prod.yml logs -f server
docker compose -f docker-compose.prod.yml logs -f migrate
docker compose -f docker-compose.prod.yml logs -f postgres
```

如果迁移失败，先看 `migrate` 日志。修复 SQL 或环境变量后重新执行：

```bash
docker compose -f docker-compose.prod.yml up migrate
docker compose -f docker-compose.prod.yml up -d server
```

### 7. 更新版本

```bash
git pull
docker build -t cq-rs:latest .
docker compose -f docker-compose.prod.yml up -d
```

`migrate` 服务会检查 `schema_migrations`，已经执行过的 SQL 文件会跳过。

发布前请先查看 [CHANGELOG.md](./CHANGELOG.md)，确认本次版本号、功能变化、数据库迁移和验证状态已经记录。服务健康检查里的版本来自 `cq-server` 的 Cargo 版本。

### 8. 备份和恢复

备份：

```bash
docker compose -f docker-compose.prod.yml exec server sh /app/docker/backup.sh
```

如果使用当前镜像没有内置 `pg_dump`，可从宿主机执行：

```bash
mkdir -p data/backup
docker compose -f docker-compose.prod.yml exec -T postgres \
  pg_dump -U cq -d cq --format=custom > data/backup/cq-$(date -u +%Y%m%dT%H%M%SZ).dump
```

恢复前先停止后端：

```bash
docker compose -f docker-compose.prod.yml stop server
docker compose -f docker-compose.prod.yml exec -T postgres \
  pg_restore --clean --if-exists --no-owner -U cq -d cq < data/backup/your-backup.dump
docker compose -f docker-compose.prod.yml up -d server
```

### 9. 无域名临时部署

只想用 `http://VPS_IP:3000` 临时试跑：

```bash
cp .env.example .env
docker compose up -d --build
```

这种模式不带 HTTPS，不建议正式开放给玩家。

## License

MIT，见 [LICENSE](./LICENSE)。
