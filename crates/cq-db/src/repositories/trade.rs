use cq_domain::inventory::bag_limit_for_level;
use cq_protocol::dto::TradeConsignmentView;
use serde_json::json;
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TradeError {
    #[error("consignment not found")]
    NotFound,
    #[error("invalid price")]
    InvalidPrice,
    #[error("item cannot be listed")]
    ItemUnavailable,
    #[error("cannot buy own consignment")]
    OwnConsignment,
    #[error("not enough yuanbao")]
    NotEnoughYuanbao,
    #[error("not enough gold")]
    NotEnoughGold,
    #[error("bag is full")]
    BagFull,
    #[error("database error")]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Clone, FromRow)]
struct TradeConsignmentRow {
    id: i64,
    seller_character_id: i64,
    seller_name: String,
    item_id: i64,
    template_id: String,
    name: String,
    kind: String,
    template_slot: Option<String>,
    rarity: String,
    quantity: i64,
    price: i64,
    price_currency: String,
    listing_fee_gold: i64,
    trade_tax_yuanbao: i64,
    trade_tax_gold: i64,
    seller_receives_yuanbao: i64,
    seller_receives_gold: i64,
    stats: serde_json::Value,
    bind: bool,
    durability: i32,
    expires_at: String,
    created_at: String,
    mine: bool,
}

#[derive(Debug, Clone)]
pub struct TradeRepository {
    pool: PgPool,
}

const LISTING_FEE_GOLD_RATE_BPS: i64 = 50;
const LISTING_FEE_GOLD_MIN: i64 = 5;
const LISTING_FEE_GOLD_MAX: i64 = 5_000;
const TRADE_TAX_YUANBAO_RATE_BPS: i64 = 300;
const TRADE_TAX_YUANBAO_FREE_PRICE: i64 = 10;
const TRADE_TAX_YUANBAO_MAX: i64 = 50_000;
const TRADE_TAX_GOLD_RATE_BPS: i64 = 500;
const TRADE_TAX_GOLD_FREE_PRICE: i64 = 200;
const TRADE_TAX_GOLD_MAX: i64 = 500_000;

impl TradeRepository {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn list(&self, character_id: i64) -> Result<Vec<TradeConsignmentView>, sqlx::Error> {
        self.expire_old_consignments().await?;
        let rows = sqlx::query_as::<_, TradeConsignmentRow>(trade_list_sql())
            .bind(character_id)
            .bind(Option::<i64>::None)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(consignment_view).collect())
    }

    pub async fn create(
        &self,
        character_id: i64,
        item_id: i64,
        price: i64,
        price_currency: &str,
    ) -> Result<Vec<TradeConsignmentView>, TradeError> {
        if price <= 0 {
            return Err(TradeError::InvalidPrice);
        }
        let price_currency = normalize_currency(price_currency);

        let listing_fee_gold = calculate_listing_fee_gold(price);
        let trade_tax_yuanbao = if price_currency == "yuanbao" { calculate_trade_tax_yuanbao(price) } else { 0 };
        let trade_tax_gold = if price_currency == "gold" { calculate_trade_tax_gold(price) } else { 0 };
        let mut tx = self.pool.begin().await?;
        let (seller_gold,): (i64,) = sqlx::query_as(
            "select gold from characters where id = $1 and deleted_at is null for update",
        )
        .bind(character_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(TradeError::NotFound)?;
        if seller_gold < listing_fee_gold {
            return Err(TradeError::NotEnoughGold);
        }

        let updated = sqlx::query(
            r#"
            update inventory_items
            set location = 'consignment', slot = null
            where id = $1
              and character_id = $2
              and location = 'bag'
              and bind = false
            "#,
        )
        .bind(item_id)
        .bind(character_id)
        .execute(&mut *tx)
        .await?;
        if updated.rows_affected() == 0 {
            return Err(TradeError::ItemUnavailable);
        }

        if listing_fee_gold > 0 {
            sqlx::query("update characters set gold = gold - $2 where id = $1")
                .bind(character_id)
                .bind(listing_fee_gold)
                .execute(&mut *tx)
                .await?;
        }

        sqlx::query(
            r#"
            insert into consignments
              (seller_character_id, item_id, price_yuanbao, price_currency, fee_yuanbao, trade_tax_gold, listing_fee_gold, expires_at)
            values ($1, $2, $3, $4, $5, $6, $7, now() + interval '7 days')
            "#,
        )
        .bind(character_id)
        .bind(item_id)
        .bind(price)
        .bind(price_currency)
        .bind(trade_tax_yuanbao)
        .bind(trade_tax_gold)
        .bind(listing_fee_gold)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(self.list(character_id).await?)
    }

    pub async fn buy(
        &self,
        buyer_character_id: i64,
        consignment_id: i64,
    ) -> Result<Vec<TradeConsignmentView>, TradeError> {
        let mut tx = self.pool.begin().await?;
        let row = lock_consignment(&mut tx, consignment_id).await?;
        if row.seller_character_id == buyer_character_id {
            return Err(TradeError::OwnConsignment);
        }

        let buyer: (i64, i64, i32) = sqlx::query_as(
            r#"
            select gold, yuanbao, level
            from characters
            where id = $1 and deleted_at is null
            for update
            "#,
        )
        .bind(buyer_character_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(TradeError::NotFound)?;
        if row.price_currency == "gold" {
            if buyer.0 < row.price {
                return Err(TradeError::NotEnoughGold);
            }
        } else if buyer.1 < row.price {
            return Err(TradeError::NotEnoughYuanbao);
        }

        let (bag_count,): (i64,) = sqlx::query_as(
            "select count(*)::bigint from inventory_items where character_id = $1 and location = 'bag'",
        )
        .bind(buyer_character_id)
        .fetch_one(&mut *tx)
        .await?;
        if bag_count as usize >= bag_limit_for_level(buyer.2) {
            return Err(TradeError::BagFull);
        }

        sqlx::query("select id from characters where id = $1 and deleted_at is null for update")
            .bind(row.seller_character_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(TradeError::NotFound)?;

        if row.price_currency == "gold" {
            sqlx::query("update characters set gold = gold - $2 where id = $1")
                .bind(buyer_character_id)
                .bind(row.price)
                .execute(&mut *tx)
                .await?;
            sqlx::query("update characters set gold = gold + $2 where id = $1")
                .bind(row.seller_character_id)
                .bind(row.seller_receives_gold)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query("update characters set yuanbao = yuanbao - $2 where id = $1")
                .bind(buyer_character_id)
                .bind(row.price)
                .execute(&mut *tx)
                .await?;
            sqlx::query("update characters set yuanbao = yuanbao + $2 where id = $1")
                .bind(row.seller_character_id)
                .bind(row.seller_receives_yuanbao)
                .execute(&mut *tx)
                .await?;
        }
        sqlx::query(
            r#"
            update inventory_items
            set character_id = $2, location = 'bag', slot = null
            where id = $1 and location = 'consignment'
            "#,
        )
        .bind(row.item_id)
        .bind(buyer_character_id)
        .execute(&mut *tx)
        .await?;
        sqlx::query("update consignments set status = 'sold', item_id = null where id = $1")
            .bind(consignment_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            r#"
            insert into consignment_history
              (
                consignment_id,
                buyer_character_id,
                seller_character_id,
                item_snapshot,
                price_yuanbao,
                price_currency,
                trade_tax_yuanbao,
                trade_tax_gold,
                seller_receives_yuanbao,
                seller_receives_gold,
                listing_fee_gold
              )
            values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(consignment_id)
        .bind(buyer_character_id)
        .bind(row.seller_character_id)
        .bind(json!({
            "item_id": row.item_id,
            "template_id": row.template_id,
            "name": row.name,
            "kind": row.kind,
            "rarity": row.rarity,
            "quantity": row.quantity,
            "stats": row.stats,
            "durability": row.durability
        }))
        .bind(row.price)
        .bind(&row.price_currency)
        .bind(row.trade_tax_yuanbao)
        .bind(row.trade_tax_gold)
        .bind(row.seller_receives_yuanbao)
        .bind(row.seller_receives_gold)
        .bind(row.listing_fee_gold)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(self.list(buyer_character_id).await?)
    }

    pub async fn cancel(
        &self,
        character_id: i64,
        consignment_id: i64,
    ) -> Result<Vec<TradeConsignmentView>, TradeError> {
        let mut tx = self.pool.begin().await?;
        let row = lock_consignment(&mut tx, consignment_id).await?;
        if row.seller_character_id != character_id {
            return Err(TradeError::NotFound);
        }

        sqlx::query("update inventory_items set location = 'bag', slot = null where id = $1")
            .bind(row.item_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("update consignments set status = 'cancelled', item_id = null where id = $1")
            .bind(consignment_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(self.list(character_id).await?)
    }

    pub async fn expire_old_consignments(&self) -> Result<u64, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let rows = sqlx::query_as::<_, (i64, i64)>(
            r#"
            select id, item_id
            from consignments
            where status = 'listed'
              and expires_at < now()
            for update
            "#,
        )
        .fetch_all(&mut *tx)
        .await?;
        if rows.is_empty() {
            tx.commit().await?;
            return Ok(0);
        }

        let consignment_ids = rows.iter().map(|(id, _)| *id).collect::<Vec<_>>();
        let item_ids = rows.iter().map(|(_, item_id)| *item_id).collect::<Vec<_>>();
        sqlx::query(
            r#"
            update inventory_items
            set location = 'bag', slot = null
            where id = any($1::bigint[])
              and location = 'consignment'
            "#,
        )
        .bind(&item_ids)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"
            update consignments
            set status = 'expired', item_id = null, updated_at = now()
            where id = any($1::bigint[])
            "#,
        )
        .bind(&consignment_ids)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(consignment_ids.len() as u64)
    }
}

async fn lock_consignment(
    tx: &mut Transaction<'_, Postgres>,
    consignment_id: i64,
) -> Result<TradeConsignmentRow, TradeError> {
    let sql = format!("{} for update of c, ii", trade_list_sql());
    sqlx::query_as::<_, TradeConsignmentRow>(&sql)
        .bind(0_i64)
        .bind(consignment_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(TradeError::NotFound)
}

fn trade_list_sql() -> &'static str {
    r#"
    select
      c.id,
      c.seller_character_id,
      seller.name as seller_name,
      ii.id as item_id,
      ii.template_id,
      it.name,
      it.kind,
      it.slot as template_slot,
      it.rarity,
      ii.quantity,
      c.price_yuanbao as price,
      coalesce(c.price_currency, 'yuanbao') as price_currency,
      c.listing_fee_gold,
      case when coalesce(c.price_currency, 'yuanbao') = 'yuanbao' then c.fee_yuanbao else 0 end as trade_tax_yuanbao,
      case when coalesce(c.price_currency, 'yuanbao') = 'gold' then c.trade_tax_gold else 0 end as trade_tax_gold,
      case when coalesce(c.price_currency, 'yuanbao') = 'yuanbao' then greatest(c.price_yuanbao - c.fee_yuanbao, 0) else 0 end::bigint as seller_receives_yuanbao,
      case when coalesce(c.price_currency, 'yuanbao') = 'gold' then greatest(c.price_yuanbao - c.trade_tax_gold, 0) else 0 end::bigint as seller_receives_gold,
      it.stats,
      ii.bind,
      ii.durability,
      c.expires_at::text as expires_at,
      c.created_at::text as created_at,
      (c.seller_character_id = $1) as mine
    from consignments c
    join inventory_items ii on ii.id = c.item_id
    join item_templates it on it.id = ii.template_id
    join characters seller on seller.id = c.seller_character_id
    where c.status = 'listed'
      and c.expires_at > now()
      and ($2::bigint is null or c.id = $2)
    order by c.created_at desc, c.id desc
    "#
}

fn consignment_view(row: TradeConsignmentRow) -> TradeConsignmentView {
    TradeConsignmentView {
        id: row.id,
        seller_character_id: row.seller_character_id,
        seller_name: row.seller_name,
        item_id: row.item_id,
        template_id: row.template_id,
        name: row.name,
        kind: row.kind,
        template_slot: row.template_slot,
        rarity: row.rarity,
        quantity: row.quantity,
        price: row.price,
        price_currency: row.price_currency,
        listing_fee_gold: row.listing_fee_gold,
        trade_tax_yuanbao: row.trade_tax_yuanbao,
        trade_tax_gold: row.trade_tax_gold,
        seller_receives_yuanbao: row.seller_receives_yuanbao,
        seller_receives_gold: row.seller_receives_gold,
        stats: row.stats,
        bind: row.bind,
        durability: row.durability,
        expires_at: row.expires_at,
        created_at: row.created_at,
        mine: row.mine,
    }
}

fn calculate_listing_fee_gold(price_yuanbao: i64) -> i64 {
    if price_yuanbao <= 0 {
        return 0;
    }
    ceil_bps(price_yuanbao, LISTING_FEE_GOLD_RATE_BPS)
        .max(LISTING_FEE_GOLD_MIN)
        .min(LISTING_FEE_GOLD_MAX)
}

fn calculate_trade_tax_yuanbao(price_yuanbao: i64) -> i64 {
    if price_yuanbao <= TRADE_TAX_YUANBAO_FREE_PRICE {
        return 0;
    }
    let max_tax = (price_yuanbao / 10).max(1).min(TRADE_TAX_YUANBAO_MAX);
    ceil_bps(price_yuanbao, TRADE_TAX_YUANBAO_RATE_BPS).clamp(1, max_tax)
}

fn calculate_trade_tax_gold(price_gold: i64) -> i64 {
    if price_gold <= TRADE_TAX_GOLD_FREE_PRICE {
        return 0;
    }
    let max_tax = (price_gold / 10).max(1).min(TRADE_TAX_GOLD_MAX);
    ceil_bps(price_gold, TRADE_TAX_GOLD_RATE_BPS).clamp(1, max_tax)
}

fn normalize_currency(value: &str) -> &'static str {
    match value.trim() {
        "gold" => "gold",
        _ => "yuanbao",
    }
}

fn ceil_bps(value: i64, bps: i64) -> i64 {
    value.saturating_mul(bps).saturating_add(9_999) / 10_000
}
