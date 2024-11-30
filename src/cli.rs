use dialoguer::{Input, Select, Confirm, MultiSelect};
use colored::*;
use anyhow::Result;

use crate::htmlscraper::{search_recipes, fetch_recipe_details};
use crate::database::{RecipeDatabase, Recipe};

pub fn run_cli() -> Result<()> {
    let db = RecipeDatabase::new()?;

    loop {
        println!("\n{}", "Recipe Manager CLI".blue().bold());
        let options = vec![
            "Search Recipes", 
            "View Shopping List", 
            "Mark Ingredients", 
            "Exit"
        ];

        let selection = Select::new()
            .with_prompt("Choose an action")
            .items(&options)
            .interact()?;

        match selection {
            0 => search_and_save_recipe(&db)?,
            1 => view_shopping_list(&db)?,
            2 => mark_ingredients(&db)?,
            3 => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn search_and_save_recipe(db: &RecipeDatabase) -> Result<()> {
    let search_term: String = Input::new()
        .with_prompt("Enter recipe search term")
        .interact_text()?;

    let results = search_recipes(&search_term)?;
    
    let recipe_selection = Select::new()
        .with_prompt("Select a recipe")
        .items(&results.iter().map(|r| r.title.clone()).collect::<Vec<_>>())
        .interact()?;

    let selected_recipe = &results[recipe_selection];
    let recipe_details = fetch_recipe_details(&selected_recipe.link)?;

    let category: Option<String> = Input::new()
        .with_prompt("Enter category for this recipe (optional)")
        .allow_empty(true)
        .interact_text()
        .map(|s: String| if s.is_empty() { None } else { Some(s) })?;

    let recipe = Recipe {
        id: None,
        title: selected_recipe.title.clone(),
        link: selected_recipe.link.clone(),
        category,
    };

    let recipe_id = db.add_recipe(&recipe)?;
    db.add_ingredients(recipe_id, &recipe_details.materials)?;

    println!("Recipe saved successfully!");
    println!("\nMaterials:");
    for material in &recipe_details.materials {
        println!("- {}", material);
    }

    Ok(())
}

fn view_shopping_list(db: &RecipeDatabase) -> Result<()> {
    let shopping_list = db.get_shopping_list()?;
    
    if shopping_list.is_empty() {
        println!("\n{}", "Your shopping list is empty!".green());
        return Ok(());
    }

    println!("\n{}", "Shopping List:".blue().bold());
    for item in &shopping_list {
        println!("- {}", item);
    }

    if Confirm::new()
        .with_prompt("Would you like to clear the shopping list?")
        .interact()? 
    {
        for item in &shopping_list {
            db.mark_ingredient(item, true)?;
        }
        println!("Shopping list cleared!");
    }

    Ok(())
}

fn mark_ingredients(db: &RecipeDatabase) -> Result<()> {
    let shopping_list = db.get_shopping_list()?;
    
    if shopping_list.is_empty() {
        println!("\n{}", "No ingredients to mark!".green());
        return Ok(());
    }

    let selected_ingredients = MultiSelect::new()
        .with_prompt("Select ingredients you have")
        .items(&shopping_list)
        .interact()?;

    for idx in selected_ingredients {
        let ingredient = &shopping_list[idx];
        db.mark_ingredient(ingredient, true)?;
    }
    
    println!("Ingredients marked successfully!");
    Ok(())
}
