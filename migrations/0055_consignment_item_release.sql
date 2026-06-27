alter table consignments
  drop constraint if exists consignments_item_id_fkey;

alter table consignments
  alter column item_id drop not null;

alter table consignments
  add constraint consignments_item_id_fkey
  foreign key (item_id) references inventory_items(id) on delete set null;

alter table consignments
  drop constraint if exists consignments_listed_item_required;

alter table consignments
  add constraint consignments_listed_item_required
  check (status <> 'listed' or item_id is not null);

create index if not exists idx_consignments_item_id
  on consignments (item_id)
  where item_id is not null;

update inventory_items ii
set location = 'bag', slot = null
from consignments c
where c.item_id = ii.id
  and c.status in ('sold', 'cancelled', 'expired')
  and ii.location = 'consignment';

update consignments
set item_id = null
where status in ('sold', 'cancelled', 'expired');
