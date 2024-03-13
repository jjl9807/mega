//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;

use crate::db_enums::MergeStatus;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "mega_blob")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub repo_id: i64,
    #[sea_orm(unique)]
    pub blob_id: String,
    pub commit_id: String,
    pub name: String,
    pub mr_id: i64,
    pub status: MergeStatus,
    pub size: i32,
    #[sea_orm(column_type = "Text")]
    pub full_path: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
