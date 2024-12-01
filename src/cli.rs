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
            "View Saved Recipes",
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
            1 => view_saved_recipes(&db)?,
            2 => view_shopping_list(&db)?,
            3 => mark_ingredients(&db)?,
            4 => break,
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

    // Combine steps into a single string
    let steps = recipe_details.steps.join("\n\n");

    let recipe = Recipe {
        id: None,
        title: selected_recipe.title.clone(),
        link: selected_recipe.link.clone(),
        category,
        steps: Some(steps.clone()),
    };

    let recipe_id = db.add_recipe(&recipe)?;
    db.add_ingredients(recipe_id, &recipe_details.materials)?;

    println!("Recipe saved successfully!");
    
    // Ask if user wants to view recipe details
    if Confirm::new()
        .with_prompt("Would you like to view the recipe details?")
        .interact()?
    {
        println!("\n{}", "Recipe Materials:".blue().bold());
        for material in &recipe_details.materials {
            println!("- {}", material);
        }

        println!("\n{}", "Recipe Steps:".blue().bold());
        for (idx, step) in recipe_details.steps.iter().enumerate() {
            println!("{}. {}", idx + 1, step);
        }
    }

    Ok(())
}

fn view_saved_recipes(db: &RecipeDatabase) -> Result<()> {
    // Allow filtering by category
    let filter_category = Confirm::new()
        .with_prompt("Do you want to filter recipes by category?")
        .interact()?;

    let category = if filter_category {
    Some(
        Input::new()
            .with_prompt("Enter category to filter")
            .allow_empty(true)
            .interact_text()?
    )
} else {
    None
};

let recipes = db.get_recipes(category.as_ref().map(String::as_str))?;

    if recipes.is_empty() {
        println!("\n{}", "No saved recipes found!".green());
        return Ok(());
    }

    // Let user select a recipe to view
    let recipe_titles: Vec<String> = recipes.iter().map(|r| r.title.clone()).collect();
    
    let recipe_selection = Select::new()
        .with_prompt("Select a recipe to view")
        .items(&recipe_titles)
        .interact()?;

    let selected_recipe = &recipes[recipe_selection];

    // Fetch and display recipe details
    println!("\n{}", selected_recipe.title.blue().bold());
    if let Some(category) = &selected_recipe.category {
        println!("Category: {}", category);
    }
    println!("Link: {}", selected_recipe.link);

    // Fetch ingredients
    let ingredients = db.get_recipe_ingredients(selected_recipe.id.unwrap())?;
    println!("\n{}", "Ingredients:".blue());
    for ingredient in &ingredients {
        println!("- {}", ingredient);
    }

    // Display steps if available
    if let Some(steps) = &selected_recipe.steps {
        println!("\n{}", "Recipe Steps:".blue());
        for (idx, step) in steps.split("\n\n").enumerate() {
            println!("{}. {}", idx + 1, step);
        }
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
