use crate::core::tables::TableUtil;
use crate::Table;
use anyhow::{anyhow, Result};
use sqlx::{Database, Pool};

pub struct Migrate<DB: Database> {
    pub(crate) migrations_path: String,
    pub(crate) pool: Pool<DB>,
}

impl<DB: Database> Migrate<DB> {
    pub fn new(migrations_path: String, pool: Pool<DB>) -> Self {
        Migrate {
            migrations_path,
            pool,
        }
    }
    pub async fn migrate(&self, tables: Vec<impl Table>) -> Result<()> {
        println!("run migrate");
        let _ = tables;
        //         let sql = format!(
        //             r#"BEGIN;
        // {}
        // COMMIT;"#,
        //             tables
        //                 .iter()
        //                 .map(|table| table.get_create_sql())
        //                 .collect::<Vec<String>>()
        //                 .join("\n")
        //         );
        //         sqlx::query(&sql).execute(pool).await?;
        Ok(())
    }

    pub async fn update(&self) {
        println!("run update");
    }

    pub async fn rollback(&self) {
        println!("run rollback");
    }
    pub async fn generate(&self, utils: Box<dyn TableUtil>) -> Result<()> {
        println!("run generate");
        let sql = utils.get_all_tables();
        println!("sql: {}", sql);
        // let tables = sqlx::query_as(&sql)
        //     .fetch_all(&pool)
        //     .await
        //     .map_err(|e| anyhow!("{}", e))?;
        // println!("tables: {:?}", tables);
        // for table in tables {
        //     let create_sql = sqlx::query_as(&utils.get_table(&table))
        //         .fetch_all(&pool)
        //         .await?;
        //     println!("create_sql: {:?}", create_sql);
        // }
        Ok(())
    }
}
