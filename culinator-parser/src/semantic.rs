use culinator_core::*;
use std::collections::{BTreeMap, HashSet};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String),
    String(String),
    Number(f64),
    Percent(f64),
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Lt,
    Gt,
    Comma,
    Semi,
    Eq,
    Dot,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("lex error at byte {0}: {1}")]
    Lex(usize, String),
    #[error("parse error near token {0}: {1}")]
    Syntax(usize, String),
    #[error("syntax error: {0}")]
    Lossless(String),
}

impl From<crate::syntax::SyntaxError> for ParseError {
    fn from(error: crate::syntax::SyntaxError) -> Self {
        Self::Lossless(error.to_string())
    }
}

pub(crate) fn parse_semantic_document(source: &str) -> Result<Document, ParseError> {
    let tokens = lex(source)?;
    Parser { tokens, at: 0 }.document()
}

pub(crate) fn parse_semantic_recipe(source: &str) -> Result<Recipe, ParseError> {
    match parse_semantic_document(source)? {
        Document::Recipe { recipe } => Ok(recipe),
        Document::RecipeBook { .. } => Err(ParseError::Syntax(
            0,
            "expected a recipe document, found a recipe book".into(),
        )),
    }
}

pub(crate) fn parse_semantic_recipe_book(source: &str) -> Result<RecipeBook, ParseError> {
    match parse_semantic_document(source)? {
        Document::RecipeBook { book } => Ok(book),
        Document::Recipe { .. } => Err(ParseError::Syntax(
            0,
            "expected a recipe book document, found a recipe".into(),
        )),
    }
}

fn lex(source: &str) -> Result<Vec<Token>, ParseError> {
    let b = source.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    while i < b.len() {
        match b[i] as char {
            c if c.is_whitespace() => i += 1,
            '/' if i + 1 < b.len() && b[i + 1] == b'/' => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            '/' if i + 1 < b.len() && b[i + 1] == b'*' => {
                let start = i;
                i += 2;
                while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                if i + 1 >= b.len() {
                    return Err(ParseError::Lex(start, "unterminated block comment".into()));
                }
                i += 2;
            }
            '{' => {
                out.push(Token::LBrace);
                i += 1
            }
            '}' => {
                out.push(Token::RBrace);
                i += 1
            }
            '[' => {
                out.push(Token::LBracket);
                i += 1
            }
            ']' => {
                out.push(Token::RBracket);
                i += 1
            }
            '<' => {
                out.push(Token::Lt);
                i += 1
            }
            '>' => {
                out.push(Token::Gt);
                i += 1
            }
            ',' => {
                out.push(Token::Comma);
                i += 1
            }
            ';' => {
                out.push(Token::Semi);
                i += 1
            }
            '=' => {
                out.push(Token::Eq);
                i += 1
            }
            '.' => {
                out.push(Token::Dot);
                i += 1
            }
            '"' => {
                i += 1;
                let start = i;
                while i < b.len() && b[i] != b'"' {
                    if b[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                if i >= b.len() {
                    return Err(ParseError::Lex(start, "unterminated string".into()));
                }
                out.push(Token::String(source[start..i].to_string()));
                i += 1;
            }
            c if c.is_ascii_digit() || c == '-' => {
                let start = i;
                i += 1;
                while i < b.len() && ((b[i] as char).is_ascii_digit() || b[i] == b'.') {
                    i += 1;
                }
                let n: f64 = source[start..i]
                    .parse()
                    .map_err(|_| ParseError::Lex(start, "invalid number".into()))?;
                if i < b.len() && b[i] == b'%' {
                    i += 1;
                    out.push(Token::Percent(n));
                } else {
                    out.push(Token::Number(n));
                }
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                i += 1;
                while i < b.len() && ((b[i] as char).is_ascii_alphanumeric() || b[i] == b'_') {
                    i += 1;
                }
                out.push(Token::Ident(source[start..i].to_string()));
            }
            c => return Err(ParseError::Lex(i, format!("unexpected character `{c}`"))),
        }
    }
    Ok(out)
}

struct Parser {
    tokens: Vec<Token>,
    at: usize,
}
impl Parser {
    fn document(mut self) -> Result<Document, ParseError> {
        self.word("culinator")?;
        let version = self.scalar_text()?;
        self.take(Token::Semi)?;
        match self.peek_ident()? {
            "recipe" => Ok(Document::Recipe {
                recipe: self.recipe_declaration(&version, None)?,
            }),
            "book" | "recipe_book" => Ok(Document::RecipeBook {
                book: self.book_declaration(&version)?,
            }),
            other => Err(self.err(&format!("expected `recipe` or `book`, got `{other}`"))),
        }
    }

    fn book_declaration(&mut self, version: &str) -> Result<RecipeBook, ParseError> {
        self.at += 1; // book or recipe_book
        let symbol = self.ident()?;
        let declared_type = if self.peek_ident().ok() == Some("as") {
            self.at += 1;
            self.type_ref()?
        } else {
            TypeRef::named("RecipeBook")
        };
        self.take(Token::LBrace)?;
        let mut book = RecipeBook {
            id: Uuid::new_v4(),
            symbol: symbol.clone(),
            declared_type,
            title: symbol.replace('_', " "),
            description: None,
            protocol_version: version.to_owned(),
            recipes: Vec::new(),
            properties: BTreeMap::new(),
        };
        while !self.peek_is(&Token::RBrace) {
            match self.peek_ident()? {
                "title" => {
                    self.at += 1;
                    book.title = self.string()?;
                    self.take(Token::Semi)?;
                }
                "description" => {
                    self.at += 1;
                    book.description = Some(self.string()?);
                    self.take(Token::Semi)?;
                }
                "recipe" => {
                    let mut recipe = self.recipe_declaration(version, Some(book.id))?;
                    recipe.book_id = Some(book.id);
                    book.recipes.push(recipe);
                }
                _ => {
                    let (key, value) = self.property()?;
                    book.properties.insert(key, value);
                }
            }
        }
        self.take(Token::RBrace)?;
        Ok(book)
    }

    fn recipe_declaration(
        &mut self,
        version: &str,
        book_id: Option<Uuid>,
    ) -> Result<Recipe, ParseError> {
        self.word("recipe")?;
        let symbol = self.ident()?;
        let declared_type = if self.peek_ident().ok() == Some("as") {
            self.at += 1;
            self.type_ref()?
        } else {
            TypeRef::named("Recipe")
        };
        self.take(Token::LBrace)?;
        let mut recipe = Recipe {
            id: Uuid::new_v4(),
            book_id,
            symbol: symbol.clone(),
            declared_type,
            title: symbol.replace('_', " "),
            protocol_version: version.to_owned(),
            types: vec![],
            resources: vec![],
            processes: vec![],
            operations: vec![],
            servings: vec![],
            formulas: vec![],
            yields: vec![],
            properties: BTreeMap::new(),
        };
        while !self.peek_is(&Token::RBrace) {
            let keyword = self.peek_ident()?.to_string();
            match keyword.as_str() {
                "title" => {
                    self.at += 1;
                    recipe.title = self.string()?;
                    self.take(Token::Semi)?;
                }
                "type" => recipe.types.push(self.type_decl()?),
                "resource" | "ingredient" | "material" | "container" | "equipment"
                | "environment" | "labor" => recipe.resources.push(self.resource()?),
                "process" => self.process(&mut recipe, None)?,
                "operation" => {
                    let op = self.operation("root".into())?;
                    recipe.operations.push(op);
                }
                "prep" => {
                    let op = self.prep("root".into())?;
                    recipe.operations.push(op);
                }
                "serving" => recipe.servings.push(self.serving()?),
                "yield" => recipe.yields.push(self.yield_def()?),
                "formula" => recipe.formulas.push(self.formula()?),
                _ => {
                    let (k, v) = self.property()?;
                    recipe.properties.insert(k, v);
                }
            }
        }
        self.take(Token::RBrace)?;
        register_intermediates(&mut recipe);
        Ok(recipe)
    }
    fn type_decl(&mut self) -> Result<TypeDeclaration, ParseError> {
        self.word("type")?;
        let name = self.ident()?;
        self.word("as")?;
        let base = self.type_ref()?;
        let props = self.block_properties()?;
        Ok(TypeDeclaration {
            id: Uuid::new_v4(),
            name,
            base,
            states: BTreeMap::new(),
            properties: props,
            span: None,
        })
    }
    fn resource(&mut self) -> Result<Resource, ParseError> {
        let declaration = self.ident()?;
        let symbol = self.ident()?;
        let mut kind = match declaration.as_str() {
            "ingredient" => ResourceKind::Ingredient,
            "container" => ResourceKind::Container,
            "equipment" => ResourceKind::Equipment,
            "environment" => ResourceKind::Environment,
            "labor" => ResourceKind::Labor,
            _ => ResourceKind::Material,
        };
        let mut ty = match kind {
            ResourceKind::Ingredient => TypeRef::named("Ingredient"),
            ResourceKind::Container => TypeRef::named("Container"),
            ResourceKind::Equipment => TypeRef::named("Equipment"),
            ResourceKind::Environment => TypeRef::named("Environment"),
            ResourceKind::Labor => TypeRef::named("Labor"),
            ResourceKind::Material => TypeRef::named("Material"),
            ResourceKind::Intermediate => TypeRef::named("Intermediate"),
        };
        if self.peek_ident().ok() == Some("as") {
            self.at += 1;
            ty = self.type_ref()?;
        }
        if self.peek_ident().ok() == Some("measured") {
            self.at += 1;
            self.word("by")?;
            let dimension = self.ident()?;
            ty.arguments.push(TypeRef::named(title_case(&dimension)));
        }
        if declaration == "resource" {
            kind = match ty.name.as_str() {
                "Ingredient" => ResourceKind::Ingredient,
                "Container" => ResourceKind::Container,
                "Equipment" | "Oven" | "Burner" => ResourceKind::Equipment,
                "Environment" => ResourceKind::Environment,
                "Labor" => ResourceKind::Labor,
                _ => ResourceKind::Material,
            };
        }
        let mut props = self.block_properties()?;
        let optional = matches!(props.remove("optional"), Some(Value::Boolean(true)));
        let divided = matches!(props.remove("divided"), Some(Value::Boolean(true)));
        let substitutes = match props.remove("substitutes") {
            Some(Value::List(items)) => items,
            Some(other) => vec![other],
            None => vec![],
        };
        Ok(Resource {
            id: Uuid::new_v4(),
            symbol,
            declared_type: ty,
            kind,
            optional,
            divided,
            substitutes,
            properties: props,
            span: None,
        })
    }
    fn process(&mut self, recipe: &mut Recipe, parent: Option<String>) -> Result<(), ParseError> {
        self.word("process")?;
        let symbol = self.ident()?;
        let ty = if self.peek_ident().ok() == Some("as") {
            self.at += 1;
            self.type_ref()?
        } else {
            TypeRef::named("Process")
        };
        self.take(Token::LBrace)?;
        let mut p = Process {
            id: Uuid::new_v4(),
            symbol: symbol.clone(),
            declared_type: ty,
            parent,
            operations: vec![],
            properties: BTreeMap::new(),
        };
        while !self.peek_is(&Token::RBrace) {
            match self.peek_ident()? {
                "operation" => {
                    let op = self.operation(symbol.clone())?;
                    p.operations.push(op.symbol.clone());
                    recipe.operations.push(op)
                }
                "prep" => {
                    let op = self.prep(symbol.clone())?;
                    p.operations.push(op.symbol.clone());
                    recipe.operations.push(op)
                }
                "process" => self.process(recipe, Some(symbol.clone()))?,
                _ => {
                    let (k, v) = self.property()?;
                    p.properties.insert(k, v);
                }
            }
        }
        self.take(Token::RBrace)?;
        recipe.processes.push(p);
        Ok(())
    }
    fn operation(&mut self, process: String) -> Result<Operation, ParseError> {
        self.word("operation")?;
        let symbol = self.ident()?;
        let ty = if self.peek_ident().ok() == Some("as") {
            self.at += 1;
            self.type_ref()?
        } else if self.peek_ident().ok() == Some("does") {
            self.at += 1;
            TypeRef::named(title_case(&self.ident()?))
        } else {
            TypeRef::named("Operation")
        };
        let mut op = self.blank_operation(symbol, ty, process);
        self.operation_body(&mut op)?;
        Ok(op)
    }
    /// Desugar `prep <verb> <ingredient> [into <output>] (; | { ... })` into a
    /// regular operation. The operation is named `<verb>_<ingredient>` (so
    /// downstream `after` references read naturally and match hand-written
    /// operations), takes the ingredient as input, and produces `<output>`
    /// (defaulting to `<ingredient>_<verb>`). The produced material is picked up
    /// by [`register_intermediates`], so no separate `material` declaration is
    /// required. An optional block accepts the same properties as `operation`
    /// (duration, labor, after, additional inputs, ...).
    fn prep(&mut self, process: String) -> Result<Operation, ParseError> {
        self.word("prep")?;
        let verb = self.ident()?;
        let ingredient = self.ident()?;
        let output = if self.peek_ident().ok() == Some("into") {
            self.at += 1;
            self.ident()?
        } else {
            format!("{ingredient}_{verb}")
        };
        let mut op = self.blank_operation(
            format!("{verb}_{ingredient}"),
            TypeRef::named(title_case(&verb)),
            process,
        );
        // Prep is hands-on knife/mix work unless the block says otherwise.
        op.labor = Some(LaborMode::Active);
        op.bindings.push(ResourceBinding {
            resource: ingredient,
            role: BindingRole::Input,
            quantity: None,
            exclusive: false,
        });
        op.bindings.push(ResourceBinding {
            resource: output,
            role: BindingRole::Output,
            quantity: None,
            exclusive: false,
        });
        if self.peek_is(&Token::LBrace) {
            self.operation_body(&mut op)?;
        } else {
            self.take(Token::Semi)?;
        }
        Ok(op)
    }
    fn blank_operation(&self, symbol: String, ty: TypeRef, process: String) -> Operation {
        Operation {
            id: Uuid::new_v4(),
            symbol,
            declared_type: ty,
            process,
            labor: None,
            duration_min_seconds: None,
            duration_max_seconds: None,
            duration_estimated: false,
            target_temperature: None,
            heat_level: None,
            doneness: vec![],
            optional: false,
            dependencies: vec![],
            bindings: vec![],
            requirements: vec![],
            effects: vec![],
            properties: BTreeMap::new(),
            span: None,
        }
    }
    fn operation_body(&mut self, op: &mut Operation) -> Result<(), ParseError> {
        self.take(Token::LBrace)?;
        while !self.peek_is(&Token::RBrace) {
            let key = self.peek_ident()?.to_string();
            match key.as_str() {
                "labor" => {
                    self.at += 1;
                    op.labor = Some(match self.ident()?.as_str() {
                        "active" => LaborMode::Active,
                        "monitor" => LaborMode::Monitor,
                        "automated" => LaborMode::Automated,
                        _ => LaborMode::Passive,
                    });
                    self.take(Token::Semi)?
                }
                "after" => {
                    self.at += 1;
                    if self.peek_is(&Token::LBracket) {
                        // List form is always a plain finish-start fan-in.
                        for d in self.symbol_list()? {
                            op.dependencies.push(Dependency {
                                predecessor: d,
                                kind: DependencyKind::FinishStart,
                                minimum_lag_seconds: None,
                                maximum_lag_seconds: None,
                                optional: false,
                            })
                        }
                    } else {
                        // Single form may carry modifiers: a dependency kind, a
                        // `lag <duration>`, and/or `optional`.
                        let predecessor = self.path()?;
                        let mut kind = DependencyKind::FinishStart;
                        let mut minimum_lag_seconds = None;
                        let mut optional = false;
                        loop {
                            match self.peek_ident().ok() {
                                Some("start_start") => {
                                    self.at += 1;
                                    kind = DependencyKind::StartStart;
                                }
                                Some("finish_finish") => {
                                    self.at += 1;
                                    kind = DependencyKind::FinishFinish;
                                }
                                Some("start_finish") => {
                                    self.at += 1;
                                    kind = DependencyKind::StartFinish;
                                }
                                Some("lag") => {
                                    self.at += 1;
                                    minimum_lag_seconds = Some(self.read_duration_seconds()?);
                                }
                                Some("optional") => {
                                    self.at += 1;
                                    optional = true;
                                }
                                _ => break,
                            }
                        }
                        op.dependencies.push(Dependency {
                            predecessor,
                            kind,
                            minimum_lag_seconds,
                            maximum_lag_seconds: None,
                            optional,
                        })
                    }
                    self.take(Token::Semi)?
                }
                "input" | "output" | "produces" | "target" | "tool" | "container" | "equipment" => {
                    self.at += 1;
                    let role = match key.as_str() {
                        "input" => BindingRole::Input,
                        "output" | "produces" => BindingRole::Output,
                        "target" => BindingRole::Target,
                        "tool" => BindingRole::Tool,
                        "container" => BindingRole::Container,
                        _ => BindingRole::Equipment,
                    };
                    if self.peek_is(&Token::LBracket) {
                        for r in self.symbol_list()? {
                            op.bindings.push(ResourceBinding {
                                resource: r,
                                role,
                                quantity: None,
                                exclusive: false,
                            })
                        }
                    } else {
                        // Single form may carry a per-step amount:
                        // `input butter 6 tbsp;` (divided ingredients).
                        let resource = self.path()?;
                        let quantity = if matches!(self.tokens.get(self.at), Some(Token::Number(_)))
                        {
                            Some(self.read_quantity()?)
                        } else {
                            None
                        };
                        op.bindings.push(ResourceBinding {
                            resource,
                            role,
                            quantity,
                            exclusive: false,
                        })
                    }
                    self.take(Token::Semi)?
                }
                "temperature" => {
                    self.at += 1;
                    op.target_temperature = Some(self.read_quantity()?);
                    self.take(Token::Semi)?
                }
                "heat" => {
                    self.at += 1;
                    op.heat_level = Some(match self.ident()?.as_str() {
                        "low" => HeatLevel::Low,
                        "medium_low" => HeatLevel::MediumLow,
                        "medium" => HeatLevel::Medium,
                        "medium_high" => HeatLevel::MediumHigh,
                        "high" => HeatLevel::High,
                        other => {
                            return Err(self.err(&format!("unknown heat level `{other}`")))
                        }
                    });
                    self.take(Token::Semi)?
                }
                "until" => {
                    self.at += 1;
                    let cue = match self.ident()?.as_str() {
                        "internal_temp" => DonenessKind::InternalTemp,
                        "visual" => DonenessKind::Visual,
                        "tester" => DonenessKind::Tester,
                        "texture" => DonenessKind::Texture,
                        "rise" => DonenessKind::Rise,
                        other => {
                            return Err(self.err(&format!("unknown doneness cue `{other}`")))
                        }
                    };
                    // value_until_semi consumes the trailing `;`.
                    let value = self.value_until_semi()?;
                    op.doneness.push(DonenessCue { kind: cue, value });
                }
                "optional" => {
                    self.at += 1;
                    op.optional = if self.peek_is(&Token::Semi) {
                        true
                    } else {
                        self.ident()? != "false"
                    };
                    self.take(Token::Semi)?
                }
                "requires" => {
                    self.at += 1;
                    let text = self.until_semi();
                    op.requirements.push(Predicate { source: text });
                }
                "duration" => {
                    self.at += 1;
                    let estimated = self.peek_ident().ok() == Some("estimated");
                    if estimated {
                        self.at += 1;
                    }
                    op.duration_estimated = estimated;
                    if self.peek_ident().ok() == Some("up") {
                        // `duration up to N unit;` -> open-ended lower bound
                        // (holding / make-ahead ceiling, e.g. "up to overnight").
                        self.at += 1;
                        self.word("to")?;
                        op.duration_min_seconds = Some(0);
                        op.duration_max_seconds = Some(self.read_duration_seconds()?);
                    } else {
                        let secs = self.read_duration_seconds()?;
                        op.duration_min_seconds = Some(secs);
                        if self.peek_ident().ok() == Some("to") {
                            self.at += 1;
                            op.duration_max_seconds = Some(self.read_duration_seconds()?);
                        } else {
                            op.duration_max_seconds = Some(secs);
                        }
                    }
                    self.take(Token::Semi)?
                }
                _ => {
                    let (k, v) = self.property()?;
                    op.properties.insert(k, v);
                }
            }
        }
        self.take(Token::RBrace)?;
        Ok(())
    }
    fn serving(&mut self) -> Result<Serving, ParseError> {
        self.word("serving")?;
        let symbol = self.ident()?;
        let ty = self.read_measured_type("Serving")?;
        let props = self.block_properties()?;
        let amount = props.get("amount").cloned().unwrap_or(Value::Number(1.0));
        let mass_grams = props.get("mass").and_then(quantity_grams);
        Ok(Serving {
            symbol,
            declared_type: ty,
            amount,
            mass_grams,
            is_default: matches!(props.get("default"), Some(Value::Boolean(true))),
        })
    }
    fn yield_def(&mut self) -> Result<YieldDefinition, ParseError> {
        self.word("yield")?;
        let symbol = self.ident()?;
        let ty = self.read_measured_type("Yield")?;
        let props = self.block_properties()?;
        let amount = props.get("amount").cloned().unwrap_or(Value::Number(1.0));
        let mass_grams = props.get("mass").and_then(quantity_grams);
        Ok(YieldDefinition {
            symbol,
            declared_type: ty,
            amount,
            mass_grams,
            properties: props,
        })
    }
    fn formula(&mut self) -> Result<Formula, ParseError> {
        self.word("formula")?;
        let symbol = self.ident()?;
        let ty = if self.peek_ident().ok() == Some("as") {
            self.at += 1;
            self.type_ref()?
        } else if self.peek_ident().ok() == Some("relative") {
            self.at += 1;
            self.word("to")?;
            let _ = self.ident()?;
            TypeRef::named("ReferenceFormula")
        } else if self.peek_ident().ok() == Some("of") {
            self.at += 1;
            self.word("total")?;
            TypeRef::named("TotalFormula")
        } else {
            TypeRef::named("ReferenceFormula")
        };
        self.take(Token::LBrace)?;
        let basis = if ty.name == "TotalFormula" {
            FormulaBasis::PercentOfTotal
        } else {
            FormulaBasis::ReferencePercent
        };
        let mut ingredients = vec![];
        let mut props = BTreeMap::new();
        while !self.peek_is(&Token::RBrace) {
            if self.peek_ident()? == "ingredient" {
                self.at += 1;
                let s = self.ident()?;
                let mut ity = TypeRef::named("Ingredient");
                if self.peek_ident().ok() == Some("as") {
                    self.at += 1;
                    ity = self.type_ref()?;
                }
                if self.peek_ident().ok() == Some("measured") {
                    self.at += 1;
                    self.word("by")?;
                    ity.arguments
                        .push(TypeRef::named(title_case(&self.ident()?)));
                }
                let ip = self.block_properties()?;
                let pct = ip.get("percentage").and_then(value_number);
                let mass = ip.get("mass").and_then(quantity_grams);
                let mode = match ip.get("basis") {
                    Some(Value::Symbol(x)) if x == "total" => FormulaBasis::PercentOfTotal,
                    Some(Value::Symbol(x)) if x == "absolute" => FormulaBasis::AbsoluteMass,
                    _ => basis,
                };
                ingredients.push(FormulaIngredient {
                    id: Uuid::new_v4(),
                    symbol: s.clone(),
                    name: ip.get("name").and_then(value_text).unwrap_or(s),
                    stage: ip
                        .get("stage")
                        .and_then(value_text)
                        .unwrap_or("final".into()),
                    basis: mode,
                    percentage: pct,
                    mass_grams: mass,
                    is_reference: matches!(ip.get("reference"), Some(Value::Boolean(true))),
                    is_flour: matches!(ip.get("flour"), Some(Value::Boolean(true)))
                        || ity.name.contains("Flour"),
                    water_fraction: ip
                        .get("water_fraction")
                        .and_then(value_number)
                        .unwrap_or(0.0),
                    scalable: !matches!(ip.get("scalable"), Some(Value::Boolean(false))),
                    properties: ip,
                });
            } else {
                let (k, v) = self.property()?;
                props.insert(k, v);
            }
        }
        self.take(Token::RBrace)?;
        Ok(Formula {
            id: Uuid::new_v4(),
            recipe_id: None,
            symbol: symbol.clone(),
            name: props.get("name").and_then(value_text).unwrap_or(symbol),
            basis,
            ingredients,
            properties: props,
        })
    }
    fn block_properties(&mut self) -> Result<BTreeMap<String, Value>, ParseError> {
        self.take(Token::LBrace)?;
        let mut p = BTreeMap::new();
        while !self.peek_is(&Token::RBrace) {
            let (k, v) = self.property()?;
            p.insert(k, v);
        }
        self.take(Token::RBrace)?;
        Ok(p)
    }
    fn property(&mut self) -> Result<(String, Value), ParseError> {
        let key = self.ident()?;
        let v = self.value_until_semi()?;
        Ok((key, v))
    }
    fn value_until_semi(&mut self) -> Result<Value, ParseError> {
        let start = self.at;
        let first = self
            .tokens
            .get(self.at)
            .cloned()
            .ok_or_else(|| self.err("expected value"))?;
        let v = match first {
            Token::String(s) => {
                self.at += 1;
                Value::Text(s)
            }
            Token::Percent(n) => {
                self.at += 1;
                Value::Number(n)
            }
            Token::Number(n) => {
                self.at += 1;
                // A bare `to` after the number begins a range, so don't eat it
                // as a unit (`2 to 3 clove`).
                let first = match self.tokens.get(self.at).cloned() {
                    Some(Token::Ident(unit)) if unit != "to" => {
                        self.at += 1;
                        Value::Quantity(Quantity {
                            value: n,
                            dimension: dimension(&unit),
                            unit,
                        })
                    }
                    _ => Value::Number(n),
                };
                if self.peek_ident().ok() == Some("to") {
                    self.at += 1;
                    let max = self.read_range_bound()?;
                    Value::Range {
                        min: Box::new(first),
                        max: Box::new(max),
                    }
                } else {
                    first
                }
            }
            Token::Ident(s) => {
                self.at += 1;
                match s.as_str() {
                    "true" => Value::Boolean(true),
                    "false" => Value::Boolean(false),
                    _ => Value::Symbol(s),
                }
            }
            Token::LBracket => Value::List(self.value_list()?),
            _ => return Err(self.err("unsupported property value")),
        };
        if !self.peek_is(&Token::Semi) {
            while !self.peek_is(&Token::Semi) && self.at < self.tokens.len() {
                self.at += 1;
            }
            let text = self.tokens[start..self.at]
                .iter()
                .map(|t| format!("{t:?}"))
                .collect::<Vec<_>>()
                .join(" ");
            self.take(Token::Semi)?;
            return Ok(Value::Text(text));
        }
        self.take(Token::Semi)?;
        Ok(v)
    }
    fn value_list(&mut self) -> Result<Vec<Value>, ParseError> {
        self.take(Token::LBracket)?;
        let mut v = vec![];
        while !self.peek_is(&Token::RBracket) {
            v.push(match self.tokens.get(self.at).cloned() {
                Some(Token::Ident(s)) => {
                    self.at += 1;
                    Value::Symbol(s)
                }
                Some(Token::String(s)) => {
                    self.at += 1;
                    Value::Text(s)
                }
                Some(Token::Number(n)) => {
                    self.at += 1;
                    Value::Number(n)
                }
                _ => return Err(self.err("invalid list value")),
            });
            if self.peek_is(&Token::Comma) {
                self.at += 1;
            }
        }
        self.take(Token::RBracket)?;
        Ok(v)
    }
    fn symbol_list(&mut self) -> Result<Vec<String>, ParseError> {
        if self.peek_is(&Token::LBracket) {
            self.take(Token::LBracket)?;
            let mut v = vec![];
            while !self.peek_is(&Token::RBracket) {
                v.push(self.path()?);
                if self.peek_is(&Token::Comma) {
                    self.at += 1;
                }
            }
            self.take(Token::RBracket)?;
            Ok(v)
        } else {
            Ok(vec![self.path()?])
        }
    }
    fn path(&mut self) -> Result<String, ParseError> {
        let mut s = self.ident()?;
        while self.peek_is(&Token::Dot) {
            self.at += 1;
            s.push('.');
            s.push_str(&self.ident()?);
        }
        Ok(s)
    }
    fn type_ref(&mut self) -> Result<TypeRef, ParseError> {
        let name = self.ident()?;
        let mut args = vec![];
        if self.peek_is(&Token::Lt) {
            self.at += 1;
            while !self.peek_is(&Token::Gt) {
                args.push(self.type_ref()?);
                if self.peek_is(&Token::Comma) {
                    self.at += 1;
                }
            }
            self.take(Token::Gt)?;
        }
        Ok(TypeRef {
            name,
            arguments: args,
        })
    }
    fn read_measured_type(&mut self, base: &str) -> Result<TypeRef, ParseError> {
        let mut ty = TypeRef::named(base);
        if self.peek_ident().ok() == Some("as") {
            self.at += 1;
            ty = self.type_ref()?;
        }
        if self.peek_ident().ok() == Some("measured") {
            self.at += 1;
            self.word("by")?;
            ty.arguments
                .push(TypeRef::named(title_case(&self.ident()?)));
        }
        Ok(ty)
    }
    fn scalar_text(&mut self) -> Result<String, ParseError> {
        match self.tokens.get(self.at).cloned() {
            Some(Token::Number(n)) => {
                self.at += 1;
                Ok(n.to_string())
            }
            Some(Token::Ident(s)) => {
                self.at += 1;
                Ok(s)
            }
            _ => Err(self.err("expected version")),
        }
    }
    fn string(&mut self) -> Result<String, ParseError> {
        match self.tokens.get(self.at).cloned() {
            Some(Token::String(s)) => {
                self.at += 1;
                Ok(s)
            }
            _ => Err(self.err("expected string")),
        }
    }
    fn number(&mut self) -> Result<f64, ParseError> {
        match self.tokens.get(self.at).cloned() {
            Some(Token::Number(n)) => {
                self.at += 1;
                Ok(n)
            }
            Some(Token::Percent(n)) => {
                self.at += 1;
                Ok(n)
            }
            _ => Err(self.err("expected number")),
        }
    }
    fn ident(&mut self) -> Result<String, ParseError> {
        match self.tokens.get(self.at).cloned() {
            Some(Token::Ident(s)) => {
                self.at += 1;
                Ok(s)
            }
            _ => Err(self.err("expected identifier")),
        }
    }
    /// Read `<number> <unit>` into a [`Quantity`] (unit required).
    fn read_quantity(&mut self) -> Result<Quantity, ParseError> {
        let value = self.number()?;
        let unit = self.ident()?;
        Ok(Quantity {
            value,
            dimension: dimension(&unit),
            unit,
        })
    }
    /// Read the upper bound of a range: `<number>` with an optional unit.
    fn read_range_bound(&mut self) -> Result<Value, ParseError> {
        let n = self.number()?;
        match self.tokens.get(self.at).cloned() {
            Some(Token::Ident(unit)) => {
                self.at += 1;
                Ok(Value::Quantity(Quantity {
                    value: n,
                    dimension: dimension(&unit),
                    unit,
                }))
            }
            _ => Ok(Value::Number(n)),
        }
    }
    /// Read `<number> <time-unit>` and normalize to seconds.
    fn read_duration_seconds(&mut self) -> Result<u64, ParseError> {
        let n = self.number()?;
        let unit = self.ident()?;
        Ok(duration_seconds(n, &unit))
    }
    fn word(&mut self, w: &str) -> Result<(), ParseError> {
        let got = self.ident()?;
        if got == w {
            Ok(())
        } else {
            Err(self.err(&format!("expected `{w}`, got `{got}`")))
        }
    }
    fn take(&mut self, t: Token) -> Result<(), ParseError> {
        if self.tokens.get(self.at) == Some(&t) {
            self.at += 1;
            Ok(())
        } else {
            Err(self.err(&format!("expected {t:?}")))
        }
    }
    fn peek_is(&self, t: &Token) -> bool {
        self.tokens.get(self.at) == Some(t)
    }
    fn peek_ident(&self) -> Result<&str, ParseError> {
        match self.tokens.get(self.at) {
            Some(Token::Ident(s)) => Ok(s),
            _ => Err(self.err("expected declaration or property")),
        }
    }
    fn until_semi(&mut self) -> String {
        let mut s = String::new();
        while !self.peek_is(&Token::Semi) && self.at < self.tokens.len() {
            s.push_str(&format!("{:?} ", self.tokens[self.at]));
            self.at += 1;
        }
        self.at += 1;
        s
    }
    fn err(&self, m: &str) -> ParseError {
        ParseError::Syntax(self.at, m.into())
    }
}
/// Give any operation output that has no matching resource declaration an
/// implicit [`ResourceKind::Intermediate`] resource. This lets a recipe wire
/// operations together with `produces`/`output` and downstream `input` without
/// forcing the author to also declare a `material` for every partial product.
fn register_intermediates(recipe: &mut Recipe) {
    let declared: HashSet<&str> = recipe.resources.iter().map(|r| r.symbol.as_str()).collect();
    let mut added: HashSet<String> = HashSet::new();
    let mut intermediates = Vec::new();
    for op in &recipe.operations {
        for binding in &op.bindings {
            if binding.role == BindingRole::Output
                && !declared.contains(binding.resource.as_str())
                && added.insert(binding.resource.clone())
            {
                intermediates.push(Resource {
                    id: Uuid::new_v4(),
                    symbol: binding.resource.clone(),
                    declared_type: TypeRef::named("Intermediate"),
                    kind: ResourceKind::Intermediate,
                    optional: false,
                    divided: false,
                    substitutes: vec![],
                    properties: BTreeMap::new(),
                    span: None,
                });
            }
        }
    }
    recipe.resources.extend(intermediates);
}

fn title_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
fn dimension(u: &str) -> Dimension {
    Dimension::from_unit(u)
}
/// Normalize `<number> <time-unit>` to whole seconds. Unknown units fall back to
/// seconds so a bare number never silently scales wrongly.
fn duration_seconds(n: f64, unit: &str) -> u64 {
    (n * time_unit_seconds(unit).unwrap_or(1.0)) as u64
}
fn quantity_grams(v: &Value) -> Option<f64> {
    match v {
        Value::Quantity(q) => q.as_grams(),
        _ => None,
    }
}
fn value_number(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => Some(*n),
        Value::Quantity(q) => Some(q.value),
        _ => None,
    }
}
fn value_text(v: &Value) -> Option<String> {
    match v {
        Value::Text(s) | Value::Symbol(s) => Some(s.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod test;
