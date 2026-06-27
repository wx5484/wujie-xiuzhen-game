alter table consignments
  add column if not exists listing_fee_gold bigint not null default 0;

alter table consignment_history
  add column if not exists trade_tax_yuanbao bigint not null default 0,
  add column if not exists seller_receives_yuanbao bigint not null default 0,
  add column if not exists listing_fee_gold bigint not null default 0;

update consignments
set fee_yuanbao = least(greatest(ceil(price_yuanbao * 0.03)::bigint, 0), greatest(price_yuanbao / 10, 0))
where fee_yuanbao = 0 and price_yuanbao > 10;

update consignment_history
set
  trade_tax_yuanbao = least(greatest(ceil(price_yuanbao * 0.03)::bigint, 0), greatest(price_yuanbao / 10, 0)),
  seller_receives_yuanbao = greatest(price_yuanbao - least(greatest(ceil(price_yuanbao * 0.03)::bigint, 0), greatest(price_yuanbao / 10, 0)), 0)
where seller_receives_yuanbao = 0;
