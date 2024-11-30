use rusqlite::{Connection, Result, params};
use anyhow::Context;

#[derive(Debug)]
pub struct Recipe {
    pub id: Option<i64>,
    pub title: String,
    pub link: String,
    pub category: Option<String>,
}

#[derive(Debug)]
pub struct Ingredient {
    pub id: Option<i64>,
    pub name: String,
    pub recipe_id: i64,
    pub have: bool,
}

pub struct RecipeDatabase {
    conn: Connection,
}

impl RecipeDatabase {
    pub fn new() -> Result<Self> {
        let conn = Connection::open("recipes.db")?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS recipes (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                link TEXT NOT NULL,
                category TEXT
            )", [])?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ingredients (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                recipe_id INTEGER,
                have INTEGER DEFAULT 0,
                FOREIGN KEY(recipe_id) REFERENCES recipes(id)
            )", [])?;
        
        Ok(RecipeDatabase { conn })
    }
    
    pub fn add_recipe(&self, recipe: &Recipe) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO recipes (title, link, category) VALUES (?1, ?2, ?3)",
            params![recipe.title, recipe.link, recipe.category]
        )?;
        
        Ok(self.conn.last_insert_rowid())
    }
    
    pub fn add_ingredients(&self, recipe_id: i64, ingredients: &[String]) -> Result<()> {
        for ingredient in ingredients {
            self.conn.execute(
                "INSERT INTO ingredients (name, recipe_id) VALUES (?1, ?2)",
                params![ingredient, recipe_id]
            )?;
        }
        Ok(())
    }
    
    pub fn get_shopping_list(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM ingredients WHERE have = 0"
        )?;
        
        let ingredient_iter = stmt.query_map([], |row| {
            row.get(0)
        })?;
        
        let mut shopping_list = Vec::new();
        for ingredient in ingredient_iter {
            shopping_list.push(ingredient?);
        }
        
        Ok(shopping_list)
    }
    
    pub fn mark_ingredient(&self, ingredient_name: &str, have: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE ingredients SET have = ?1 WHERE name = ?2",
            params![have as i32, ingredient_name]
        )?;
        Ok(())
    }
}
