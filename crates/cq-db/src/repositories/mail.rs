use cq_protocol::dto::{PlayerMailAttachmentView, PlayerMailView};
use sqlx::{FromRow, PgPool, Postgres, Transaction};

use super::character::CharacterRecord;

#[derive(Debug, Clone, FromRow)]
struct MailRow {
    id: i64,
    to_character_id: i64,
    from_name: String,
    title: String,
    body: String,
    read: bool,
    claimed: bool,
    expires_at: Option<String>,
    created_at: String,
}

#[derive(Debug, Clone, FromRow)]
struct MailAttachmentRow {
    id: i64,
    item_template_id: Option<String>,
    item_name: Option<String>,
    quantity: i64,
    gold: i64,
    yuanbao: i64,
    claimed: bool,
}

#[derive(Debug, Clone)]
pub struct ClaimedMailRecord {
    pub mail: PlayerMailView,
    pub character: CharacterRecord,
    pub gold: i64,
    pub yuanbao: i64,
    pub item_quantity: i64,
}

#[derive(Debug, Clone)]
pub struct MailRepository {
    pool: PgPool,
}

impl MailRepository {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn list_for_character(&self, character_id: i64) -> Result<Vec<PlayerMailView>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MailRow>(
            r#"
            select id,
                   to_character_id,
                   from_name,
                   title,
                   body,
                   read,
                   claimed,
                   expires_at::text as expires_at,
                   created_at::text as created_at
            from mails
            where to_character_id = $1
              and (expires_at is null or expires_at > now())
            order by created_at desc, id desc
            "#,
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;

        let mut mails = Vec::with_capacity(rows.len());
        for row in rows {
            let attachments = self.attachments_for_mail(row.id).await?;
            mails.push(mail_view(row, attachments));
        }
        Ok(mails)
    }

    pub async fn mark_read(&self, character_id: i64, mail_id: i64) -> Result<PlayerMailView, sqlx::Error> {
        let result = sqlx::query(
            r#"
            update mails
            set read = true
            where id = $1
              and to_character_id = $2
              and (expires_at is null or expires_at > now())
            "#,
        )
        .bind(mail_id)
        .bind(character_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        self.get_for_character(character_id, mail_id).await
    }

    pub async fn delete_for_character(&self, character_id: i64, mail_id: i64) -> Result<Vec<PlayerMailView>, sqlx::Error> {
        let result = sqlx::query(
            r#"
            delete from mails
            where id = $1
              and to_character_id = $2
              and (expires_at is null or expires_at > now())
            "#,
        )
        .bind(mail_id)
        .bind(character_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        self.list_for_character(character_id).await
    }

    pub async fn claim(&self, character_id: i64, mail_id: i64) -> Result<ClaimedMailRecord, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let _mail = sqlx::query_as::<_, MailRow>(
            r#"
            select id,
                   to_character_id,
                   from_name,
                   title,
                   body,
                   read,
                   claimed,
                   expires_at::text as expires_at,
                   created_at::text as created_at
            from mails
            where id = $1
              and to_character_id = $2
              and (expires_at is null or expires_at > now())
            for update
            "#,
        )
        .bind(mail_id)
        .bind(character_id)
        .fetch_one(&mut *tx)
        .await?;

        let attachments = sqlx::query_as::<_, MailAttachmentRow>(
            r#"
            select ma.id,
                   ma.item_template_id,
                   it.name as item_name,
                   ma.quantity,
                   ma.gold,
                   ma.yuanbao,
                   ma.claimed_at is not null as claimed
            from mail_attachments ma
            left join item_templates it on it.id = ma.item_template_id
            where ma.mail_id = $1
            order by ma.id asc
            for update of ma
            "#,
        )
        .bind(mail_id)
        .fetch_all(&mut *tx)
        .await?;

        let mut gold = 0i64;
        let mut yuanbao = 0i64;
        let mut item_quantity = 0i64;

        for attachment in attachments.iter().filter(|item| !item.claimed) {
            gold = gold.saturating_add(attachment.gold.max(0));
            yuanbao = yuanbao.saturating_add(attachment.yuanbao.max(0));
            if let Some(template_id) = attachment.item_template_id.as_deref() {
                if grant_item_in_tx(&mut tx, character_id, template_id, attachment.quantity).await? {
                    item_quantity = item_quantity.saturating_add(attachment.quantity.max(1));
                }
            }
        }

        let character = sqlx::query_as::<_, CharacterRecord>(
            r#"
            update characters
            set gold = gold + $2,
                yuanbao = yuanbao + $3
            where id = $1 and deleted_at is null
            returning id, account_id, name, class, level, exp, gold, yuanbao, power
            "#,
        )
        .bind(character_id)
        .bind(gold)
        .bind(yuanbao)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            update mail_attachments
            set claimed_at = now()
            where mail_id = $1 and claimed_at is null
            "#,
        )
        .bind(mail_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            update mails
            set read = true,
                claimed = true
            where id = $1 and to_character_id = $2
            "#,
        )
        .bind(mail_id)
        .bind(character_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        let mail = self.get_for_character(character_id, mail_id).await?;
        Ok(ClaimedMailRecord { mail, character, gold, yuanbao, item_quantity })
    }

    pub async fn create_welcome_mail(&self, character_id: i64) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let (mail_id,): (i64,) = sqlx::query_as(
            r#"
            insert into mails (to_character_id, from_name, title, body)
            values (
              $1,
              '系统',
              '新手行囊',
              '欢迎来到比奇。这里有几件基础补给：药水、回城卷、木剑和启动金币。领取后可以直接出城练级。'
            )
            returning id
            "#,
        )
        .bind(character_id)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            insert into mail_attachments (mail_id, item_template_id, quantity, gold, yuanbao) values
              ($1, 'potion_small', 5, 0, 0),
              ($1, 'potion_mana', 3, 0, 0),
              ($1, 'scroll_return', 2, 0, 0),
              ($1, 'sword_wood', 1, 0, 0),
              ($1, null, 1, 500, 0)
            "#,
        )
        .bind(mail_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn get_for_character(&self, character_id: i64, mail_id: i64) -> Result<PlayerMailView, sqlx::Error> {
        let row = sqlx::query_as::<_, MailRow>(
            r#"
            select id,
                   to_character_id,
                   from_name,
                   title,
                   body,
                   read,
                   claimed,
                   expires_at::text as expires_at,
                   created_at::text as created_at
            from mails
            where id = $1
              and to_character_id = $2
              and (expires_at is null or expires_at > now())
            "#,
        )
        .bind(mail_id)
        .bind(character_id)
        .fetch_one(&self.pool)
        .await?;
        let attachments = self.attachments_for_mail(row.id).await?;
        Ok(mail_view(row, attachments))
    }

    async fn attachments_for_mail(&self, mail_id: i64) -> Result<Vec<PlayerMailAttachmentView>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MailAttachmentRow>(
            r#"
            select ma.id,
                   ma.item_template_id,
                   it.name as item_name,
                   ma.quantity,
                   ma.gold,
                   ma.yuanbao,
                   ma.claimed_at is not null as claimed
            from mail_attachments ma
            left join item_templates it on it.id = ma.item_template_id
            where ma.mail_id = $1
            order by ma.id asc
            "#,
        )
        .bind(mail_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(attachment_view).collect())
    }
}

async fn grant_item_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    template_id: &str,
    quantity: i64,
) -> Result<bool, sqlx::Error> {
    let quantity = quantity.max(1);
    let template: Option<(bool,)> = sqlx::query_as("select stackable from item_templates where id = $1")
        .bind(template_id)
        .fetch_optional(&mut **tx)
        .await?;
    let Some((stackable,)) = template else {
        return Ok(false);
    };

    if stackable {
        let result = sqlx::query(
            r#"
            update inventory_items
            set quantity = quantity + $3
            where id = (
              select id
              from inventory_items
              where character_id = $1 and template_id = $2 and location = 'bag'
              order by id asc
              limit 1
            )
            "#,
        )
        .bind(character_id)
        .bind(template_id)
        .bind(quantity)
        .execute(&mut **tx)
        .await?;
        if result.rows_affected() > 0 {
            return Ok(true);
        }
    }

    sqlx::query(
        r#"
        insert into inventory_items (character_id, template_id, quantity, location)
        values ($1, $2, $3, 'bag')
        "#,
    )
    .bind(character_id)
    .bind(template_id)
    .bind(quantity)
    .execute(&mut **tx)
    .await?;
    Ok(true)
}

fn mail_view(row: MailRow, attachments: Vec<PlayerMailAttachmentView>) -> PlayerMailView {
    PlayerMailView {
        id: row.id,
        to_character_id: row.to_character_id,
        from_name: row.from_name,
        title: row.title,
        body: row.body,
        read: row.read,
        claimed: row.claimed,
        expires_at: row.expires_at,
        created_at: row.created_at,
        attachments,
    }
}

fn attachment_view(row: MailAttachmentRow) -> PlayerMailAttachmentView {
    PlayerMailAttachmentView {
        id: row.id,
        item_template_id: row.item_template_id,
        item_name: row.item_name,
        quantity: row.quantity,
        gold: row.gold,
        yuanbao: row.yuanbao,
        claimed: row.claimed,
    }
}
