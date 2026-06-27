insert into game_settings (key, value, version)
values ('coffee_qr_url', '""'::jsonb, 1)
on conflict (key) do nothing;

update skills
set config = jsonb_set(config, '{drop_rate}', '"0.01%"'::jsonb, true),
    updated_at = now()
where id in (
  'skill_sword_clear_heart',
  'skill_sword_zhuxian',
  'skill_spell_chaos_orb',
  'skill_spell_thunder_array',
  'skill_soul_all_unity',
  'skill_soul_yanluo_prison'
);
