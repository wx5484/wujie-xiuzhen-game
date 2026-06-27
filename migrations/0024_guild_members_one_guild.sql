create unique index if not exists guild_members_one_guild_per_character
  on guild_members(character_id);
