-- 0.2.7: inventory page support, stronger bot PvP curve, and boss-gated breakthrough systems.

create table if not exists character_system_unlocks (
  character_id bigint not null references characters(id) on delete cascade,
  system_id text not null,
  source_mob_id text not null,
  unlocked_at timestamptz not null default now(),
  primary key (character_id, system_id)
);

create index if not exists idx_character_system_unlocks_character
  on character_system_unlocks (character_id);

update mob_templates
set name = '幻境牛魔王'
where id = 'bull_king';

update bot_profiles
set hp = greatest(hp, 160 + greatest(level - 1, 0) * 24),
    mp = greatest(mp, 60 + greatest(level - 1, 0) * 8),
    power = greatest(
      power,
      level * (
        42 + case bot_class
          when 'mage' then 18
          when 'taoist' then 14
          when 'assassin' then 20
          else 16
        end
      )
    );
