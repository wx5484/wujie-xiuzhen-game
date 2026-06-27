# WebSocket JSON 协议

所有消息都使用统一 envelope：

```json
{
  "type": "cmd",
  "seq": 1,
  "payload": {}
}
```

客户端消息：

- `auth`：`{ "token": "...", "character_id": 1, "device": "web" }`
- `cmd`：`{ "command": "move", "args": { "direction": "north" } }`
- `state_request`：主动刷新房间与角色状态
- `mail_action`、`trade_action`、`guild_action`、`pet_action`、`treasure_action`、`activity_action`

服务端消息：

- `ack`
- `auth_ok` / `auth_error`
- `state`
- `room_state`
- `combat_log`
- `system_notice`
- `mail_update`
- `trade_update`
- `guild_update`
- `activity_update`
- `error`
- `force_logout`
