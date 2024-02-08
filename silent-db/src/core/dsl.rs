use sqlparser::ast::{Statement, TableConstraint};
use std::cmp::Ordering;
use std::ops::Deref;

#[derive(Debug, Eq, PartialEq)]
pub struct SqlStatement(pub(crate) Statement, pub(crate) String);

impl From<(Statement, String)> for SqlStatement {
    fn from((statement, sql): (Statement, String)) -> Self {
        SqlStatement(statement, sql)
    }
}

impl Deref for SqlStatement {
    type Target = Statement;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl PartialOrd for SqlStatement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SqlStatement {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.0.clone(), other.0.clone()) {
            (
                Statement::CreateTable {
                    name: table_name,
                    constraints,
                    ..
                },
                Statement::CreateTable {
                    name: other_table_name,
                    constraints: other_constraints,
                    ..
                },
            ) => {
                if constraints.iter().any(|c| {
                    if let TableConstraint::ForeignKey { foreign_table, .. } = c {
                        foreign_table == &other_table_name
                    } else {
                        false
                    }
                }) {
                    Ordering::Greater
                } else if other_constraints.iter().any(|c| {
                    if let TableConstraint::ForeignKey { foreign_table, .. } = c {
                        foreign_table == &table_name
                    } else {
                        false
                    }
                }) {
                    Ordering::Less
                } else {
                    table_name.to_string().cmp(&other_table_name.to_string())
                }
            }
            (_, _) => self.0.to_string().cmp(&other.0.to_string()),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use sqlparser::dialect::MySqlDialect;
    use sqlparser::parser::Parser;
    #[test]
    fn test_sql_statement() {
        let mut sql_statements = vec![];
        let sql = r#"CREATE TABLE `sys_auth_client` (
  `id` char(36) NOT NULL COMMENT 'ID',
  `apply_user_id` char(36) NOT NULL COMMENT '申请用户',
  PRIMARY KEY (`id`),
  UNIQUE KEY `client_name` (`client_name`),
  KEY `apply_user` (`apply_user_id`),
  CONSTRAINT `apply_user` FOREIGN KEY (`apply_user_id`) REFERENCES `sys_user` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='应用client表';"#;
        let dialect = MySqlDialect {};
        let statement = Parser::parse_sql(&dialect, sql).unwrap().pop().unwrap();
        let sql_statement = SqlStatement(statement, sql.to_string());
        sql_statements.push(sql_statement);
        let sql = r#"CREATE TABLE `sys_user` (
  `id` char(36) NOT NULL COMMENT 'ID',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='用户表';"#;
        let dialect = MySqlDialect {};
        let statement = Parser::parse_sql(&dialect, sql).unwrap().pop().unwrap();
        let sql_statement = SqlStatement(statement, sql.to_string());
        sql_statements.push(sql_statement);
        sql_statements.sort();
        assert!(sql_statements[0] < sql_statements[1]);
    }
}
