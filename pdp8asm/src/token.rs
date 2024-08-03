use std::fmt::Display;

#[derive(Debug)]
pub enum TokenKind<'d> {
    Word(&'d str),
    Integer(u8),
    OpenSquare,
    CloseSquare,
    CurrentInst,
}
pub struct Token<'d> {
    pub kind: TokenKind<'d>,
}
pub struct TDisplay<'a, 'd>(pub &'a Token<'d>);
impl Display for TDisplay<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let t = self.0;
        match t.kind {
            TokenKind::Word(word) => write!(f, "Word({:?})",word),
            TokenKind::Integer(v) => write!(f, "Int({})",v),
            TokenKind::CurrentInst => write!(f, "$"),
            TokenKind::OpenSquare => write!(f, "["),
            TokenKind::CloseSquare => write!(f, "]"),
        }
    }
}
