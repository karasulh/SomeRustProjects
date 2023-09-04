use crate::schema::items;
use serde_derive::*;
//schema macro generates type safe table object and creates a seperate internal module for each database table.
//queryable and insertable are two main derivable traits for Diesel.
//PGConnection: handle representing an open connection to PostGres database

#[derive(Queryable,Serialize, Deserialize, Clone,Debug)]
pub struct Item {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub price: i32,
    pub instock:i32
}

#[derive(Queryable,Insertable,Clone,Debug)]
#[table_name = "items"]
pub struct NewItem<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub price: i32,
}