use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_raw(sea_orm::Statement::from_string(
            manager.get_database_backend(),
            "CREATE SCHEMA IF NOT EXISTS auth;".to_string(),
        ))
        .await?;
        db.execute_raw(sea_orm::Statement::from_string(
            manager.get_database_backend(),
            "CREATE SCHEMA IF NOT EXISTS mgmt;".to_string(),
        ))
        .await?;
        db.execute_raw(sea_orm::Statement::from_string(
            manager.get_database_backend(),
            "CREATE SCHEMA IF NOT EXISTS app;".to_string(),
        ))
        .await?;

        manager
            .create_table(
                Table::create()
                    .table(("auth", "user"))
                    .if_not_exists()
                    .col(big_pk_auto("id"))
                    .col(string("username").unique_key())
                    .col(string("password_hash"))
                    .col(boolean("enabled"))
                    .col(timestamp_with_time_zone("created_at"))
                    .col(uuid("security_stamp"))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(("auth", "role"))
                    .if_not_exists()
                    .col(big_pk_auto("id"))
                    .col(string("name").unique_key())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(("auth", "user_role"))
                    .if_not_exists()
                    .col(big_integer("user_id"))
                    .col(big_integer("role_id"))
                    .primary_key(
                        Index::create()
                            .name("pk-user_role")
                            .col("user_id")
                            .col("role_id"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_role-user_id")
                            .from_tbl(("auth", "user_role"))
                            .from_col("user_id")
                            .to_tbl(("auth", "user"))
                            .to_col("id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_role-role_id")
                            .from_tbl(("auth", "user_role"))
                            .from_col("role_id")
                            .to_tbl(("auth", "role"))
                            .to_col("id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(("auth", "user_role"))
                    .name("idx-user_role-role_id")
                    .col("role_id")
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(("mgmt", "job"))
                    .if_not_exists()
                    .col(big_pk_auto("id"))
                    .col(tiny_integer("job_type"))
                    .col(json("job_args"))
                    .col(tiny_integer("status"))
                    .col(string("description"))
                    .col(string("error_message"))
                    .col(boolean("success"))
                    .col(big_integer("created_by_id"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-job-created_by_id")
                            .from_tbl(("mgmt", "job"))
                            .from_col("created_by_id")
                            .to_tbl(("auth", "user"))
                            .to_col("id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(timestamp_with_time_zone("created_at"))
                    .col(timestamp_with_time_zone("completed_at").null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(("mgmt", "job"))
                    .name("idx-job-created_by_id")
                    .col("created_by_id")
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(("mgmt", "storage"))
                    .if_not_exists()
                    .col(big_pk_auto("id"))
                    .col(string("name").unique_key())
                    .col(string("path").unique_key())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(("app", "music_info"))
                    .if_not_exists()
                    .col(binary("id").primary_key())
                    .col(array("artists", ColumnType::String(StringLen::None)).not_null())
                    .col(string("album_title").not_null())
                    .col(string("title").not_null())
                    .col(integer("track_number").not_null())
                    .col(integer("audio_bitrate").not_null())
                    .col(integer("sample_rate").not_null())
                    .col(tiny_integer("channels").not_null())
                    .col(tiny_integer("bit_depth").not_null())
                    .col(big_integer("storage_id").not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-music_info-storage_id")
                            .from_tbl(("app", "music_info"))
                            .from_col("storage_id")
                            .to_tbl(("mgmt", "storage"))
                            .to_col("id")
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Restrict),
                    )
                    .col(string("file_path").not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(("app", "music_info"))
                    .name("idx-music_info-storage_id")
                    .col("storage_id")
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(("app", "music_list"))
                    .if_not_exists()
                    .col(big_pk_auto("id"))
                    .col(string("name").unique_key())
                    .col(big_integer("user_id").not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-music_list-user_id")
                            .from_tbl(("app", "music_list"))
                            .from_col("user_id")
                            .to_tbl(("auth", "user"))
                            .to_col("id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(("app", "music_list"))
                    .name("idx-music_list-user_id")
                    .col("user_id")
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(("app", "music_info_music_list"))
                    .if_not_exists()
                    .col(big_integer("music_list_id").not_null())
                    .col(binary("music_info_id").not_null())
                    .primary_key(
                        Index::create()
                            .name("pk-music_info_music_list")
                            .col("music_list_id")
                            .col("music_info_id"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-music_info_music_list-music_list_id")
                            .from_tbl(("app", "music_info_music_list"))
                            .from_col("music_list_id")
                            .to_tbl(("app", "music_list"))
                            .to_col("id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-music_info_music_list-music_info_id")
                            .from_tbl(("app", "music_info_music_list"))
                            .from_col("music_info_id")
                            .to_tbl(("app", "music_info"))
                            .to_col("id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(string("order").not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(("app", "music_info_music_list"))
                    .name("idx-music_info_music_list-music_list_id_order")
                    .col("music_list_id")
                    .col("order")
                    .unique()
                    .to_owned(),
            )
            .await?;

        let stmt = Query::insert()
            .into_table(("auth", "role"))
            .columns(["name"])
            .values_panic(["admin".into()])
            .to_owned();

        manager.execute(stmt).await?;

        let stmt = Query::insert()
            .into_table(("auth", "role"))
            .columns(["name"])
            .values_panic(["normal-user".into()])
            .to_owned();

        manager.execute(stmt).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(("app", "music_info_music_list"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(("app", "music_list")).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(("app", "music_info")).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(("mgmt", "storage")).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(("mgmt", "job")).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(("auth", "user_role")).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(("auth", "role")).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(("auth", "user")).to_owned())
            .await?;

        let db = manager.get_connection();
        db.execute_raw(sea_orm::Statement::from_string(
            manager.get_database_backend(),
            "DROP SCHEMA IF EXISTS auth;".to_string(),
        ))
        .await?;
        db.execute_raw(sea_orm::Statement::from_string(
            manager.get_database_backend(),
            "DROP SCHEMA IF EXISTS mgmt;".to_string(),
        ))
        .await?;
        db.execute_raw(sea_orm::Statement::from_string(
            manager.get_database_backend(),
            "DROP SCHEMA IF EXISTS app;".to_string(),
        ))
        .await?;

        Ok(())
    }
}
