insert into recharge_cards (code, yuanbao) values
  ('TESTVIP100', 100),
  ('TESTVIP101', 100),
  ('TESTVIP102', 100),
  ('TESTSVIP500', 500)
on conflict (code) do update set
  yuanbao = excluded.yuanbao
where recharge_cards.used_by is null and recharge_cards.used_at is null;
