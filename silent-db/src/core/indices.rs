pub trait IndexTypeTrait {
    fn get_type_str(&self) -> String;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum IndexSort {
    ASC,
    DESC,
}

impl IndexSort {
    pub fn to_str(&self) -> &str {
        match self {
            IndexSort::ASC => "ASC",
            IndexSort::DESC => "DESC",
        }
    }
}

pub trait IndexTrait {
    fn get_alias(&self) -> Option<String>;
    fn get_type(&self) -> Box<dyn IndexTypeTrait>;
    fn get_fields(&self) -> Vec<String>;
    fn get_sort(&self) -> IndexSort {
        IndexSort::ASC
    }
    fn get_create_sql(&self) -> String {
        if self.get_fields().is_empty() {
            panic!("Index fields is empty");
        }
        let fields = self
            .get_fields()
            .iter()
            .map(|f| format!("`{}`", f))
            .collect::<Vec<String>>()
            .join(",");
        let mut sql = self.get_type().get_type_str().to_string();
        if let Some(alias) = self.get_alias() {
            sql.push_str(&format!(" `{}`", alias));
        }
        sql.push_str(&format!(" ({})", fields));
        sql.push_str(&format!(" {}", self.get_sort().to_str()));
        sql
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Eq, PartialEq)]
    struct TestIndex {
        alias: Option<String>,
        index_type: TestIndexType,
        fields: Vec<String>,
        sort: IndexSort,
    }

    impl IndexTrait for TestIndex {
        fn get_alias(&self) -> Option<String> {
            self.alias.clone()
        }

        fn get_type(&self) -> Box<dyn IndexTypeTrait> {
            Box::new(self.index_type.clone())
        }

        fn get_fields(&self) -> Vec<String> {
            self.fields.clone()
        }

        fn get_sort(&self) -> IndexSort {
            self.sort.clone()
        }
    }

    #[allow(dead_code)]
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum TestIndexType {
        Unique,
        Index,
        FullText,
        Spatial,
    }

    impl IndexTypeTrait for TestIndexType {
        fn get_type_str(&self) -> String {
            match self {
                TestIndexType::Unique => "UNIQUE KEY",
                TestIndexType::Index => "INDEX",
                TestIndexType::FullText => "FULLTEXT KEY",
                TestIndexType::Spatial => "SPATIAL KEY",
            }
            .to_string()
        }
    }

    #[test]
    fn test_int_field() {
        let index = TestIndex {
            alias: Some("idx".to_string()),
            index_type: TestIndexType::Unique,
            fields: vec!["id".to_string()],
            sort: IndexSort::ASC,
        };
        assert_eq!(index.get_alias().unwrap(), "idx");
        assert_eq!(index.get_fields(), vec!["id"]);
        assert_eq!(index.get_type().get_type_str(), "UNIQUE KEY");
        assert_eq!(index.get_create_sql(), "UNIQUE KEY `idx` (`id`) ASC");
    }
}
