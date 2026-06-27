-- 0.2.13 follow-up: test tools and probability tuning.

update item_templates
set
  stats = stats
    || '{"star_devourer_kill_growth_pct":0.05}'::jsonb
    || '{"special_mechanism_extra":"击杀怪物时有0.05%概率永久增加1点随机基础属性"}'::jsonb,
  updated_at = now()
where id = 'bracelet_star_devourer';
