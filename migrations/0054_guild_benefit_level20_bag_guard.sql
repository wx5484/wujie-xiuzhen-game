alter table guild_benefit_claims
  drop constraint if exists guild_benefit_claims_level_check;

alter table guild_benefit_claims
  add constraint guild_benefit_claims_level_check
  check (level between 1 and 20);

create index if not exists idx_inventory_decompose_scan
  on inventory_items (character_id, location, bind, id)
  where location in ('bag', 'warehouse');
