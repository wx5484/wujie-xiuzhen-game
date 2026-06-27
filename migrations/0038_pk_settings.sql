alter table character_state
  add column if not exists pk_enabled boolean not null default false,
  add column if not exists sweep_attack_players boolean not null default false;
