//! Stage 2 — parse ONE spec tree (`Alias = quick_sort< … >`) into a [`SpecNode`].

#[derive(Debug, Clone)]
pub struct SpecNode {
    pub name: String,
    pub args: Vec<Arg>,
}

#[derive(Debug, Clone)]
pub enum Arg {
    /// A nested type slot: `partition = LL_partition<…>`.
    Slot { name: String, value: SpecNode },
    /// A const bound by name: `ping_pong = true`.
    NamedConst { name: String, value: String },
    /// A positional const literal: the `32` in `insertion<32>`.
    Const(String),
}

#[derive(Debug, PartialEq, Clone)]
enum Tok {
    Ident(String),
    Num(String),
    Lt,
    Gt,
    Eq,
    Comma,
}

fn tokenize(s: &str) -> Result<Vec<Tok>, String> {
    let mut out = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            c if c.is_whitespace() => {
                chars.next();
            }
            '<' => { chars.next(); out.push(Tok::Lt); }
            '>' => { chars.next(); out.push(Tok::Gt); }
            '=' => { chars.next(); out.push(Tok::Eq); }
            ',' => { chars.next(); out.push(Tok::Comma); }
            c if c.is_alphanumeric() || c == '_' => {
                let mut w = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        w.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if w.chars().all(|c| c.is_ascii_digit()) {
                    out.push(Tok::Num(w));
                } else {
                    out.push(Tok::Ident(w));
                }
            }
            other => return Err(format!("unexpected character `{other}`")),
        }
    }
    Ok(out)
}

/// Is `tok` a const literal (`32`, `true`, `false`)?
fn const_literal(tok: &Tok) -> Option<String> {
    match tok {
        Tok::Num(n) => Some(n.clone()),
        Tok::Ident(id) if id == "true" || id == "false" => Some(id.clone()),
        _ => None,
    }
}

/// Parse `Alias = <tree>` or just `<tree>`. Returns the optional alias and the
/// root node.
pub fn parse_spec(text: &str) -> Result<(Option<String>, SpecNode), String> {
    let toks = tokenize(text)?;
    let mut pos = 0;
    // Optional `Ident =` alias prefix (but not `Ident = <literal>`, which is a
    // bare const-bound node — disambiguated by the value not being a literal).
    let alias = if matches!(toks.first(), Some(Tok::Ident(_)))
        && matches!(toks.get(1), Some(Tok::Eq))
        && toks.get(2).and_then(const_literal).is_none()
    {
        let name = if let Some(Tok::Ident(s)) = toks.first() { s.clone() } else { unreachable!() };
        pos = 2;
        Some(name)
    } else {
        None
    };
    let node = parse_node(&toks, &mut pos)?;
    if pos != toks.len() {
        return Err("trailing tokens after spec".into());
    }
    Ok((alias, node))
}

fn parse_node(toks: &[Tok], pos: &mut usize) -> Result<SpecNode, String> {
    let name = match toks.get(*pos) {
        Some(Tok::Ident(s)) => {
            *pos += 1;
            s.clone()
        }
        _ => return Err("expected a component name".into()),
    };
    let mut args = Vec::new();
    if matches!(toks.get(*pos), Some(Tok::Lt)) {
        *pos += 1; // consume <
        while !matches!(toks.get(*pos), Some(Tok::Gt)) {
            match toks.get(*pos) {
                None => return Err("unterminated `<`".into()),
                // positional const literal: `insertion<32>`
                Some(t) if const_literal(t).is_some() => {
                    args.push(Arg::Const(const_literal(t).unwrap()));
                    *pos += 1;
                }
                Some(Tok::Ident(name)) => {
                    let name = name.clone();
                    *pos += 1;
                    if !matches!(toks.get(*pos), Some(Tok::Eq)) {
                        return Err(format!("slot `{name}` must be written `{name} = <value>`"));
                    }
                    *pos += 1; // consume =
                    // `name = <literal>` is a named const; `name = comp<…>` is a slot.
                    match toks.get(*pos).and_then(const_literal) {
                        Some(value) => {
                            args.push(Arg::NamedConst { name, value });
                            *pos += 1;
                        }
                        None => {
                            let value = parse_node(toks, pos)?;
                            args.push(Arg::Slot { name, value });
                        }
                    }
                }
                _ => return Err("expected a slot binding or literal".into()),
            }
            if matches!(toks.get(*pos), Some(Tok::Comma)) {
                *pos += 1; // commas optional
            }
        }
        *pos += 1; // consume >
    }
    Ok(SpecNode { name, args })
}

// ─────────────────────────────────────────────────────────────────────────────
// The typed constraint language (the front-end for partial families / full
// generation). A query is a sequence of let-bindings ending in a target; holes
// (`.`/`*`/`?`/`?N@seed`) are the only difference between a pinned spec, a
// partial family, and full generation. Lowered to a *set* of ground [`SpecNode`]
// trees by the `solve` module — so the same evaluator serves all three.
// ─────────────────────────────────────────────────────────────────────────────

/// Default automatic-expansion depth when a query omits `depth N;`.
pub const DEFAULT_DEPTH: usize = 4;

/// A whole query: optional recursion bound, optional `N of` whole-sort sampler,
/// and an ordered list of bindings. The last binding is the target.
#[derive(Debug, Clone)]
pub struct Query {
    pub depth: usize,
    pub take: Option<Take>,
    pub bindings: Vec<Binding>,
}

/// `N of [@seed]` — request N distinct whole sorts (sampled, seeded, deduped).
#[derive(Debug, Clone, Copy)]
pub struct Take {
    pub n: usize,
    pub seed: u64,
}

/// `let name: Role[refinements] = value;` (the `let` is optional — the compact
/// `name: Role = value` form is accepted too).
#[derive(Debug, Clone)]
pub struct Binding {
    pub name: String,
    pub role: String,
    pub refinements: Vec<Refinement>,
    pub value: QValue,
}

/// A refinement on a binding's role, e.g. `Partition[pivot = p]`. `param` must
/// be a referenceable param of the chosen filler; `value` is threaded in (a
/// shared variable, usually) and role-checked against that param.
#[derive(Debug, Clone)]
pub struct Refinement {
    pub param: String,
    pub value: QValue,
}

/// A value in the query language. Its interpretation as a TYPE vs CONST is
/// deferred to the solver, which knows the surrounding param's kind.
#[derive(Debug, Clone)]
pub enum QValue {
    /// `.` / `*` / `?` / `?N@seed` — a hole, with its quantifier.
    Hole(Quant),
    /// A bare identifier: a reference to an earlier binding (a shared variable)
    /// OR a nullary component name. Resolved by environment lookup at solve time.
    Ident(String),
    /// A component application: `quick_sort(partition = part, pivot = p, …)`.
    App { name: String, args: Vec<QArg> },
    /// A literal const token: `32`, `true`, `false`.
    Const(String),
}

/// One argument to a component application.
#[derive(Debug, Clone)]
pub enum QArg {
    /// `name = value` — a slot or named const (disambiguated by the param kind).
    Named { name: String, value: QValue },
    /// A positional const literal (or shared const variable).
    Pos(QValue),
}

/// The four hole quantifiers. Every random form carries an explicit seed so
/// builds are reproducible.
#[derive(Debug, Clone, Copy)]
pub enum Quant {
    /// `.` or `*` — all valid fillers (a cross-product dimension).
    Exhaustive,
    /// `?` — one random filler.
    One { seed: u64 },
    /// `?N@seed` — N distinct random fillers.
    N { n: usize, seed: u64 },
}

#[derive(Debug, PartialEq, Clone)]
enum QTok {
    Ident(String),
    Num(String),
    Colon,
    Semi,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Eq,
    Comma,
    Dot,
    Star,
    Question,
    At,
}

fn tokenize_query(s: &str) -> Result<Vec<QTok>, String> {
    let mut out = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            c if c.is_whitespace() => {
                chars.next();
            }
            '#' => {
                // line comment
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '\n' {
                        break;
                    }
                }
            }
            ':' => { chars.next(); out.push(QTok::Colon); }
            ';' => { chars.next(); out.push(QTok::Semi); }
            '(' => { chars.next(); out.push(QTok::LParen); }
            ')' => { chars.next(); out.push(QTok::RParen); }
            '[' => { chars.next(); out.push(QTok::LBracket); }
            ']' => { chars.next(); out.push(QTok::RBracket); }
            '=' => { chars.next(); out.push(QTok::Eq); }
            ',' => { chars.next(); out.push(QTok::Comma); }
            '.' => { chars.next(); out.push(QTok::Dot); }
            '*' => { chars.next(); out.push(QTok::Star); }
            '?' => { chars.next(); out.push(QTok::Question); }
            '@' => { chars.next(); out.push(QTok::At); }
            c if c.is_alphanumeric() || c == '_' => {
                let mut w = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        w.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if w.chars().all(|c| c.is_ascii_digit()) {
                    out.push(QTok::Num(w));
                } else {
                    out.push(QTok::Ident(w));
                }
            }
            other => return Err(format!("unexpected character `{other}` in query")),
        }
    }
    Ok(out)
}

struct QParser<'a> {
    toks: &'a [QTok],
    pos: usize,
}

impl<'a> QParser<'a> {
    fn peek(&self) -> Option<&QTok> {
        self.toks.get(self.pos)
    }
    fn peek_at(&self, n: usize) -> Option<&QTok> {
        self.toks.get(self.pos + n)
    }
    fn bump(&mut self) -> Option<&QTok> {
        let t = self.toks.get(self.pos);
        self.pos += 1;
        t
    }
    fn eat(&mut self, t: &QTok) -> Result<(), String> {
        if self.peek() == Some(t) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!("expected {t:?}, found {:?}", self.peek()))
        }
    }
    fn ident(&mut self) -> Result<String, String> {
        match self.bump() {
            Some(QTok::Ident(s)) => Ok(s.clone()),
            other => Err(format!("expected an identifier, found {other:?}")),
        }
    }
    fn num(&mut self) -> Result<u64, String> {
        match self.bump() {
            Some(QTok::Num(n)) => n.parse().map_err(|_| format!("bad number `{n}`")),
            other => Err(format!("expected a number, found {other:?}")),
        }
    }
    fn is_keyword(&self, kw: &str) -> bool {
        matches!(self.peek(), Some(QTok::Ident(s)) if s == kw)
    }

    /// `.` | `*` | `?` | `?N` | `?@seed` | `?N@seed`
    fn quant(&mut self) -> Result<Quant, String> {
        match self.peek() {
            Some(QTok::Dot) | Some(QTok::Star) => {
                self.pos += 1;
                Ok(Quant::Exhaustive)
            }
            Some(QTok::Question) => {
                self.pos += 1;
                let n = if matches!(self.peek(), Some(QTok::Num(_))) {
                    Some(self.num()?)
                } else {
                    None
                };
                let seed = if matches!(self.peek(), Some(QTok::At)) {
                    self.pos += 1;
                    self.num()?
                } else {
                    0
                };
                Ok(match n {
                    Some(n) => Quant::N { n: n as usize, seed },
                    None => Quant::One { seed },
                })
            }
            other => Err(format!("expected a hole quantifier, found {other:?}")),
        }
    }

    fn value(&mut self) -> Result<QValue, String> {
        match self.peek() {
            Some(QTok::Dot) | Some(QTok::Star) | Some(QTok::Question) => {
                Ok(QValue::Hole(self.quant()?))
            }
            Some(QTok::Num(n)) => {
                let n = n.clone();
                self.pos += 1;
                Ok(QValue::Const(n))
            }
            Some(QTok::Ident(s)) if s == "true" || s == "false" => {
                let s = s.clone();
                self.pos += 1;
                Ok(QValue::Const(s))
            }
            Some(QTok::Ident(_)) => {
                let name = self.ident()?;
                if matches!(self.peek(), Some(QTok::LParen)) {
                    let args = self.app_args()?;
                    Ok(QValue::App { name, args })
                } else {
                    Ok(QValue::Ident(name))
                }
            }
            other => Err(format!("expected a value, found {other:?}")),
        }
    }

    fn app_args(&mut self) -> Result<Vec<QArg>, String> {
        self.eat(&QTok::LParen)?;
        let mut args = Vec::new();
        while self.peek() != Some(&QTok::RParen) {
            if self.peek().is_none() {
                return Err("unterminated `(`".into());
            }
            // `name = value` (named) vs a positional value.
            let named = matches!(self.peek(), Some(QTok::Ident(_)))
                && matches!(self.peek_at(1), Some(QTok::Eq));
            if named {
                let name = self.ident()?;
                self.eat(&QTok::Eq)?;
                let value = self.value()?;
                args.push(QArg::Named { name, value });
            } else {
                args.push(QArg::Pos(self.value()?));
            }
            if matches!(self.peek(), Some(QTok::Comma)) {
                self.pos += 1;
            }
        }
        self.eat(&QTok::RParen)?;
        Ok(args)
    }

    fn binding(&mut self) -> Result<Binding, String> {
        if self.is_keyword("let") {
            self.pos += 1;
        }
        let name = self.ident()?;
        self.eat(&QTok::Colon)?;
        let role = self.ident()?;
        let mut refinements = Vec::new();
        if matches!(self.peek(), Some(QTok::LBracket)) {
            self.pos += 1;
            while self.peek() != Some(&QTok::RBracket) {
                if self.peek().is_none() {
                    return Err("unterminated `[`".into());
                }
                let param = self.ident()?;
                self.eat(&QTok::Eq)?;
                let value = self.value()?;
                refinements.push(Refinement { param, value });
                if matches!(self.peek(), Some(QTok::Comma)) {
                    self.pos += 1;
                }
            }
            self.eat(&QTok::RBracket)?;
        }
        self.eat(&QTok::Eq)?;
        let value = self.value()?;
        self.eat(&QTok::Semi)?;
        Ok(Binding { name, role, refinements, value })
    }
}

/// Parse a whole query: `[depth N;] [N of [@seed]] <binding>; … <target>;`.
pub fn parse_query(text: &str) -> Result<Query, String> {
    let toks = tokenize_query(text)?;
    let mut p = QParser { toks: &toks, pos: 0 };

    let mut depth = DEFAULT_DEPTH;
    if p.is_keyword("depth") {
        p.pos += 1;
        depth = p.num()? as usize;
        p.eat(&QTok::Semi)?;
    }

    // `N of [@seed]` — only when a Num is immediately followed by `of`.
    let mut take = None;
    if matches!(p.peek(), Some(QTok::Num(_)))
        && matches!(p.peek_at(1), Some(QTok::Ident(s)) if s == "of")
    {
        let n = p.num()? as usize;
        p.pos += 1; // consume `of`
        let seed = if matches!(p.peek(), Some(QTok::At)) {
            p.pos += 1;
            p.num()?
        } else {
            0
        };
        take = Some(Take { n, seed });
    }

    let mut bindings = Vec::new();
    while p.peek().is_some() {
        bindings.push(p.binding()?);
    }
    if bindings.is_empty() {
        return Err("a query needs at least one binding (the target)".into());
    }
    Ok(Query { depth, take, bindings })
}
