use std::ops::{Add, BitAnd, BitOr};

#[derive(Debug)]
pub struct QueryBuilder {
    field: String,
    action: String,
    value: String,
}

impl QueryBuilder {
    pub fn get_sql(&self) -> String {
        let field = if self.field.is_empty() || self.field.starts_with('(') {
            self.field.clone()
        } else {
            format!("`{}`", self.field)
        };
        let action = if self.action.is_empty() {
            self.action.clone()
        } else {
            format!("{} ", self.action)
        };
        format!("{} {}{}", field, action, self.value)
    }
}

#[derive(Debug)]
pub struct QueryBuilderGroup {
    group: Vec<QueryBuilder>,
}

impl QueryBuilderGroup {
    pub fn get_sql(&self) -> String {
        self.group
            .iter()
            .map(|builder| builder.get_sql())
            .collect::<Vec<String>>()
            .join(" AND ")
    }

    pub fn get_query(&self) -> String {
        format!("WHERE {}", self.get_sql())
    }
}

impl Add for QueryBuilderGroup {
    type Output = QueryBuilderGroup;

    fn add(self, other: QueryBuilderGroup) -> QueryBuilderGroup {
        let QueryBuilderGroup { mut group } = self;
        group.extend(other.group);
        QueryBuilderGroup { group }
    }
}

impl BitAnd for QueryBuilderGroup {
    type Output = QueryBuilderGroup;

    fn bitand(self, other: QueryBuilderGroup) -> QueryBuilderGroup {
        let builder = QueryBuilder {
            field: format!("({})", self.get_sql()),
            action: "AND".to_string(),
            value: format!("({})", other.get_sql()),
        };
        QueryBuilderGroup {
            group: vec![builder],
        }
    }
}

impl BitOr for QueryBuilderGroup {
    type Output = QueryBuilderGroup;

    fn bitor(self, other: QueryBuilderGroup) -> QueryBuilderGroup {
        let builder = QueryBuilder {
            field: format!("({})", self.get_sql()),
            action: "OR".to_string(),
            value: format!("({})", other.get_sql()),
        };
        QueryBuilderGroup {
            group: vec![builder],
        }
    }
}

pub trait Query {
    fn get_field() -> String;
    #[inline]
    fn parse<T>(value: T) -> String
    where
        T: ToString,
    {
        match value.to_string().parse::<f64>().map(|_| value.to_string()) {
            Ok(value) => value,
            Err(_) => format!("'{}'", value.to_string()),
        }
    }
    fn query_eq<T>(value: T) -> QueryBuilderGroup
    where
        T: ToString,
    {
        QueryBuilderGroup {
            group: vec![QueryBuilder {
                field: Self::get_field(),
                action: "=".to_string(),
                value: Self::parse(value),
            }],
        }
    }
    fn query_ne<T>(value: T) -> QueryBuilderGroup
    where
        T: ToString,
    {
        QueryBuilderGroup {
            group: vec![QueryBuilder {
                field: Self::get_field(),
                action: "!=".to_string(),
                value: Self::parse(value),
            }],
        }
    }
    fn query_gt<T>(value: T) -> QueryBuilderGroup
    where
        T: ToString,
    {
        QueryBuilderGroup {
            group: vec![QueryBuilder {
                field: Self::get_field(),
                action: ">".to_string(),
                value: Self::parse(value),
            }],
        }
    }
    fn gte<T>(value: T) -> QueryBuilderGroup
    where
        T: ToString,
    {
        QueryBuilderGroup {
            group: vec![QueryBuilder {
                field: Self::get_field(),
                action: ">=".to_string(),
                value: Self::parse(value),
            }],
        }
    }
    fn query_lt<T>(value: T) -> QueryBuilderGroup
    where
        T: ToString,
    {
        QueryBuilderGroup {
            group: vec![QueryBuilder {
                field: Self::get_field(),
                action: "<".to_string(),
                value: Self::parse(value),
            }],
        }
    }
    fn lte<T>(value: T) -> QueryBuilderGroup
    where
        T: ToString,
    {
        QueryBuilderGroup {
            group: vec![QueryBuilder {
                field: Self::get_field(),
                action: "<=".to_string(),
                value: Self::parse(value),
            }],
        }
    }
    fn like<T>(value: T) -> QueryBuilderGroup
    where
        T: ToString,
    {
        QueryBuilderGroup {
            group: vec![QueryBuilder {
                field: Self::get_field(),
                action: "LIKE".to_string(),
                value: format!("'%{}%'", value.to_string()),
            }],
        }
    }
    fn starts_with<T>(value: T) -> QueryBuilderGroup
    where
        T: ToString,
    {
        QueryBuilderGroup {
            group: vec![QueryBuilder {
                field: Self::get_field(),
                action: "LIKE".to_string(),
                value: format!("'{}%'", value.to_string()),
            }],
        }
    }
    fn ends_with<T>(value: T) -> QueryBuilderGroup
    where
        T: ToString,
    {
        QueryBuilderGroup {
            group: vec![QueryBuilder {
                field: Self::get_field(),
                action: "LIKE".to_string(),
                value: format!("'%{}'", value.to_string()),
            }],
        }
    }

    fn between<T>(value: (T, T)) -> QueryBuilderGroup
    where
        T: ToString,
    {
        QueryBuilderGroup {
            group: vec![QueryBuilder {
                field: Self::get_field(),
                action: "BETWEEN".to_string(),
                value: format!("{} AND {}", Self::parse(value.0), Self::parse(value.1)),
            }],
        }
    }
    fn r#in<T>(value: Vec<T>) -> QueryBuilderGroup
    where
        T: ToString,
    {
        QueryBuilderGroup {
            group: vec![QueryBuilder {
                field: Self::get_field(),
                action: "IN".to_string(),
                value: format!(
                    "({})",
                    value
                        .iter()
                        .map(|v| format!("'{}'", v.to_string()))
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
            }],
        }
    }
    fn is_null() -> QueryBuilderGroup {
        QueryBuilderGroup {
            group: vec![QueryBuilder {
                field: Self::get_field(),
                action: "IS".to_string(),
                value: "NULL".to_string(),
            }],
        }
    }
    fn not_null() -> QueryBuilderGroup {
        QueryBuilderGroup {
            group: vec![QueryBuilder {
                field: Self::get_field(),
                action: "IS NOT".to_string(),
                value: "NULL".to_string(),
            }],
        }
    }
    fn raw<T>(value: T) -> QueryBuilderGroup
    where
        T: ToString,
    {
        QueryBuilderGroup {
            group: vec![QueryBuilder {
                field: "".to_string(),
                action: "".to_string(),
                value: value.to_string(),
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type User = String;
    impl Query for User {
        fn get_field() -> String {
            "user".to_string()
        }
    }

    type Age = u16;
    impl Query for Age {
        fn get_field() -> String {
            "age".to_string()
        }
    }

    #[test]
    fn test_query_builder() {
        let query = User::query_eq("zhangsan") & Age::query_gt(18)
            | User::query_eq("lisi") & Age::query_lt(30);
        assert_eq!(
            query.get_sql(),
            "((`user` = 'zhangsan') AND (`age` > 18)) OR ((`user` = 'lisi') AND (`age` < 30))"
        );
        let query = (User::query_eq("zhangsan") + Age::query_gt(18))
            | User::query_eq("lisi") & Age::query_lt(30);
        assert_eq!(
            query.get_sql(),
            "(`user` = 'zhangsan' AND `age` > 18) OR ((`user` = 'lisi') AND (`age` < 30))"
        );
        let query = User::query_eq("zhangsan") + Age::query_gt(18) + Age::query_lt(30);
        assert_eq!(
            query.get_sql(),
            "`user` = 'zhangsan' AND `age` > 18 AND `age` < 30"
        );
        let query = User::query_eq("zhangsan") & Age::gte(18) & Age::lte(30);
        assert_eq!(
            query.get_sql(),
            "((`user` = 'zhangsan') AND (`age` >= 18)) AND (`age` <= 30)"
        );
        let query = User::like("zhangsan") + Age::between((18, 30));
        assert_eq!(
            query.get_sql(),
            "`user` LIKE '%zhangsan%' AND `age` BETWEEN 18 AND 30"
        );
        let query = User::r#in(vec!["zhangsan", "lisi"]) + Age::query_ne(18);
        assert_eq!(
            query.get_sql(),
            "`user` IN ('zhangsan', 'lisi') AND `age` != 18"
        );
        let query = User::is_null() + Age::not_null();
        assert_eq!(query.get_sql(), "`user` IS NULL AND `age` IS NOT NULL");
        let query = User::starts_with("zhang") + User::ends_with("san");
        assert_eq!(
            query.get_sql(),
            "`user` LIKE 'zhang%' AND `user` LIKE '%san'"
        );
    }
}
