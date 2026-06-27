create index if not exists guild_applications_pending_guild_idx
  on guild_applications (guild_id, created_at, id)
  where status = 'pending';

create index if not exists guild_members_character_role_idx
  on guild_members (character_id, role);
