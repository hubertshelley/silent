use crate::core::dsl::SqlStatement;
use crate::core::tables::TableUtil;
use crate::Table;
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use console::style;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use sqlx::any::AnyConnectOptions;
use sqlx::migrate::{MigrationType, Migrator};
use sqlx::{Acquire, AnyConnection, ConnectOptions, FromRow};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct Migrate {
    pub(crate) migrations_path: String,
    pub(crate) options: AnyConnectOptions,
    pub(crate) conn: Option<AnyConnection>,
}

#[derive(Debug, FromRow)]
struct TableName(String);

#[derive(Debug, FromRow)]
struct TableCreate(String, String);
impl Migrate {
    pub fn new(migrations_path: String, options: AnyConnectOptions) -> Self {
        Migrate {
            migrations_path,
            options,
            conn: None,
        }
    }
    #[inline]
    async fn connect(&mut self) -> Result<()> {
        if self.conn.is_none() {
            sqlx::any::install_default_drivers();
            let conn = self.options.connect().await?;
            self.conn = Some(conn);
        }
        Ok(())
    }
    #[inline]
    async fn get_conn(mut self) -> Result<AnyConnection> {
        if self.conn.is_none() {
            self.connect().await?;
        }
        self.conn.ok_or(anyhow!("no connection"))
    }
    #[inline]
    async fn get_migrator(&self) -> Result<Migrator> {
        Migrator::new(Path::new(&self.migrations_path))
            .await
            .map_err(|e| anyhow!("{}", e))
    }
    pub async fn make_migration(&mut self, tables: Vec<impl Table>) -> Result<()> {
        println!("run migrate");
        // todo!()
        let _ = tables;
        Ok(())
    }
    pub async fn migrate(self) -> Result<()> {
        println!("run migrate");
        let migrator = self.get_migrator().await?;
        let mut conn = self.get_conn().await?;
        // let mut transaction = conn.begin().await?;
        migrator
            .run(&mut conn)
            .await
            .map_err(|e| anyhow!("migrate error: {}", e))
        //     ?;
        // match result {
        //     Ok(_) => transaction.commit().await?,
        //     Err(err) => {
        //         transaction.rollback().await?;
        //         return Err(anyhow!("migrate error: {}", err));
        //     }
        // }
        // Ok(())
    }

    pub async fn rollback(self, target: i64) -> Result<()> {
        println!("run rollback");
        let migrator = self.get_migrator().await?;
        let mut conn = self.get_conn().await?;
        migrator.undo(&mut conn, target).await?;
        Ok(())
    }
    pub async fn generate(self, utils: Box<dyn TableUtil>) -> Result<()> {
        println!("run generate");
        let migrations_path = self.migrations_path.clone();
        let mut conn = self.get_conn().await?;
        let sql = utils.get_all_tables();
        let tables: Vec<TableName> = sqlx::query_as(&sql)
            .fetch_all(&mut conn)
            .await
            .map_err(|e| anyhow!("{}", e))?;
        let path = Path::new(&migrations_path);
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        if !path.is_dir() {
            return Err(anyhow!("migrations path is not a directory"));
        }
        if path.read_dir()?.next().is_some() {
            return Err(anyhow!("migrations path is not empty"));
        }
        let prefix = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let up_path_buf = create_file(
            &migrations_path,
            &prefix,
            "init",
            MigrationType::ReversibleUp,
        )?;
        let down_path_buf = create_file(
            &migrations_path,
            &prefix,
            "init",
            MigrationType::ReversibleDown,
        )?;
        let mut generate_tables: Vec<SqlStatement> = vec![];
        if let Err(()) = {
            for table in tables {
                let create_list: Vec<TableCreate> = sqlx::query_as(&utils.get_table(&table.0))
                    .fetch_all(&mut conn)
                    .await?;
                let dialect = GenericDialect {};
                for create in create_list {
                    let sql = create.1;
                    let ast = Parser::parse_sql(&dialect, &sql)?
                        .pop()
                        .ok_or(anyhow!("failed to parse sql"))?;
                    generate_tables.push((ast, sql.to_string()).into());
                }
            }
            let mut up_file = OpenOptions::new().append(true).open(&up_path_buf)?;
            generate_tables.sort();
            for sql in &generate_tables {
                up_file.write_all(sql.1.as_bytes())?;
                up_file.write_all(b";\n")?;
            }
            up_file.write_all(sql.as_bytes())?;
            Ok(())
        } {
            fs::remove_file(up_path_buf)?;
            fs::remove_file(down_path_buf)?;
        }
        Ok(())
    }
}
fn create_file(
    migration_source: &str,
    file_prefix: &str,
    description: &str,
    migration_type: MigrationType,
) -> Result<PathBuf> {
    let mut file_name = file_prefix.to_string();
    file_name.push('_');
    file_name.push_str(&description.replace(' ', "_"));
    file_name.push_str(migration_type.suffix());

    let mut path = PathBuf::new();
    path.push(migration_source);
    path.push(&file_name);

    println!("Creating {}", style(path.display()).cyan());

    let mut file = File::create(&path).context("Failed to create migration file")?;

    Write::write_all(&mut file, migration_type.file_content().as_bytes())?;

    Ok(path)
}
