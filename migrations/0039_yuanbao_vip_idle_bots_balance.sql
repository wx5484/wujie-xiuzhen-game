alter table character_state
  add column if not exists last_idle_regen_at timestamptz not null default now();

update character_state
set last_idle_regen_at = coalesce(last_idle_regen_at, now());

create table if not exists vip_potion_settings (
  character_id bigint primary key references characters(id) on delete cascade,
  hp_enabled boolean not null default true,
  hp_threshold_pct integer not null default 35 check (hp_threshold_pct between 1 and 99),
  hp_template_id text not null default 'potion_large' references item_templates(id),
  mp_enabled boolean not null default true,
  mp_threshold_pct integer not null default 30 check (mp_threshold_pct between 1 and 99),
  mp_template_id text not null default 'potion_mana_large' references item_templates(id),
  updated_at timestamptz not null default now()
);

create trigger vip_potion_settings_set_updated_at before update on vip_potion_settings
  for each row execute function set_updated_at();

update item_templates
set flags = flags || '{"shop":"supply"}'::jsonb
where id in ('potion_small', 'potion_mana', 'scroll_return');

insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats, flags) values
  ('potion_large', '强效红药', 'consumable', null, 'uncommon', 240, true, '{"hp":260}', '{"shop":"supply"}'),
  ('potion_mana_large', '强效蓝药', 'consumable', null, 'uncommon', 280, true, '{"mp":220}', '{"shop":"supply"}'),
  ('potion_full', '万年雪霜', 'consumable', null, 'rare', 100, true, '{"full_restore":true,"hp_pct":100,"mp_pct":100}', '{"shop":"yuanbao","yuanbao_price":100}')
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats,
  flags = excluded.flags;

insert into pet_templates (id, name, rarity, base_hp, base_atk, skills) values
  ('pet_white_tiger', '白虎幼崽', 'rare', 220, 24, '["撕咬","护主","巡猎"]'),
  ('pet_fire_bird', '烈焰灵鸟', 'epic', 170, 38, '["火羽","灼烧","灵压"]')
on conflict (id) do update set
  name = excluded.name,
  rarity = excluded.rarity,
  base_hp = excluded.base_hp,
  base_atk = excluded.base_atk,
  skills = excluded.skills;

insert into treasure_templates (id, name, family, passive, config) values
  ('treasure_dragon_seal', '龙纹印', 'dragon', '主攻法宝：提升攻击、魔法、少量生命和暴击，适合推进中期地图与首领战。', '{"atk_pct":4,"mag_pct":4,"hp_pct":2,"crit_pct":1,"boss_drop_pct":2}'),
  ('treasure_guard_mirror', '玄光镜', 'guard', '守护法宝：提升生命、物防、魔防和魔法上限，适合挂机、越级探索和高阶 Boss。', '{"hp_pct":6,"def_pct":3,"mdef_pct":5,"mp_pct":2}')
on conflict (id) do update set
  name = excluded.name,
  family = excluded.family,
  passive = excluded.passive,
  config = excluded.config;

insert into mob_templates (id, name, level, max_hp, atk, def, exp, gold, boss, respawn_seconds) values
  ('chicken', '鸡', 1, 22, 2, 0, 8, 5, false, 45),
  ('deer', '鹿', 2, 30, 3, 1, 12, 8, false, 60),
  ('sheep', '羊', 3, 44, 4, 1, 18, 12, false, 60),
  ('scarecrow', '稻草人', 2, 34, 4, 0, 14, 8, false, 45),
  ('hook_cat', '钉耙猫', 4, 68, 8, 2, 32, 20, false, 80),
  ('forest_wolf', '森林狼', 6, 104, 12, 4, 48, 30, false, 90),
  ('poison_bee', '毒蜂', 7, 88, 14, 3, 55, 34, false, 80),
  ('cave_bat', '洞穴蝙蝠', 8, 122, 16, 5, 70, 42, false, 90),
  ('mine_zombie', '矿洞僵尸', 10, 178, 21, 8, 100, 60, false, 120),
  ('corpse_warrior', '尸王护卫', 14, 320, 33, 14, 180, 95, false, 150),
  ('corpse_king', '尸王', 18, 1250, 52, 22, 520, 260, true, 600),
  ('snake', '毒蛇', 11, 205, 24, 9, 115, 68, false, 100),
  ('red_snake', '红蛇', 14, 310, 32, 12, 165, 88, false, 120),
  ('valley_bandit', '山谷流寇', 16, 400, 40, 16, 220, 120, false, 130),
  ('serpent_guard', '蛇王卫士', 19, 560, 52, 22, 310, 160, false, 150),
  ('serpent_king', '山谷蛇王', 22, 1800, 78, 34, 900, 420, true, 900),
  ('woma_guard', '沃玛卫士', 18, 650, 56, 26, 360, 180, false, 150),
  ('woma_warrior', '沃玛战将', 21, 830, 68, 32, 470, 230, false, 180),
  ('woma_elder', '沃玛长老', 24, 1060, 82, 40, 620, 300, false, 210),
  ('woma_lord', '沃玛教主', 28, 3200, 122, 60, 1800, 850, true, 1200),
  ('desert_wolf', '沙漠土狼', 21, 760, 68, 28, 430, 210, false, 140),
  ('armored_beetle', '盔甲虫', 23, 900, 76, 38, 520, 250, false, 160),
  ('desert_bandit', '盟重流寇', 25, 1050, 88, 40, 650, 320, false, 180),
  ('red_boar', '红野猪', 27, 1220, 98, 46, 760, 370, false, 180),
  ('black_boar', '黑野猪', 29, 1420, 110, 54, 900, 430, false, 190),
  ('white_boar', '白野猪', 32, 4400, 158, 82, 2600, 1200, true, 1200),
  ('zuma_guard', '祖玛卫士', 32, 1880, 138, 70, 1250, 600, false, 220),
  ('zuma_archer', '祖玛弓箭手', 34, 1700, 158, 62, 1350, 660, false, 220),
  ('zuma_statue', '祖玛雕像', 36, 2480, 175, 88, 1650, 760, false, 240),
  ('zuma_lord', '祖玛教主', 38, 24000, 265, 130, 18000, 8000, true, 1800),
  ('cangyue_warrior', '苍月妖兵', 34, 2100, 150, 74, 1500, 720, false, 220),
  ('sea_demon', '海魔妖', 36, 2350, 168, 80, 1750, 800, false, 240),
  ('bone_soldier', '骨魔士兵', 38, 2750, 188, 90, 2100, 920, false, 260),
  ('bone_general', '骨魔将', 42, 4300, 232, 114, 3200, 1350, false, 300),
  ('nether_lord', '黄泉教主', 45, 30000, 325, 162, 24000, 10000, true, 1800),
  ('bull_guard', '牛魔护卫', 42, 4500, 238, 122, 3400, 1450, false, 300),
  ('bull_warrior', '牛魔战士', 45, 5400, 270, 138, 4300, 1700, false, 320),
  ('bull_king', '牛魔王', 50, 42000, 395, 210, 36000, 15000, true, 2400),
  ('moon_spider', '月魔蜘蛛', 70, 22000, 560, 290, 18000, 3600, false, 300),
  ('blood_giant', '血巨人', 78, 32000, 700, 380, 26000, 5000, false, 360),
  ('redmoon_priest', '赤月祭司', 86, 44000, 860, 500, 36000, 7000, false, 420),
  ('twin_head_guard', '双头金刚', 94, 125000, 1180, 720, 90000, 16000, true, 1200),
  ('redmoon_demon', '赤月恶魔', 100, 260000, 1450, 880, 180000, 32000, true, 1800),
  ('molong_guard', '魔龙守卫', 100, 65000, 1260, 760, 52000, 9500, false, 420),
  ('molong_lancer', '魔龙枪兵', 110, 92000, 1540, 920, 70000, 12500, false, 480),
  ('molong_hunter', '魔龙猎手', 120, 125000, 1820, 1100, 92000, 16000, false, 540),
  ('molong_general', '魔龙统领', 132, 185000, 2180, 1340, 130000, 22000, false, 600),
  ('molong_warrior', '魔龙战将', 142, 245000, 2520, 1580, 170000, 28000, false, 660),
  ('molong_blood_demon', '魔龙血魔', 150, 390000, 2920, 1840, 240000, 42000, true, 1800),
  ('molong_lord', '魔龙教主', 150, 620000, 3150, 1980, 360000, 65000, true, 2400)
on conflict (id) do update set
  name = excluded.name,
  level = excluded.level,
  max_hp = excluded.max_hp,
  atk = excluded.atk,
  def = excluded.def,
  exp = excluded.exp,
  gold = excluded.gold,
  boss = excluded.boss,
  respawn_seconds = excluded.respawn_seconds;

create table if not exists bot_profiles (
  id bigserial primary key,
  name text not null unique,
  bot_class text not null default 'warrior' check (bot_class in ('warrior', 'mage', 'taoist', 'assassin')),
  level integer not null default 1 check (level between 1 and 200),
  exp bigint not null default 0,
  gold bigint not null default 0,
  power bigint not null default 120,
  zone text not null default 'bq_plains',
  room text not null default 'newbie_village',
  hp bigint not null default 120,
  mp bigint not null default 50,
  mode text not null default 'progression' check (mode in ('progression', 'dispatch', 'team_farm', 'fixed_clear')),
  team_code text not null default '',
  target_zone text not null default '',
  target_room text not null default '',
  enabled boolean not null default true,
  script jsonb not null default '{}'::jsonb,
  last_action_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create index if not exists bot_profiles_position_idx on bot_profiles (enabled, zone, room);
create index if not exists bot_profiles_mode_idx on bot_profiles (mode, enabled, team_code);

create trigger bot_profiles_set_updated_at before update on bot_profiles
  for each row execute function set_updated_at();

insert into bot_profiles
  (name, bot_class, level, exp, gold, power, zone, room, mode, team_code, target_zone, target_room, script)
values
  ('练级小队一号', 'warrior', 2, 40, 35, 160, 'bq_plains', 'newbie_village', 'progression', 'newbie-a', '', '', '{"note":"从新手村自然成长"}'),
  ('练级小队二号', 'mage', 2, 30, 28, 150, 'bq_plains', 'newbie_village', 'progression', 'newbie-a', '', '', '{"note":"从新手村自然成长"}'),
  ('矿洞清场一号', 'taoist', 12, 1800, 420, 520, 'bq_plains', 'mine_entrance', 'fixed_clear', 'mine-clear', 'bq_plains', 'mine_entrance', '{"note":"固定矿洞清场"}'),
  ('石墓组队一号', 'warrior', 28, 16000, 2200, 1800, 'mengzhong', 'stone_tomb', 'team_farm', 'stone-team', 'mengzhong', 'stone_tomb', '{"note":"同地图组队刷怪"}')
on conflict (name) do update set
  bot_class = excluded.bot_class,
  mode = excluded.mode,
  team_code = excluded.team_code,
  target_zone = excluded.target_zone,
  target_room = excluded.target_room,
  script = excluded.script;
