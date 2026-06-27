create table quest_templates (
  id text primary key,
  category text not null check (category in ('newbie', 'main', 'side', 'daily')),
  name text not null,
  description text not null,
  min_level integer not null default 1,
  sort_order integer not null default 0,
  enabled boolean not null default true,
  objectives jsonb not null default '{}'::jsonb,
  rewards jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table character_quests (
  character_id bigint not null references characters(id) on delete cascade,
  quest_id text not null references quest_templates(id) on delete cascade,
  period_key text not null default 'once',
  progress jsonb not null default '{"value":0}'::jsonb,
  completed_at timestamptz,
  claimed_at timestamptz,
  updated_at timestamptz not null default now(),
  primary key (character_id, quest_id, period_key)
);

create index character_quests_character_idx on character_quests (character_id, quest_id);
create index quest_templates_enabled_idx on quest_templates (enabled, category, sort_order);

create trigger quest_templates_set_updated_at before update on quest_templates
  for each row execute function set_updated_at();
create trigger character_quests_set_updated_at before update on character_quests
  for each row execute function set_updated_at();

insert into quest_templates
  (id, category, name, description, min_level, sort_order, objectives, rewards)
values
  (
    'newbie_first_hunt',
    'newbie',
    '初试锋芒',
    '在比奇野外击败任意 1 只怪物，熟悉自动战斗节奏。',
    1,
    10,
    '{"kind":"kill_any","required":1}',
    '{"gold":80,"items":[{"template_id":"potion_small","quantity":3,"bind":true}]}'
  ),
  (
    'newbie_hunt_5',
    'newbie',
    '稳住脚跟',
    '累计击败 5 只怪物，积攒第一批补给。',
    1,
    20,
    '{"kind":"kill_any","required":5}',
    '{"gold":120,"items":[{"template_id":"potion_mana","quantity":2,"bind":true},{"template_id":"pet_food","quantity":1,"bind":true}]}'
  ),
  (
    'newbie_first_enhance',
    'newbie',
    '打磨兵刃',
    '完成 1 次装备强化，了解金币和材料消耗。',
    1,
    30,
    '{"kind":"enhance","required":1}',
    '{"gold":120,"items":[{"template_id":"scroll_return","quantity":1,"bind":true}]}'
  ),
  (
    'newbie_join_guild',
    'newbie',
    '找个靠山',
    '加入任意行会，为后续行会玩法做准备。',
    1,
    40,
    '{"kind":"join_guild","required":1}',
    '{"gold":120,"items":[{"template_id":"treasure_shard","quantity":1,"bind":true}]}'
  ),
  (
    'main_reach_level_5',
    'main',
    '比奇试炼',
    '角色等级达到 5 级，完成新手期第一段成长。',
    1,
    100,
    '{"kind":"level","required":5}',
    '{"gold":180,"items":[{"template_id":"potion_small","quantity":3,"bind":true}]}'
  ),
  (
    'main_reach_level_10',
    'main',
    '走出比奇',
    '角色等级达到 10 级，准备进入毒蛇山谷。',
    5,
    110,
    '{"kind":"level","required":10}',
    '{"gold":300,"items":[{"template_id":"potion_mana","quantity":3,"bind":true}]}'
  ),
  (
    'side_safe_afk',
    'side',
    '离线也要练功',
    '完成 1 次挂机结算，了解放置收益。',
    1,
    200,
    '{"kind":"afk_settle","required":1}',
    '{"gold":90,"items":[{"template_id":"potion_mana","quantity":1,"bind":true},{"template_id":"cultivation_pill","quantity":1,"bind":true}]}'
  ),
  (
    'daily_hunt_20',
    'daily',
    '每日猎魔',
    '今日累计击败 20 只怪物。',
    1,
    300,
    '{"kind":"kill_any","required":20}',
    '{"gold":180,"items":[{"template_id":"potion_small","quantity":2,"bind":true},{"template_id":"pet_food","quantity":1,"bind":true}]}'
  ),
  (
    'daily_afk_settle',
    'daily',
    '每日收菜',
    '今日完成 1 次挂机结算。',
    1,
    310,
    '{"kind":"afk_settle","required":1}',
    '{"gold":80}'
  )
on conflict (id) do update set
  category = excluded.category,
  name = excluded.name,
  description = excluded.description,
  min_level = excluded.min_level,
  sort_order = excluded.sort_order,
  objectives = excluded.objectives,
  rewards = excluded.rewards,
  enabled = true;
