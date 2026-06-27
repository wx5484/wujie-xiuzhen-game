-- 0.2.8: daily stamina and fatigue state.

alter table character_state
  add column if not exists stamina integer not null default 5000,
  add column if not exists stamina_max integer not null default 5000,
  add column if not exists stamina_recovered_on date not null default ((now() at time zone 'Asia/Shanghai')::date);

update character_state
set stamina_max = 5000,
    stamina = least(greatest(stamina, 0), 5000),
    stamina_recovered_on = coalesce(stamina_recovered_on, (now() at time zone 'Asia/Shanghai')::date);

alter table character_state
  drop constraint if exists character_state_stamina_range,
  add constraint character_state_stamina_range
    check (stamina between 0 and stamina_max and stamina_max = 5000);
