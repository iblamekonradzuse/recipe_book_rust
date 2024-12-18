use dialoguer::{Input, Select, Confirm, MultiSelect};
use colored::*;
use anyhow::Result;

use crate::htmlscraper::{search_recipes, fetch_recipe_details};
use crate::database::{RecipeDatabase, Recipe};

pub fn run_cli() -> Result<()> {
    let mut db = RecipeDatabase::new()?;

    loop {
        println!("\n{}", "Recipe Manager CLI".blue().bold());
        let options = vec![
            "Search Recipes", 
            "View Saved Recipes",
            "Delete Saved Recipes",
            "View Shopping List", 
            "Add to Shopping List",
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
            2 => delete_saved_recipes(&db)?,
            3 => view_shopping_list(&db)?,
            4 => add_to_shopping_list(&db)?,
            5 => mark_ingredients(&mut db)?,
            6 => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn add_to_shopping_list(db: &RecipeDatabase) -> Result<()> {
    let add_options = vec![
        "Add from Saved Recipes",
        "Add Manually",
        "Cancel"
    ];

    let selection = Select::new()
        .with_prompt("How would you like to add items?")
        .items(&add_options)
        .interact()?;

    match selection {
        0 => add_ingredients_from_recipes(db)?,
        1 => add_manual_ingredients(db)?,
        2 => return Ok(()), // Cancel
        _ => unreachable!(),
    }

    Ok(())
}

fn add_ingredients_from_recipes(db: &RecipeDatabase) -> Result<()> {
    // Fetch saved recipes
    let recipes = db.get_recipes(None)?;

    if recipes.is_empty() {
        println!("\n{}", "No saved recipes found!".green());
        return Ok(());
    }

    // Let user select a recipe
    let recipe_titles: Vec<String> = recipes.iter().map(|r| r.title.clone()).collect();
    
    let recipe_selection = Select::new()
        .with_prompt("Select a recipe to view ingredients")
        .items(&recipe_titles)
        .interact()?;

    let selected_recipe = &recipes[recipe_selection];

    // Fetch ingredients for the selected recipe
    let ingredients = db.get_recipe_ingredients(selected_recipe.id.unwrap())?;

    if ingredients.is_empty() {
        println!("\n{}", "No ingredients found for this recipe!".green());
        return Ok(());
    }

    // Let user select ingredients to add to shopping list
    let selected_ingredient_indices = MultiSelect::new()
        .with_prompt("Select ingredients to add to shopping list")
        .items(&ingredients)
        .interact()?;

    if selected_ingredient_indices.is_empty() {
        println!("\n{}", "No ingredients selected.".green());
        return Ok(());
    }

    // Collect selected ingredients
    let selected_ingredients: Vec<String> = selected_ingredient_indices
        .iter()
        .map(|&idx| ingredients[idx].clone())
        .collect();

    // Confirm adding ingredients
    if Confirm::new()
        .with_prompt(format!("Add {} ingredient(s) to shopping list?", selected_ingredients.len()))
        .interact()?
    {
        // Add ingredients to database
        for ingredient in &selected_ingredients {
            db.add_ingredients(selected_recipe.id.unwrap(), &[ingredient.clone()])?;
        }

        println!("{} ingredient(s) added to shopping list!", selected_ingredients.len());
    }

    Ok(())
}

fn add_manual_ingredients(db: &RecipeDatabase) -> Result<()> {
    // Prompt for multiple ingredients (comma-separated)
    let ingredients_input = Input::<String>::new()
        .with_prompt("Enter ingredients to add (comma-separated)")
        .interact_text()?;

    // Split and trim ingredients
    let ingredients: Vec<String> = ingredients_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if ingredients.is_empty() {
        println!("\n{}", "No ingredients entered.".green());
        return Ok(());
    }

    // Confirm adding ingredients
    if Confirm::new()
        .with_prompt(format!("Add {} ingredient(s) to shopping list?", ingredients.len()))
        .interact()?
    {
        // Add ingredients to database (with a dummy recipe_id of 0 for manual entries)
        for ingredient in &ingredients {
            db.add_ingredients(0, &[ingredient.clone()])?;
        }

        println!("{} ingredient(s) added to shopping list!", ingredients.len());
    }

    Ok(())
}

fn search_and_save_recipe(db: &RecipeDatabase) -> Result<()> {
    // Specify String as the type for Input
    let search_term = Input::<String>::new()
        .with_prompt("Enter recipe search term")
        .interact_text()?;

    // Rest of the function remains the same
    let search_results = search_recipes(&search_term)?;

    if search_results.is_empty() {
        println!("\n{}", "No recipes found!".green());
        return Ok(());
    }

    // Convert search results to a vector of titles
    let result_titles: Vec<String> = search_results.iter().map(|r| r.title.clone()).collect();

    // Let user select a recipe to save
    let selection = Select::new()
        .with_prompt("Select a recipe to save")
        .items(&result_titles)
        .interact()?;

    // Fetch recipe details
    let selected_recipe = &search_results[selection];
    let recipe_details = fetch_recipe_details(&selected_recipe.link)?;

    // Prompt for category with String type
    let category = Input::<String>::new()
        .with_prompt("Enter a category for this recipe (optional)")
        .allow_empty(true)
        .interact_text()?;

    // Create recipe object
    let recipe = Recipe {
        id: None,
        title: selected_recipe.title.clone(),
        link: selected_recipe.link.clone(),
        category: if category.is_empty() { None } else { Some(category) },
        steps: Some(recipe_details.steps.join("\n")),
    };

    // Save recipe to database
    let recipe_id = db.add_recipe(&recipe)?;

    // Save ingredients to database
    db.add_ingredients(recipe_id, &recipe_details.materials)?;

    println!("\n{} {} {}", "Recipe".green(), recipe.title.bold(), "saved successfully!".green());

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

    // Fetch recipes
    let recipes = db.get_recipes(category.as_ref().map(String::as_str))?;

    if recipes.is_empty() {
        println!("\n{}", "No saved recipes found!".green());
        return Ok(());
    }

    // Let user select a recipe to view details
    let recipe_titles: Vec<String> = recipes.iter().map(|r| r.title.clone()).collect();
    
    let selection = Select::new()
        .with_prompt("Select a recipe to view details")
        .items(&recipe_titles)
        .interact()?;

    let selected_recipe = &recipes[selection];

    // Fetch ingredients for the selected recipe
    let ingredients = db.get_recipe_ingredients(selected_recipe.id.unwrap())?;

    // Display recipe details
    println!("\n{}", "Recipe Details:".blue().bold());
    println!("Title: {}", selected_recipe.title);
    
    if let Some(category) = &selected_recipe.category {
        println!("Category: {}", category);
    }

    println!("\n{}:", "Ingredients".blue());
    for ingredient in ingredients {
        println!("- {}", ingredient);
    }

    println!("\n{}:", "Steps".blue());
    if let Some(steps) = &selected_recipe.steps {
        steps.split('\n').enumerate().for_each(|(i, step)| {
            println!("{}. {}", i + 1, step);
        });
    }

    Ok(())
}

fn view_shopping_list(db: &RecipeDatabase) -> Result<()> {
    let shopping_list = db.get_shopping_list()?;
    
    if shopping_list.is_empty() {
        println!("\n{}", "Shopping list is empty!".green());
        return Ok(());
    }

    println!("\n{}", "Shopping List:".blue().bold());
    for (i, ingredient) in shopping_list.iter().enumerate() {
        println!("{}. {}", i + 1, ingredient);
    }

    Ok(())
}

fn delete_saved_recipes(db: &RecipeDatabase) -> Result<()> {
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

    // Reload recipes after deletion to ensure the latest list
    let mut recipes = db.get_recipes(category.as_ref().map(String::as_str))?;

    if recipes.is_empty() {
        println!("\n{}", "No saved recipes found!".green());
        return Ok(());
    }

    // Let user select recipes to delete
    let recipe_titles: Vec<String> = recipes.iter().map(|r| r.title.clone()).collect();
    
    let recipe_selections = MultiSelect::new()
        .with_prompt("Select recipes to delete")
        .items(&recipe_titles)
        .interact()?;

    // Confirm deletion
    if !recipe_selections.is_empty() {
        if Confirm::new()
            .with_prompt(format!("Are you sure you want to delete {} recipe(s)?", recipe_selections.len()))
            .interact()?
        {
            for idx in recipe_selections {
                let recipe = &recipes[idx];
                let recipe_id = recipe.id.unwrap(); // Safe as we retrieved recipes from the database
                db.delete_recipe(recipe_id)?;
                println!("Deleted recipe: {}", recipe.title);
            }
            
            // Reload recipes to reflect deletions
            recipes = db.get_recipes(category.as_ref().map(String::as_str))?;
            
            println!("Selected recipes deleted successfully!");
            
            // If no recipes remain, return
            if recipes.is_empty() {
                println!("\n{}", "No saved recipes remaining!".green());
                return Ok(());
            }
        }
    }

    Ok(())
}


fn mark_ingredients(db: &mut RecipeDatabase) -> Result<()> {
    let shopping_list = db.get_shopping_list()?;
    
    if shopping_list.is_empty() {
        println!("\n{}", "No ingredients to mark!".green());
        return Ok(());
    }

    // Let user select ingredients to mark as bought
    let selected_ingredients = MultiSelect::new()
        .with_prompt("Select ingredients you have bought (Press SPACE to select, ENTER to confirm)")
        .items(&shopping_list)
        .interact()?;

    if !selected_ingredients.is_empty() {
        // Collect the selected ingredients
        let marked_ingredients: Vec<String> = selected_ingredients
            .iter()
            .map(|&idx| shopping_list[idx].clone())
            .collect();

        // Confirm marking ingredients as bought
        if Confirm::new()
            .with_prompt(format!("Do you want to mark {} ingredient(s) as bought?", marked_ingredients.len()))
            .interact()?
        {
            // Mark ingredients as bought and remove from shopping list
            db.mark_and_remove_ingredients(&marked_ingredients)?;

            println!("{} ingredient(s) marked as bought!", marked_ingredients.len());
        }
    }
    
    Ok(())
}
