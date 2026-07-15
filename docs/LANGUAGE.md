# Culinograph language 0.3

Culinograph uses readable declaration keywords that compile to explicit semantic types.

```cg
ingredient flour measured by mass {
    quantity 500 g;
}
```

The semantic model exposed to the validator, database, and LSP is `Ingredient<Mass>`. The generic spelling remains accepted for advanced and custom types, but is no longer required for common declarations.

## Common declarations

```cg
ingredient water measured by mass { quantity 340 g; }
material dough measured by mass { }
container bowl measured by volume { capacity 4 l; }
equipment oven as ConvectionOven measured by temperature { }
process mixing { }
operation combine does mix { }
yield loaf measured by mass { mass 850 g; }
serving slice measured by mass { mass 50 g; }
```

`as` expresses a specialization; `measured by` expresses the quantity dimension. Thus `ingredient starter as Levain measured by mass` resolves to a Levain ingredient measured in mass.

## Formulas

```cg
formula dough relative to flour {
    ingredient flour measured by mass {
        percentage 100%;
        reference true;
        flour true;
    }
    ingredient water measured by mass {
        percentage 68%;
        water_fraction 1;
    }
}
```

Use `formula blend of total` for percentages of total product mass. Fixed mass lines use `basis absolute`.

The older generic syntax remains backwards compatible:

```cg
resource flour as Ingredient<Mass> { quantity 500 g; }
```

## Recipe books

A recipe book is the organizational root for a collection of recipes. It is typed as
`RecipeBook`, has its own metadata, and contains complete recipe declarations:

```culinograph
culinograph 0.3;

book family_favorites {
    title "Family Favorites";
    description "Recipes we make often.";

    recipe sunday_bread {
        title "Sunday Bread";
    }

    recipe tomato_soup {
        title "Tomato Soup";
    }
}
```

Standalone recipe documents remain valid. When imported into SQLite, a standalone recipe
may be assigned to a book without modifying its recipe source. Book membership is therefore
organizational metadata, while a book document can also be used as a portable multi-recipe
exchange format.
