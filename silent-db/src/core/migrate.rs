use crate::core::dsl::SqlStatement;
use crate::core::tables::TableUtil;
use crate::Table;
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use console::style;
use sqlparser::ast::Statement;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use sqlx::any::AnyConnectOptions;
use sqlx::migrate::{MigrationType, Migrator};
use sqlx::{AnyConnection, ConnectOptions, FromRow};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::ops::Deref;
use std::path::{Path, PathBuf};

pub struct Migrate {
    pub(crate) migrations_path: String,
    pub(crate) options: AnyConnectOptions,
    pub(crate) conn: Option<AnyConnection>,
}

#[derive(Debug, FromRow)]
struct TableName(String);

#[allow(dead_code)]
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
        for table in tables {
            table.get_create_sql();
        }
        Ok(())
    }
    pub async fn migrate(self) -> Result<()> {
        println!("run migrate");
        let migrator = self.get_migrator().await?;
        let mut conn = self.get_conn().await?;
        migrator
            .run(&mut conn)
            .await
            .map_err(|e| anyhow!("migrate error: {}", e))
    }

    pub async fn rollback(self, target: i64) -> Result<()> {
        println!("run rollback");
        let migrator = self.get_migrator().await?;
        let mut conn = self.get_conn().await?;
        migrator.undo(&mut conn, target).await?;
        Ok(())
    }
    async fn get_exist_tables(self, utils: &dyn TableUtil) -> Result<Vec<SqlStatement>> {
        let mut conn = self.get_conn().await?;
        if conn.backend_name() != utils.get_name() {
            return Err(anyhow!(
                "database({}) is not support, database({}) is supported",
                conn.backend_name(),
                utils.get_name()
            ));
        }
        let sql = utils.get_all_tables();
        let tables: Vec<TableName> = sqlx::query_as(&sql)
            .fetch_all(&mut conn)
            .await
            .map_err(|e| anyhow!("{}", e))?;
        let mut generate_tables: Vec<SqlStatement> = vec![];
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
                if let Statement::CreateTable { name, .. } = ast.clone() {
                    if name.to_string() == "`_sqlx_migrations`" {
                        continue;
                    }
                }
                generate_tables.push((ast, sql.to_string()).into());
            }
        }
        Ok(generate_tables)
    }
    async fn generate_files(
        self,
        utils: Box<dyn TableUtil>,
        up_path_buf: PathBuf,
        down_path_buf: PathBuf,
    ) -> Result<()> {
        let mut generate_tables = self.get_exist_tables(utils.deref()).await?;
        let mut up_file = OpenOptions::new().append(true).open(&up_path_buf)?;
        let mut down_file = OpenOptions::new().append(true).open(&down_path_buf)?;
        generate_tables.sort();
        for sql in &generate_tables {
            up_file.write_all(sql.1.as_bytes())?;
            up_file.write_all(b";\n")?;
        }
        generate_tables.reverse();
        for sql in &generate_tables {
            let table = utils.transform(sql)?;
            down_file.write_all(table.get_drop_sql().as_bytes())?;
            down_file.write_all(b"\n")?;
        }
        let models_path = Path::new("./src/models");
        utils.generate_models(generate_tables, models_path)?;
        println!(
            "Generate models at {:?} success",
            style(models_path).green()
        );
        Ok(())
    }
    pub async fn generate(self, utils: Box<dyn TableUtil>) -> Result<()> {
        println!("run generate");
        let migrations_path = self.migrations_path.clone();
        let path = Path::new(&migrations_path);
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        if !path.is_dir() {
            return Err(anyhow!("migrations path is not a directory"));
        }
        // TODO: check if the migrations path is empty
        // if path.read_dir()?.next().is_some() {
        //     return Err(anyhow!("migrations path is not empty"));
        // }
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
        match self
            .generate_files(utils, up_path_buf.clone(), down_path_buf.clone())
            .await
        {
            Ok(_) => {
                println!(
                    "Migration files {} generated at ./{}",
                    style(prefix).yellow(),
                    style(migrations_path).green()
                );
                Ok(())
            }
            Err(e) => {
                fs::remove_file(up_path_buf)?;
                fs::remove_file(down_path_buf)?;
                println!(
                    "Migration files {} generate failed by {}",
                    style(prefix).yellow(),
                    style(e.to_string()).red()
                );
                Err(e)
            }
        }
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
