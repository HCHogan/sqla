#[derive(Clone, Debug)]
pub enum AstExpr {
    Column {
        table: &'static str,
        col: &'static str,
    },
    Param {
        idx: usize,
    },
    Bool(bool),
    Int(i64),
    Str(&'static str),
    BinOp {
        op: &'static str,
        lhs: Box<AstExpr>,
        rhs: Box<AstExpr>,
    },
}

#[derive(Clone, Debug)]
pub enum AstFrom {
    Table {
        name: &'static str,
    },
    Join {
        kind: &'static str,
        left: Box<AstFrom>,
        right: Box<AstFrom>,
        on: AstExpr,
    },
}

#[derive(Clone, Debug, Default)]
pub struct AstQuery {
    pub from: Option<AstFrom>,
    pub filter: Option<AstExpr>,
    pub projection: Vec<AstExpr>,
}

impl AstQuery {
    pub fn render_sql(&self) -> String {
        let mut s = String::new();
        s.push_str("SELECT ");
        if self.projection.is_empty() {
            s.push('*');
        } else {
            for (i, e) in self.projection.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                s.push_str(&render_expr(e));
            }
        }
        if let Some(fr) = &self.from {
            s.push_str(" FROM ");
            s.push_str(&render_from(fr));
        }
        if let Some(w) = &self.filter {
            s.push_str(" WHERE ");
            s.push_str(&render_expr(w));
        }
        s
    }
}

fn render_expr(e: &AstExpr) -> String {
    match e {
        AstExpr::Column { table, col } => format!("{}.{}", table, col),
        AstExpr::Param { idx } => format!("${}", idx + 1),
        AstExpr::Bool(b) => {
            if *b {
                "TRUE".into()
            } else {
                "FALSE".into()
            }
        }
        AstExpr::Int(i) => i.to_string(),
        AstExpr::Str(s) => format!("'{}'", s.replace("'", "''")),
        AstExpr::BinOp { op, lhs, rhs } => {
            format!("({} {} {})", render_expr(lhs), op, render_expr(rhs))
        }
    }
}

fn render_from(f: &AstFrom) -> String {
    match f {
        AstFrom::Table { name } => (*name).into(),
        AstFrom::Join {
            kind,
            left,
            right,
            on,
        } => format!(
            "{} {} JOIN {} ON {}",
            render_from(left),
            kind,
            render_from(right),
            render_expr(on)
        ),
    }
}
