use rusqlite::{Connection, Result, params};

#[derive(Debug)]
pub struct Recipe {
    pub id: Option<i64>,
    pub title: String,
    pub link: String,
    pub category: Option<String>,
    pub steps: Option<String>,
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
        
        // Create recipes table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS recipes (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                link TEXT NOT NULL,
                category TEXT,
                steps TEXT
            )", [])?;
        
        // Create ingredients table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ingredients (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                recipe_id INTEGER,
                have INTEGER DEFAULT 0,
                FOREIGN KEY(recipe_id) REFERENCES recipes(id) ON DELETE CASCADE
            )", [])?;
        
        Ok(RecipeDatabase { conn })
    }
    
    pub fn add_recipe(&self, recipe: &Recipe) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO recipes (title, link, category, steps) VALUES (?1, ?2, ?3, ?4)",
            params![
                recipe.title, 
                recipe.link, 
                recipe.category, 
                recipe.steps
            ]
        )?;
        
        Ok(self.conn.last_insert_rowid())
    }

    // New method to delete a recipe
    pub fn delete_recipe(&self, recipe_id: i64) -> Result<()> {
        // This will automatically delete associated ingredients due to ON DELETE CASCADE
        self.conn.execute(
            "DELETE FROM recipes WHERE id = ?1",
            params![recipe_id]
        )?;
        
        Ok(())
    }

    pub fn get_recipes(&self, category: Option<&str>) -> Result<Vec<Recipe>> {
        let query = match category {
            Some(_) => "SELECT id, title, link, category, steps FROM recipes WHERE category = ?1",
            None => "SELECT id, title, link, category, steps FROM recipes"
        };

        let mut stmt = self.conn.prepare(query)?;

        let recipe_mapper = |row: &rusqlite::Row| {
            Ok(Recipe {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                link: row.get(2)?,
                category: row.get(3)?,
                steps: row.get(4)?,
            })
        };

        let recipe_iter = match category {
            Some(cat) => stmt.query_map(rusqlite::params![cat], recipe_mapper)?,
            None => stmt.query_map(rusqlite::params![], recipe_mapper)?
        };

        let mut recipes = Vec::new();
        for recipe in recipe_iter {
            recipes.push(recipe?);
        }

        Ok(recipes)
    }

    pub fn get_recipe_ingredients(&self, recipe_id: i64) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM ingredients WHERE recipe_id = ?1"
        )?;
        
        let ingredient_iter = stmt.query_map([recipe_id], |row| {
            row.get(0)
        })?;
        
        let mut ingredients = Vec::new();
        for ingredient in ingredient_iter {
            ingredients.push(ingredient?);
        }
        
        Ok(ingredients)
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

    pub fn mark_multiple_ingredients(&self, ingredient_names: &[String], have: bool) -> Result<()> {
        // Prepare a parameterized query to mark multiple ingredients
        let placeholders: Vec<String> = ingredient_names.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "UPDATE ingredients SET have = {} WHERE name IN ({})",
            have as i32, 
            placeholders.join(", ")
        );

        // Execute the query with all ingredient names
        self.conn.execute(&query, rusqlite::params_from_iter(ingredient_names))?;
        Ok(())
    }

        pub fn mark_and_remove_ingredients(&mut self, ingredient_names: &[String]) -> Result<()> {
        // Start a transaction to ensure atomic operation
        let tx = self.conn.transaction()?;

        // Update ingredients to mark as bought
        let placeholders: Vec<String> = ingredient_names.iter().map(|_| "?".to_string()).collect();
        let update_query = format!(
            "UPDATE ingredients SET have = 1 WHERE name IN ({})",
            placeholders.join(", ")
        );
        tx.execute(&update_query, rusqlite::params_from_iter(ingredient_names))?;

        // Delete ingredients from the ingredients table
        let delete_query = format!(
            "DELETE FROM ingredients WHERE name IN ({})",
            placeholders.join(", ")
        );
        tx.execute(&delete_query, rusqlite::params_from_iter(ingredient_names))?;

        // Commit the transaction
        tx.commit()?;

        Ok(())
    }
}
