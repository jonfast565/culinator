// Sample recipes shown in a fresh library so the app is never empty on first
// launch. These are Alton Brown recipes converted into the Culinator DSL for
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
    sourceText: `culinator 0.3;

recipe baked_macaroni_and_cheese {
    title "Baked Macaroni and Cheese";
    section "Mains";

    source "Alton Brown";
    publisher "Food Network / altonbrown.com";
    source_url "https://altonbrown.com/recipes/baked-macaroni-and-cheese/";
    attribution "Recipe by Alton Brown. Included as sample data; see the source link for the original.";

    ingredient macaroni measured by mass {
        name "elbow macaroni";
        quantity 8 oz;
    }
    // "1 tablespoon plus 1/2 teaspoon kosher salt, divided": one ingredient
    // split across steps. Per-step amounts ride the operation bindings, so no
    // single top-level quantity is declared.
    ingredient salt measured by volume {
        name "kosher salt";
        divided true;
    }
    // Butter is also divided: 3 tbsp for the roux, 3 tbsp melted for the panko.
    ingredient butter measured by volume {
        name "unsalted butter";
        divided true;
        allergen milk;
    }
    ingredient flour measured by volume {
        name "all-purpose flour";
        quantity 3 tbsp;
    }
    ingredient mustard measured by volume {
        name "ground mustard";
        quantity 1 tbsp;
    }
    ingredient paprika measured by volume {
        name "smoked or regular paprika";
        quantity 1 tsp;
    }
    ingredient half_and_half measured by volume {
        name "half and half";
        quantity 3 cup;
        allergen milk;
    }
    ingredient cheddar measured by mass {
        name "sharp cheddar";
        quantity 4 oz;
        allergen milk;
        state grated;
    }
    // Monterey jack is divided: 5 oz into the sauce, 3 oz over the top.
    ingredient jack measured by mass {
        name "Monterey jack";
        quantity 8 oz;
        allergen milk;
        state grated;
        divided true;
    }
    ingredient pepper measured by volume {
        name "freshly ground black pepper";
        quantity 0.5 tsp;
    }
    ingredient panko measured by volume {
        name "panko breadcrumbs";
        quantity 1 cup;
    }

    // Separate vessels: boiling the pasta and building the sauce can overlap
    // because they never share a pot.
    container casserole { name "4-quart casserole or souffle dish"; }
    container pot { name "4-quart pot"; }
    container saucepan { name "3-quart saucepan"; }
    equipment colander { name "colander"; }

    material cooked_macaroni measured by mass { }
    material roux measured by mass { }
    material spiced_roux measured by mass { }
    material thickened_base measured by mass { }
    material cheese_sauce measured by mass { }
    material sauce_mix measured by mass { }
    material assembled_dish measured by mass { }
    material topped_dish measured by mass { }

    // The oven preheat is ~15 unattended minutes that overlaps the prep, and
    // greasing the dish is a real (if quick) step the source calls for.
    process setup {
        operation preheat does heat {
            target casserole;
            temperature 350 fahrenheit;
            duration estimated 15 min;
            labor automated;
        }
        operation grease does coat {
            container casserole;
            note "grease the dish (butter or nonstick spray) and set aside";
            duration 1 min;
            labor active;
            produces greased_dish;
        }
    }

    process pasta {
        operation boil does heat {
            input macaroni;
            input salt 1 tbsp;
            container pot;
            heat high;
            duration 9 minutes to 10 minutes;
            until visual "al dente";
            note "barely cover with cold water; bring just to a boil, stirring occasionally";
            labor monitor;
            produces boiled_macaroni;
        }
        operation drain does strain {
            input [boiled_macaroni];
            after boil;
            equipment colander;
            note "do not rinse";
            duration 1 min;
            labor active;
            produces cooked_macaroni;
        }
    }

    // The roux is split into stages because heat, labor, and the doneness cue
    // change at each one (pale blond -> aromatic -> slightly thickened).
    process sauce {
        operation melt_butter does heat {
            input butter 3 tbsp;
            container saucepan;
            heat medium;
            duration 1 min to 2 min;
            labor active;
            produces melted_butter;
        }
        operation make_roux does heat {
            input [melted_butter, flour];
            after melt_butter;
            container saucepan;
            heat medium;
            duration 3 min;
            until visual "pale blond";
            labor active;
            produces roux;
        }
        operation bloom_spices does heat {
            input [roux, mustard, paprika];
            after make_roux;
            container saucepan;
            heat medium;
            duration 1 min;
            until visual "aromatic";
            labor active;
            produces spiced_roux;
        }
        operation build_base does heat {
            input [spiced_roux, half_and_half];
            after bloom_spices;
            container saucepan;
            heat medium_high;
            duration 7 min to 8 min;
            until texture "slightly thickened, coats the back of a spoon";
            labor monitor;
            produces thickened_base;
        }
        operation stir_in_cheese does mix {
            input [thickened_base, cheddar, pepper];
            input jack 5 oz;
            input salt 0.5 tsp;
            after build_base;
            note "off the heat, stir until the cheese melts";
            duration 2 min;
            labor active;
            produces cheese_sauce;
        }
    }

    process assembly {
        operation combine does mix {
            input [cheese_sauce, cooked_macaroni];
            after [stir_in_cheese, drain];
            container saucepan;
            duration 2 min;
            labor active;
            produces sauce_mix;
        }
        operation transfer does move {
            input [sauce_mix];
            after [combine, grease];
            container casserole;
            duration 1 min;
            labor active;
            produces assembled_dish;
        }
        operation top does mix {
            input [assembled_dish, panko];
            input jack 3 oz;
            input butter 3 tbsp;
            after transfer;
            container casserole;
            note "melt the remaining butter and toss it with the panko, then distribute evenly over the top";
            duration 2 min;
            labor active;
            produces topped_dish;
        }
        operation bake does heat {
            input [topped_dish];
            after [top, preheat];
            container casserole;
            temperature 350 fahrenheit;
            duration 15 minutes to 20 minutes;
            until visual "golden on top";
            labor passive;
            produces baked_dish;
        }
        operation cool does rest {
            input [baked_dish];
            after bake;
            note "cool 5 minutes before serving";
            duration 5 min;
            labor passive;
        }
    }

    yield servings measured by count {
        amount 6 count;
    }
}
`,
  },
  {
    symbol: "easy_crepes",
    title: "Easy Crepes",
    sourceText: `culinator 0.3;

recipe easy_crepes {
    title "Easy Crepes";
    section "Breakfast";

    source "Alton Brown";
    publisher "Food Network / altonbrown.com";
    source_url "https://altonbrown.com/recipes/easy-crepes/";
    attribution "Recipe by Alton Brown. Included as sample data; see the source link for the original.";

    ingredient eggs measured by count {
        name "eggs";
        quantity 2 count;
        size "large";
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
    // Spoon and cup measures are volume, not mass.
    ingredient flour measured by volume {
        name "all-purpose flour";
        quantity 1 cup;
    }
    ingredient butter measured by volume {
        name "unsalted butter";
        quantity 3 tbsp;
        allergen milk;
        state melted;
        note "melted and cooled";
    }
    ingredient salt measured by volume {
        name "kosher salt";
        quantity 0.25 tsp;
    }
    // "Plus additional for the pan": a distinct, open-ended use, so it is its
    // own optional ingredient rather than folded into the batter butter.
    ingredient pan_butter measured by volume {
        name "butter for the pan";
        optional true;
        note "for greasing the pan between crepes";
    }
    // Two mutually-exclusive finishes. Ingredients sharing a \`variant\` label
    // form one alternative set; neither is part of the base batter.
    ingredient herbs measured by volume {
        name "fresh herbs";
        quantity 0.25 cup;
        variant "savory";
        note "finely chopped";
    }
    ingredient sugar measured by volume {
        name "sugar";
        quantity 2 tbsp;
        variant "sweet";
    }
    ingredient vanilla measured by volume {
        name "vanilla extract";
        quantity 1 tsp;
        variant "sweet";
    }

    equipment blender { name "blender"; }
    equipment pan { name "10-inch nonstick pan"; }

    material crepe_batter measured by volume { }
    material rested_batter measured by volume { }
    material cooked_crepes measured by count { }

    process batter {
        operation blend does mix {
            input [eggs, milk, water, flour, butter, salt];
            tool blender;
            note "pulse for 10 seconds until smooth";
            duration 1 min;
            labor active;
            produces crepe_batter;
        }
        operation chill does rest {
            input [crepe_batter];
            after blend;
            note "refrigerate to let the bubbles subside; batter keeps up to 48 hours";
            duration 1 hour;
            labor passive;
            produces rested_batter;
        }
    }
    process cooking {
        // One \`cook\` step stands in for all ~11 crepes: \`repeat\` tells the
        // scheduler the duration is per crepe, so the batch cost is counted.
        operation cook does heat {
            input [rested_batter];
            input pan_butter;
            after chill;
            tool pan;
            heat medium;
            duration 45 seconds to 90 seconds;
            repeat 11;
            until visual "edges begin to lift";
            note "pour a scant 1/4 cup and swirl to coat; flip and cook about 30 seconds more; butter the pan between crepes";
            labor active;
            produces cooked_crepes;
        }
    }
    process storing {
        // Make-ahead holding: an open-ended ceiling via \`up to\`.
        operation store does rest {
            input [cooked_crepes];
            after cook;
            note "cool flat, then stack separated by parchment; refrigerate several days or freeze up to 1 month";
            duration up to 30 day;
            labor passive;
            optional true;
        }
    }

    yield crepes measured by count {
        amount 10 count to 12 count;
    }
}
`,
  },
  {
    symbol: "fully_loaded_guacamole",
    title: "Fully Loaded Guacamole",
    sourceText: `culinator 0.3;

recipe fully_loaded_guacamole {
    title "Fully Loaded Guacamole";
    section "Starters";

    source "Alton Brown";
    publisher "Food Network / altonbrown.com";
    source_url "https://altonbrown.com/recipes/fully-loaded-guacamole/";
    attribution "Recipe by Alton Brown. Included as sample data; see the source link for the original.";

    // Size grades and handling notes that a bare \`count\` would drop are carried
    // on the ingredient (\`size\`, \`note\`) instead of being smuggled into \`name\`.
    ingredient avocados measured by count {
        name "Hass avocados";
        quantity 3 count;
        state ripe;
    }
    // Spoon measures classify as \`volume\` (see Dimension::from_unit); declaring
    // them \`measured by mass\` would contradict the unit.
    ingredient lime_juice measured by volume {
        name "freshly squeezed lime juice";
        quantity 1 tbsp;
    }
    ingredient salt measured by volume {
        name "kosher salt";
        quantity 0.5 tsp;
    }
    ingredient cumin measured by volume {
        name "ground cumin";
        quantity 0.5 tsp;
    }
    // "1/4 tsp, plus more to taste": the base amount is fixed, the rest is the
    // cook's call — \`to_taste\` keeps that open-ended intent.
    ingredient cayenne measured by volume {
        name "ground cayenne pepper";
        quantity 0.25 tsp;
        to_taste true;
    }
    ingredient onion measured by count {
        name "onion";
        quantity 0.5 count;
        size "medium";
    }
    ingredient tomatoes measured by count {
        name "Roma tomatoes";
        quantity 2 count;
        size "small";
        note "seeded before dicing";
    }
    ingredient cilantro measured by volume {
        name "fresh cilantro";
        quantity 1 tbsp;
        note "measured after chopping";
    }
    ingredient jalapeno measured by count {
        name "jalapeno";
        quantity 0.5 count;
    }
    ingredient garlic measured by count {
        name "garlic clove";
        quantity 1 clove;
    }

    // Equipment and containers the source names, so the scheduler can see that
    // the mash and the knife work contend for the cook's hands but not vessels.
    equipment masher { name "potato masher"; }
    container mixing_bowl { name "large mixing bowl"; }
    container plastic_wrap { name "plastic wrap"; }

    // \`avocado_pulp\` and \`mashed_avocado\` are declared so they can carry a
    // state/note; the diced/minced prep products stay implicit intermediates.
    material avocado_pulp measured by count { }
    material mashed_avocado measured by mass {
        state chunky;
    }

    // Halving, pitting, and peeling is real knife work, not a free adjective on
    // the ingredient line — it becomes an operation with a duration.
    process prep {
        prep pit avocados into avocado_pulp {
            note "halve, pit, and peel";
            duration 3 min;
        }
        prep dice onion into diced_onion { duration 2 min; }
        prep dice tomatoes into diced_tomatoes { duration 3 min; }
        prep chop cilantro into chopped_cilantro { duration 1 min; }
        prep mince jalapeno into minced_jalapeno { duration 2 min; }
        prep mince garlic into minced_garlic { duration 1 min; }
    }

    // Mashing overlaps with the knife work; only \`fold\` waits on everything.
    process mixing {
        operation mash does mix {
            input [avocado_pulp, lime_juice, salt, cumin, cayenne];
            after pit_avocados;
            tool masher;
            container mixing_bowl;
            note "toss the avocado with lime juice first, then mash, leaving some larger chunks for texture";
            duration 3 min;
            labor active;
            produces mashed_avocado;
        }
        operation fold does mix {
            input [mashed_avocado, diced_onion, diced_tomatoes, chopped_cilantro, minced_jalapeno, minced_garlic];
            after [mash, dice_onion, dice_tomatoes, chop_cilantro, mince_jalapeno, mince_garlic];
            container mixing_bowl;
            duration 2 min;
            labor active;
            produces guacamole_mix;
        }
    }

    process resting {
        operation rest does rest {
            input [guacamole_mix];
            after fold;
            container plastic_wrap;
            note "press plastic wrap directly onto the surface and rest at room temperature";
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
