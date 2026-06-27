-- 0.2.14: guild totems, automatic Sabak, meditation training, and special passive rework.

create table if not exists guild_totems (
  character_id bigint not null references characters(id) on delete cascade,
  guild_id bigint not null references guilds(id) on delete cascade,
  totem text not null check (totem in ('qiongqi', 'bifang', 'chenghuang', 'xuangui')),
  level integer not null default 0 check (level between 0 and 100),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  primary key (character_id, totem)
);

create index if not exists guild_totems_guild_idx on guild_totems (guild_id);

create or replace function clear_guild_totems_on_member_delete()
returns trigger as $$
begin
  delete from guild_totems
  where character_id = old.character_id and guild_id = old.guild_id;
  return old;
end;
$$ language plpgsql;

drop trigger if exists guild_members_clear_totems on guild_members;
create trigger guild_members_clear_totems after delete on guild_members
  for each row execute function clear_guild_totems_on_member_delete();

create table if not exists guild_war_tech (
  guild_id bigint not null references guilds(id) on delete cascade,
  kind text not null check (kind in ('siege_chariot', 'defense_barrier')),
  level integer not null default 0 check (level between 0 and 100),
  charged_points bigint not null default 0,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  primary key (guild_id, kind)
);

create table if not exists guild_sabak_state (
  id integer primary key default 1 check (id = 1),
  winner_guild_id bigint references guilds(id) on delete set null,
  winner_name text not null default '比奇远征队',
  last_settled_at timestamptz,
  updated_at timestamptz not null default now()
);

insert into guild_sabak_state (id, winner_name)
values (1, '比奇远征队')
on conflict (id) do nothing;

create table if not exists guild_sabak_tax_claims (
  guild_id bigint not null references guilds(id) on delete cascade,
  character_id bigint not null references characters(id) on delete cascade,
  period_key text not null,
  created_at timestamptz not null default now(),
  primary key (guild_id, character_id, period_key)
);

create index if not exists guild_sabak_tax_claims_character_idx
  on guild_sabak_tax_claims (character_id, period_key);

update world_rooms
set exits = exits || '{"练功房":"training_room"}'::jsonb
where zone_id = 'mengzhong' and id = 'town';

insert into world_rooms (zone_id, id, name, description, exits, spawns, safe) values
  (
    'mengzhong',
    'training_room',
    '练功房',
    '土城练功房内刀痕遍墙，远征者可在此打坐修炼，只结算角色经验，不产出装备、金币或材料。',
    '{"盟重土城":"town"}',
    '[]',
    false
  )
on conflict (zone_id, id) do update set
  name = excluded.name,
  description = excluded.description,
  exits = excluded.exits,
  spawns = excluded.spawns,
  safe = excluded.safe;

update world_rooms
set exits = exits || '{"书斋":"study"}'::jsonb
where zone_id = 'redmoon' and id = 'bairimen_gate';

insert into world_rooms (zone_id, id, name, description, exits, spawns, safe) values
  (
    'redmoon',
    'study',
    '书斋',
    '白日门书斋灯火幽微，修行者可指定已学习技能打坐研修，只增长技能经验，不产出装备、金币或材料。',
    '{"白日门":"bairimen_gate"}',
    '[]',
    false
  )
on conflict (zone_id, id) do update set
  name = excluded.name,
  description = excluded.description,
  exits = excluded.exits,
  spawns = excluded.spawns,
  safe = excluded.safe;

update world_rooms
set description = replace(description, ' 武学宗师在土城传授特殊技能战斗本能，可用技能书残页逐级突破。', '')
where zone_id = 'mengzhong' and id = 'town';

update world_rooms
set description = description || ' 武学宗师在此传授特殊被动战斗本能与不动冥王身，只收技能书残页。'
where zone_id = 'cangyue' and id = 'safe_harbor' and description not like '%武学宗师%';

insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats, flags) values
  ('book_special_passive', '特殊被动技能书', 'book', null, 'legendary', 8000, false, '{"tier":"special_passive"}'::jsonb, '{"source":"zuma_lord"}'::jsonb)
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats,
  flags = item_templates.flags || excluded.flags;

update skills
set config = (config - 'def_bonus' - 'mdef_bonus' - 'mp_bonus' - 'crit_bonus' - 'crit_pct_bonus' - 'luck_bonus')
  || '{
       "kind":"passive",
       "type":"特殊被动",
       "desc":"特殊被动。只能在苍月岛安全区寻找武学宗师，消耗技能书残页逐级提升。",
       "source":"苍月岛安全区武学宗师",
       "requires_book":false,
       "special_upgrade_only":true,
       "atk_bonus":18,
       "mag_bonus":18,
       "hp_bonus":60,
       "luck_bonus_per_100":20
     }'::jsonb,
    max_level = 100
where id = 'talent_battle_instinct';

insert into skills (id, name, class, min_level, mp_cost, cooldown_ms, config, max_level) values
  (
    'talent_immovable_king',
    '不动冥王身',
    'all',
    1,
    0,
    0,
    '{
       "kind":"passive",
       "type":"特殊被动",
       "desc":"特殊被动。只能在苍月岛安全区寻找武学宗师，消耗技能书残页逐级提升。",
       "source":"苍月岛安全区武学宗师",
       "requires_book":false,
       "special_upgrade_only":true,
       "hp_bonus":100,
       "mp_bonus":100,
       "def_bonus":6,
       "mdef_bonus":6,
       "control_resist_pct_per_100":20
     }'::jsonb,
    100
  )
on conflict (id) do update set
  name = excluded.name,
  class = excluded.class,
  min_level = excluded.min_level,
  mp_cost = excluded.mp_cost,
  cooldown_ms = excluded.cooldown_ms,
  config = excluded.config,
  max_level = excluded.max_level;

update item_templates
set flags = flags || '{"source":"zuma_lord_plus","exclusive":true}'::jsonb
where id = 'skill_page';
