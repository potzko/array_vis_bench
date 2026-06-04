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
