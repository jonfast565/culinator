// Sample recipes shown in a fresh library so the app is never empty on first
// launch. These are Alton Brown recipes converted into the Culinograph DSL for
// demonstration; each carries a `source`/`source_url`/`attribution` credit that
// the narrative pane surfaces. See the source links for the originals.
export interface SeedRecipe {
  symbol: string;
  title: string;
  sourceText: string;
}

export const seedRecipes: SeedRecipe[] = [
  {
    symbol: "baked_macaroni_and_cheese",
    title: "Baked Macaroni and Cheese",
    sourceText: `culinograph 0.3;

recipe baked_macaroni_and_cheese {
    title "Baked Macaroni and Cheese";

    source "Alton Brown";
    publisher "Food Network / altonbrown.com";
    source_url "https://altonbrown.com/recipes/baked-macaroni-and-cheese/";
    attribution "Recipe by Alton Brown. Included as sample data; see the source link for the original.";

    ingredient macaroni measured by mass {
        name "elbow macaroni";
        quantity 0.5 lb;
    }
    ingredient butter measured by mass {
        name "unsalted butter";
        quantity 6 tbsp;
        allergen milk;
    }
    ingredient flour measured by mass {
        name "all-purpose flour";
        quantity 3 tbsp;
    }
    ingredient mustard measured by mass {
        name "ground mustard";
        quantity 1 tbsp;
    }
    ingredient paprika measured by mass {
        name "paprika";
        quantity 1 tsp;
    }
    ingredient half_and_half measured by volume {
        name "half and half";
        quantity 1.5 pint;
        allergen milk;
    }
    ingredient cheddar measured by mass {
        name "sharp cheddar";
        quantity 4 oz;
        allergen milk;
        state grated;
    }
    ingredient jack measured by mass {
        name "Monterey jack";
        quantity 8 oz;
        allergen milk;
        state grated;
    }
    ingredient panko measured by mass {
        name "panko breadcrumbs";
        quantity 1 cup;
    }

    material cooked_macaroni measured by mass { }
    material cheese_sauce measured by mass { }
    material sauce_mix measured by mass { }
    material assembled_dish measured by mass { }
    material topped_dish measured by mass { }

    process pasta {
        operation boil does heat {
            input [macaroni];
            heat high;
            duration 8 minutes to 10 minutes;
            until visual "al dente";
            labor monitor;
            produces cooked_macaroni;
        }
    }
    process sauce {
        operation make_sauce does heat {
            input [butter, flour, mustard, paprika, half_and_half];
            heat medium;
            duration 10 min to 12 min;
            until visual "thick enough to coat the back of a spoon";
            labor active;
            produces cheese_sauce;
        }
        operation stir_in_cheese does mix {
            input [cheese_sauce, cheddar, jack];
            after make_sauce;
            duration 2 min;
            labor active;
            produces sauce_mix;
        }
    }
    process assembly {
        operation combine does mix {
            input [sauce_mix, cooked_macaroni];
            after [stir_in_cheese, boil];
            duration 2 min;
            labor active;
            produces assembled_dish;
        }
        operation top does mix {
            input [assembled_dish, panko];
            after combine;
            duration 2 min;
            labor active;
            produces topped_dish;
        }
        operation bake does heat {
            input [topped_dish];
            after top;
            temperature 350 fahrenheit;
            duration 25 minutes to 30 minutes;
            until visual "golden and bubbly";
            labor passive;
        }
    }

    yield servings measured by mass {
        mass 1400 g;
    }
}
`,
  },
  {
    symbol: "easy_crepes",
    title: "Easy Crepes",
    sourceText: `culinograph 0.3;

recipe easy_crepes {
    title "Easy Crepes";

    source "Alton Brown";
    publisher "Food Network / altonbrown.com";
    source_url "https://altonbrown.com/recipes/easy-crepes/";
    attribution "Recipe by Alton Brown. Included as sample data; see the source link for the original.";

    ingredient eggs measured by count {
        name "large eggs";
        quantity 2 count;
        allergen egg;
    }
    ingredient milk measured by volume {
        name "milk";
        quantity 0.75 cup;
        allergen milk;
    }
    ingredient water measured by volume {
        name "water";
        quantity 0.5 cup;
    }
    ingredient flour measured by mass {
        name "all-purpose flour";
        quantity 1 cup;
    }
    ingredient butter measured by mass {
        name "unsalted butter";
        quantity 3 tbsp;
        allergen milk;
        state melted;
    }
    ingredient salt measured by mass {
        name "kosher salt";
        quantity 1 pinch;
    }

    material crepe_batter measured by volume { }
    material rested_batter measured by volume { }

    process batter {
        operation blend does mix {
            input [eggs, milk, water, flour, butter, salt];
            duration 1 min;
            labor active;
            produces crepe_batter;
        }
        operation chill does rest {
            input [crepe_batter];
            after blend;
            duration 1 hour to 8 hours;
            labor passive;
            produces rested_batter;
        }
    }
    process cooking {
        operation cook does heat {
            input [rested_batter];
            after chill;
            heat medium;
            duration 1 minute to 2 minutes;
            until visual "golden at the edges";
            labor active;
        }
    }

    yield crepes measured by count {
        amount 11 count;
    }
}
`,
  },
  {
    symbol: "fully_loaded_guacamole",
    title: "Fully Loaded Guacamole",
    sourceText: `culinograph 0.3;

recipe fully_loaded_guacamole {
    title "Fully Loaded Guacamole";

    source "Alton Brown";
    publisher "Food Network / altonbrown.com";
    source_url "https://altonbrown.com/recipes/fully-loaded-guacamole/";
    attribution "Recipe by Alton Brown. Included as sample data; see the source link for the original.";

    ingredient avocados measured by count {
        name "ripe Hass avocados";
        quantity 3 count;
        state ripe;
    }
    ingredient lime_juice measured by volume {
        name "fresh lime juice";
        quantity 1 tbsp to 2 tbsp;
    }
    ingredient salt measured by mass {
        name "kosher salt";
        quantity 0.5 tsp;
    }
    ingredient cumin measured by mass {
        name "ground cumin";
        quantity 0.5 tsp;
    }
    ingredient cayenne measured by mass {
        name "ground cayenne";
        quantity 1 pinch;
    }
    ingredient onion measured by count {
        name "medium onion";
        quantity 0.5 count;
    }
    ingredient tomatoes measured by count {
        name "Roma tomatoes";
        quantity 2 count;
    }
    ingredient cilantro measured by mass {
        name "fresh cilantro";
        quantity 1 tbsp;
        optional true;
    }
    ingredient jalapeno measured by count {
        name "jalapeno";
        quantity 0.5 count to 1 count;
    }
    ingredient garlic measured by count {
        name "garlic clove";
        quantity 1 clove;
    }

    // \`mashed_avocado\` is declared so it can carry a state; the diced/minced prep
    // products are left implicit and become intermediates automatically.
    material mashed_avocado measured by mass {
        state smooth;
    }

    // Prep is knife work only: none of these steps depend on one another, so the
    // scheduler is free to run them in any order (or hand them out in parallel).
    // \`prep <verb> <ingredient> into <material>\` desugars to an operation named
    // \`<verb>_<ingredient>\` that inputs the ingredient and produces the material.
    process prep {
        prep dice onion into diced_onion { duration 2 min; }
        prep dice tomatoes into diced_tomatoes { duration 3 min; }
        prep chop cilantro into chopped_cilantro { duration 1 min; }
        prep mince jalapeno into minced_jalapeno { duration 2 min; }
        prep mince garlic into minced_garlic { duration 1 min; }
    }

    // Mashing the avocados is also independent of the knife work, so it can
    // overlap with prep; only \`fold\` waits on everything upstream.
    process mixing {
        operation mash does mix {
            input [avocados, lime_juice, salt, cumin, cayenne];
            duration 3 min;
            labor active;
            produces mashed_avocado;
        }
        operation fold does mix {
            input [mashed_avocado, diced_onion, diced_tomatoes, chopped_cilantro, minced_jalapeno, minced_garlic];
            after [mash, dice_onion, dice_tomatoes, chop_cilantro, mince_jalapeno, mince_garlic];
            duration 2 min;
            labor active;
            produces guacamole_mix;
        }
    }

    process resting {
        operation rest does rest {
            input [guacamole_mix];
            after fold;
            duration 120 min;
            labor passive;
        }
    }

    yield servings measured by count {
        amount 4 count;
    }
}
`,
  },
];
