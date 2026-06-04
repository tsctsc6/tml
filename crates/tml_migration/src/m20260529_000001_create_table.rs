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
                            .name("fk-user_role_user_id")
                            .from_tbl(("auth", "user_role"))
                            .from_col("user_id")
                            .to_tbl(("auth", "user"))
                            .to_col("id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_role_role_id")
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
            .create_type(
                extension::postgres::Type::create()
                    .as_enum(("mgmt", "job_type"))
                    .values([
                        "undefined",
                        "scan_incremental",
                        "build_index",
                        "update_index",
                    ])
                    .to_owned(),
            )
            .await?;
        manager
            .create_type(
                extension::postgres::Type::create()
                    .as_enum(("mgmt", "job_status"))
                    .values(["undefined", "waiting_start", "running", "completed"])
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(("mgmt", "job"))
                    .if_not_exists()
                    .col(big_pk_auto("id"))
                    .col(
                        ColumnDef::new("job_type")
                            .custom("mgmt.job_type")
                            .not_null(),
                    )
                    .col(json("job_args"))
                    .col(
                        ColumnDef::new("job_status")
                            .custom("mgmt.job_status")
                            .not_null(),
                    )
                    .col(string("description"))
                    .col(string("error_message"))
                    .col(boolean("success"))
                    .col(big_integer("created_by_id"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-job_created_by_id")
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
                    .name("idx-job_created_by_id")
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
                    .table(("app", "album"))
                    .if_not_exists()
                    .col(big_pk_auto("id"))
                    .col(string("name").unique_key())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(("app", "music_info"))
                    .if_not_exists()
                    .col(big_pk_auto("id"))
                    .col(string("title").not_null())
                    .col(big_integer("album_id").not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-music_info_album_id")
                            .from_tbl(("app", "music_info"))
                            .from_col("album_id")
                            .to_tbl(("app", "album"))
                            .to_col("id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(integer("album_index").not_null())
                    .col(string("file_path").not_null())
                    .col(big_integer("storage_id").not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-music_info_storage_id")
                            .from_tbl(("app", "music_info"))
                            .from_col("storage_id")
                            .to_tbl(("mgmt", "storage"))
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
                    .table(("app", "music_info"))
                    .name("idx-music_info_album_id")
                    .col("album_id")
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(("app", "music_info"))
                    .name("idx-music_info_storage_id")
                    .col("storage_id")
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(("app", "performer"))
                    .if_not_exists()
                    .col(big_pk_auto("id"))
                    .col(string("name").unique_key())
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
                            .name("fk-music_list_user_id")
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
                    .name("idx-music_list_user_id")
                    .col("user_id")
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(("app", "music_info_performer"))
                    .if_not_exists()
                    .col(big_integer("music_info_id").not_null())
                    .col(big_integer("performer_id").not_null())
                    .primary_key(
                        Index::create()
                            .name("pk-music_info_performer")
                            .col("music_info_id")
                            .col("performer_id"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-music_info_performer_music_info_id")
                            .from_tbl(("app", "music_info_performer"))
                            .from_col("music_info_id")
                            .to_tbl(("app", "music_info"))
                            .to_col("id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-music_info_performer_performer_id")
                            .from_tbl(("app", "music_info_performer"))
                            .from_col("performer_id")
                            .to_tbl(("app", "performer"))
                            .to_col("id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(("app", "music_info_music_list"))
                    .if_not_exists()
                    .col(big_integer("music_info_id").not_null())
                    .col(big_integer("music_list_id").not_null())
                    .primary_key(
                        Index::create()
                            .name("pk-music_info_music_list")
                            .col("music_info_id")
                            .col("music_list_id"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-music_info_music_list_music_info_id")
                            .from_tbl(("app", "music_info_music_list"))
                            .from_col("music_info_id")
                            .to_tbl(("app", "music_info"))
                            .to_col("id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-music_info_music_list_music_list_id")
                            .from_tbl(("app", "music_info_music_list"))
                            .from_col("music_list_id")
                            .to_tbl(("app", "music_list"))
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
                    .name("idx-music_list_id_order")
                    .col("music_list_id")
                    .col("order")
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(("app", "album_performer"))
                    .if_not_exists()
                    .col(big_integer("album_id").not_null())
                    .col(big_integer("performer_id").not_null())
                    .primary_key(
                        Index::create()
                            .name("pk-album_performer")
                            .col("album_id")
                            .col("performer_id"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-album_performer_album_id")
                            .from_tbl(("app", "album_performer"))
                            .from_col("album_id")
                            .to_tbl(("app", "album"))
                            .to_col("id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-album_performer_performer_id")
                            .from_tbl(("app", "album_performer"))
                            .from_col("performer_id")
                            .to_tbl(("app", "performer"))
                            .to_col("id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
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
            .drop_table(Table::drop().table(("app", "album_performer")).to_owned())
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(("app", "music_info_music_list"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(("app", "music_info_performer"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(("app", "music_list")).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(("app", "performer")).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(("app", "music_info")).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(("app", "album")).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(("mgmt", "storage")).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(("mgmt", "job")).to_owned())
            .await?;
        manager
            .drop_type(
                extension::postgres::Type::drop()
                    .if_exists()
                    .name(("mgmt", "job_status"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_type(
                extension::postgres::Type::drop()
                    .if_exists()
                    .name(("mgmt", "job_type"))
                    .to_owned(),
            )
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
