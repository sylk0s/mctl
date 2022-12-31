use firestore::{FirestoreDb, FirestoreQueryParams};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub fn config_env_var(name: &str) -> Result<String, String> {
   std::env::var(name).map_err(|e| format!("{}: {}", name, e))
}

#[async_trait]
// Can sync with firebase
pub trait CloudSync where for<'a> Self: Deserialize<'a> + Unique + Serialize + Sync + Send {
    // Save an object [obj] to a specific [collection]
    async fn clsave(&self, collection: &'static str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let db = FirestoreDb::new(&config_env_var("PROJECT_ID")?).await?;   
        db.delete_by_id(collection, self.uuid().to_string()).await?;
        db.create_obj(collection, self.uuid().to_string(), self).await?;
        Ok(())
    }

    // Remove a specific object
    async fn clrm(&self, collection: &'static str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let db = FirestoreDb::new(&config_env_var("PROJECT_ID")?).await?;   
        db.delete_by_id(collection, self.uuid().to_string()).await?;
        Ok(())
    }

    // Get all objects from a field
    async fn clget() ->  Result<Vec<Self>, Box<dyn Error + Send + Sync>> {
        let db = FirestoreDb::new(&config_env_var("PROJECT_ID")?).await?;
        let objects: Vec<Self> = db.query_obj(FirestoreQueryParams::new(Self::clname().into())).await?;
        Ok(objects)
    }

    // Get the name associated with a type implemeneting this trait.
    fn clname() -> &'static str;
}

pub trait Unique {
    fn uuid(&self) -> String;
}
