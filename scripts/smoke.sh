#!/bin/sh
set -eu

BASE_URL="${BASE_URL:-http://127.0.0.1:3000}"
ENV_ADMIN_TOKEN=""
if [ -f .env ]; then
  ENV_ADMIN_TOKEN="$(awk -F= '/^ADMIN_BOOTSTRAP_PASSWORD=/{print substr($0, index($0, "=") + 1)}' .env | tail -n 1 | tr -d '\r')"
fi
ADMIN_TOKEN="${ADMIN_TOKEN:-${ENV_ADMIN_TOKEN:-change_me}}"
STAMP="$(date +%s)"
USERNAME="${SMOKE_USERNAME:-smoke_$STAMP}"
PASSWORD="${SMOKE_PASSWORD:-smoke123456}"
CHARACTER_NAME="${SMOKE_CHARACTER_NAME:-t$STAMP}"
SCRIPT_DIR="$(CDPATH= cd "$(dirname "$0")" && pwd)"

need_node() {
  if ! command -v node >/dev/null 2>&1; then
    echo "node is required for JSON parsing" >&2
    exit 1
  fi
}

json_pick() {
  node -e '
const fs = require("fs");
const data = JSON.parse(fs.readFileSync(0, "utf8"));
const path = process.argv[1].split(".");
let value = data;
for (const part of path) {
  value = /^[0-9]+$/.test(part) ? value?.[Number(part)] : value?.[part];
}
if (value === undefined || value === null) process.exit(2);
process.stdout.write(String(value));
' "$1"
}

json_find_mail_id() {
  node -e '
const fs = require("fs");
const data = JSON.parse(fs.readFileSync(0, "utf8"));
const mail = (data.data.mails || []).find((item) =>
  item.attachments && item.attachments.some((att) => !att.claimed)
);
if (!mail) process.exit(2);
process.stdout.write(String(mail.id));
'
}

json_find_bag_item_id() {
  node -e '
const fs = require("fs");
const data = JSON.parse(fs.readFileSync(0, "utf8"));
const item = (data.data.items || []).find((entry) => entry.location === "bag" && !entry.bind && entry.template_slot)
  || (data.data.items || []).find((entry) => entry.location === "bag" && !entry.bind);
if (!item) process.exit(2);
process.stdout.write(String(item.id));
'
}

json_find_template_item_id() {
  node -e '
const fs = require("fs");
const data = JSON.parse(fs.readFileSync(0, "utf8"));
const templateId = process.argv[1];
const location = process.argv[2] || "bag";
const item = (data.data.items || []).find((entry) => entry.location === location && entry.template_id === templateId);
if (!item) process.exit(2);
process.stdout.write(String(item.id));
' "$1" "$2"
}

json_find_detail_template_item_id() {
  node -e '
const fs = require("fs");
const data = JSON.parse(fs.readFileSync(0, "utf8"));
const templateId = process.argv[1];
const item = (data.data.inventory.items || []).find((entry) => entry.template_id === templateId);
if (!item) process.exit(2);
process.stdout.write(String(item.id));
' "$1"
}

json_find_equipment_item_id() {
  node -e '
const fs = require("fs");
const data = JSON.parse(fs.readFileSync(0, "utf8"));
const item = (data.data.items || []).find((entry) => entry.location === "bag" && entry.template_slot);
if (!item) process.exit(2);
process.stdout.write(String(item.id));
'
}

json_find_own_consignment_id() {
  node -e '
const fs = require("fs");
const data = JSON.parse(fs.readFileSync(0, "utf8"));
const item = (data.data.consignments || []).find((entry) => entry.mine);
if (!item) process.exit(2);
process.stdout.write(String(item.id));
'
}

curl_json() {
  curl -fsS "$@"
}

post_json() {
  curl_json -H "content-type: application/json" -X POST "$@"
}

auth_get() {
  curl_json -H "authorization: Bearer $TOKEN" "$@"
}

auth_post() {
  post_json -H "authorization: Bearer $TOKEN" "$@"
}

need_node

echo "1 health"
curl_json "$BASE_URL/api/healthz" >/dev/null
curl_json "$BASE_URL/api/readyz" >/dev/null

echo "2 auth"
post_json "$BASE_URL/api/auth/register" \
  -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\",\"email\":null}" >/dev/null
LOGIN_BODY="$(post_json "$BASE_URL/api/auth/login" \
  -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\",\"device\":\"smoke\"}")"
TOKEN="$(printf "%s" "$LOGIN_BODY" | json_pick data.token)"

echo "3 character"
CHAR_BODY="$(auth_post "$BASE_URL/api/characters" \
  -d "{\"name\":\"$CHARACTER_NAME\",\"class\":\"warrior\"}")"
CHARACTER_ID="$(printf "%s" "$CHAR_BODY" | json_pick data.character.id)"

echo "4 game read APIs"
auth_get "$BASE_URL/api/game/bootstrap?character_id=$CHARACTER_ID" >/dev/null
OVERVIEW_BODY="$(auth_get "$BASE_URL/api/game/overview?character_id=$CHARACTER_ID")"
printf "%s" "$OVERVIEW_BODY" | json_pick data.systems.cultivation.realm >/dev/null
auth_get "$BASE_URL/api/game/room?character_id=$CHARACTER_ID" >/dev/null
auth_post "$BASE_URL/api/game/move" \
  -d "{\"character_id\":$CHARACTER_ID,\"direction\":\"比奇平原\"}" >/dev/null
auth_post "$BASE_URL/api/game/attack" \
  -d "{\"character_id\":$CHARACTER_ID,\"target_id\":0}" >/dev/null
auth_post "$BASE_URL/api/game/explore" \
  -d "{\"character_id\":$CHARACTER_ID}" >/dev/null
auth_post "$BASE_URL/api/game/tower/challenge" \
  -d "{\"character_id\":$CHARACTER_ID}" >/dev/null
auth_post "$BASE_URL/api/game/move" \
  -d "{\"character_id\":$CHARACTER_ID,\"direction\":\"毒蛇山谷\"}" >/dev/null
auth_post "$BASE_URL/api/game/move" \
  -d "{\"character_id\":$CHARACTER_ID,\"direction\":\"山谷深处\"}" >/dev/null
auth_post "$BASE_URL/api/game/move" \
  -d "{\"character_id\":$CHARACTER_ID,\"direction\":\"盟重区域\"}" >/dev/null
auth_post "$BASE_URL/api/game/world-boss/challenge" \
  -d "{\"character_id\":$CHARACTER_ID}" >/dev/null
auth_post "$BASE_URL/api/game/move" \
  -d "{\"character_id\":$CHARACTER_ID,\"direction\":\"毒蛇山谷\"}" >/dev/null
auth_post "$BASE_URL/api/game/move" \
  -d "{\"character_id\":$CHARACTER_ID,\"direction\":\"山谷深处\"}" >/dev/null
auth_post "$BASE_URL/api/game/move" \
  -d "{\"character_id\":$CHARACTER_ID,\"direction\":\"盟重区域\"}" >/dev/null
auth_post "$BASE_URL/api/game/move" \
  -d "{\"character_id\":$CHARACTER_ID,\"direction\":\"盟重土城\"}" >/dev/null
auth_post "$BASE_URL/api/game/move" \
  -d "{\"character_id\":$CHARACTER_ID,\"direction\":\"苍月岛\"}" >/dev/null
auth_post "$BASE_URL/api/game/secret-realm/explore" \
  -d "{\"character_id\":$CHARACTER_ID}" >/dev/null
auth_get "$BASE_URL/api/game/skills?character_id=$CHARACTER_ID" >/dev/null
auth_get "$BASE_URL/api/game/afk?character_id=$CHARACTER_ID" >/dev/null

echo "4b websocket auth and command"
TOKEN="$TOKEN" CHARACTER_ID="$CHARACTER_ID" BASE_URL="$BASE_URL" node "$SCRIPT_DIR/smoke-ws.mjs"

echo "5 mail claim"
MAIL_BODY="$(auth_get "$BASE_URL/api/game/mail?character_id=$CHARACTER_ID")"
MAIL_ID="$(printf "%s" "$MAIL_BODY" | json_find_mail_id || true)"
if [ -n "${MAIL_ID:-}" ]; then
  auth_post "$BASE_URL/api/game/mail/claim" \
    -d "{\"character_id\":$CHARACTER_ID,\"mail_id\":$MAIL_ID}" >/dev/null
fi

echo "6 inventory actions"
auth_post "$BASE_URL/api/game/shop/buy" \
  -d "{\"character_id\":$CHARACTER_ID,\"template_id\":\"potion_small\",\"quantity\":1}" >/dev/null
auth_post "$BASE_URL/api/game/shop/buy" \
  -d "{\"character_id\":$CHARACTER_ID,\"template_id\":\"stone_refine\",\"quantity\":1}" >/dev/null
INV_BODY="$(auth_get "$BASE_URL/api/game/inventory?character_id=$CHARACTER_ID")"
POTION_ID="$(printf "%s" "$INV_BODY" | json_find_template_item_id potion_small bag || true)"
if [ -n "${POTION_ID:-}" ]; then
  auth_post "$BASE_URL/api/game/store" \
    -d "{\"character_id\":$CHARACTER_ID,\"item_id\":$POTION_ID}" >/dev/null
  INV_BODY="$(auth_get "$BASE_URL/api/game/inventory?character_id=$CHARACTER_ID")"
  STORED_POTION_ID="$(printf "%s" "$INV_BODY" | json_find_template_item_id potion_small warehouse || true)"
  if [ -n "${STORED_POTION_ID:-}" ]; then
    auth_post "$BASE_URL/api/game/retrieve" \
      -d "{\"character_id\":$CHARACTER_ID,\"item_id\":$STORED_POTION_ID}" >/dev/null
  fi
fi

INV_BODY="$(auth_get "$BASE_URL/api/game/inventory?character_id=$CHARACTER_ID")"
EQUIPMENT_ID="$(printf "%s" "$INV_BODY" | json_find_equipment_item_id || true)"
if [ -n "${EQUIPMENT_ID:-}" ]; then
  auth_post "$BASE_URL/api/game/enhance" \
    -d "{\"character_id\":$CHARACTER_ID,\"item_id\":$EQUIPMENT_ID}" >/dev/null
  auth_post "$BASE_URL/api/game/equipment/decompose" \
    -d "{\"character_id\":$CHARACTER_ID,\"rarities\":[],\"item_ids\":[$EQUIPMENT_ID]}" >/dev/null
fi

echo "7 trade"
INV_BODY="$(auth_get "$BASE_URL/api/game/inventory?character_id=$CHARACTER_ID")"
ITEM_ID="$(printf "%s" "$INV_BODY" | json_find_bag_item_id || true)"
if [ -n "${ITEM_ID:-}" ]; then
  auth_post "$BASE_URL/api/game/trade/list" \
    -d "{\"character_id\":$CHARACTER_ID,\"item_id\":$ITEM_ID,\"price\":1}" >/dev/null
  TRADE_BODY="$(auth_get "$BASE_URL/api/game/trade?character_id=$CHARACTER_ID")"
  CONSIGNMENT_ID="$(printf "%s" "$TRADE_BODY" | json_find_own_consignment_id || true)"
  if [ -n "${CONSIGNMENT_ID:-}" ]; then
    auth_post "$BASE_URL/api/game/trade/cancel" \
      -d "{\"character_id\":$CHARACTER_ID,\"consignment_id\":$CONSIGNMENT_ID}" >/dev/null
  fi
fi

echo "8 admin"
curl_json -H "x-admin-token: $ADMIN_TOKEN" "$BASE_URL/api/admin/dashboard" >/dev/null
curl_json -H "x-admin-token: $ADMIN_TOKEN" "$BASE_URL/api/admin/accounts" >/dev/null
curl_json -H "x-admin-token: $ADMIN_TOKEN" "$BASE_URL/api/admin/characters" >/dev/null
curl_json -H "x-admin-token: $ADMIN_TOKEN" "$BASE_URL/api/admin/character-detail?character_id=$CHARACTER_ID" >/dev/null
post_json -H "x-admin-token: $ADMIN_TOKEN" "$BASE_URL/api/admin/character-update" \
  -d "{\"character_id\":$CHARACTER_ID,\"gold\":1234,\"reason\":\"smoke\"}" >/dev/null
post_json -H "x-admin-token: $ADMIN_TOKEN" "$BASE_URL/api/admin/character-item" \
  -d "{\"character_id\":$CHARACTER_ID,\"template_id\":\"potion_small\",\"quantity\":1,\"location\":\"bag\",\"slot\":null,\"bind\":false,\"durability\":100,\"extra\":{}}" >/dev/null
DETAIL_BODY="$(curl_json -H "x-admin-token: $ADMIN_TOKEN" "$BASE_URL/api/admin/character-detail?character_id=$CHARACTER_ID")"
GM_ITEM_ID="$(printf "%s" "$DETAIL_BODY" | json_find_detail_template_item_id potion_small || true)"
if [ -n "${GM_ITEM_ID:-}" ]; then
  post_json -H "x-admin-token: $ADMIN_TOKEN" "$BASE_URL/api/admin/character-item-delete" \
    -d "{\"character_id\":$CHARACTER_ID,\"item_id\":$GM_ITEM_ID}" >/dev/null
fi
curl_json -H "x-admin-token: $ADMIN_TOKEN" "$BASE_URL/api/admin/mail" >/dev/null
curl_json -H "x-admin-token: $ADMIN_TOKEN" "$BASE_URL/api/admin/items" >/dev/null
curl_json -H "x-admin-token: $ADMIN_TOKEN" "$BASE_URL/api/admin/mobs" >/dev/null
curl_json -H "x-admin-token: $ADMIN_TOKEN" "$BASE_URL/api/admin/audit" >/dev/null
post_json -H "x-admin-token: $ADMIN_TOKEN" "$BASE_URL/api/admin/send-mail" \
  -d "{\"to_character_id\":$CHARACTER_ID,\"title\":\"smoke mail\",\"body\":\"smoke test\",\"gold\":1,\"yuanbao\":0,\"item_template_id\":null,\"quantity\":1}" >/dev/null

echo "smoke ok: user=$USERNAME character_id=$CHARACTER_ID"
