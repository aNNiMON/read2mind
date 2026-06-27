use crate::{
    db_index::{self, DbIndex},
    error::AppError,
    storage::Storage,
};

pub struct BuildIndexJob<'a> {
    pub storage: &'a Storage,
    pub db: &'a DbIndex,
}

impl<'a> BuildIndexJob<'a> {
    pub fn new(storage: &'a Storage, db: &'a DbIndex) -> Self {
        Self { storage, db }
    }

    pub fn run(&self) -> Result<(), AppError> {
        let items = self.storage.list_items()?;
        println!("Indexing {} item(s)", items.len());
        db_index::add_items(self.db, &items)
    }
}
