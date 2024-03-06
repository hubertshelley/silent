use crate::core::indices::{IndexSort, IndexTrait, IndexTypeTrait};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum IndexType {
    Unique,
    Index,
    FullText,
    Spatial,
}

impl IndexTypeTrait for IndexType {
    fn get_type_str(&self) -> String {
        match self {
            IndexType::Unique => "UNIQUE KEY",
            IndexType::Index => "INDEX",
            IndexType::FullText => "FULLTEXT KEY",
            IndexType::Spatial => "SPATIAL KEY",
        }
        .to_string()
    }
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Index {
    pub alias: Option<String>,
    pub index_type: IndexType,
    pub fields: Vec<String>,
    pub sort: IndexSort,
}

impl IndexTrait for Index {
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
