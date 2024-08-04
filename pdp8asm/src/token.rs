use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum TokenKind<'d> {
    DotWord(&'d str),
    Word(&'d str),
    Integer(u16),
    OpenSquare,
    CloseSquare,
    CurrentInst,
    DoubleDot,
}
pub struct Token<'d> {
    pub kind: TokenKind<'d>,
}
pub struct TDisplay<'a, 'd>(pub &'a Token<'d>);
impl Display for TDisplay<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let t = self.0;
        match t.kind {
            TokenKind::DotWord(word) => write!(f, "DotWord(.{:?})",word),
            TokenKind::Word(word) => write!(f, "Word({:?})",word),
            TokenKind::Integer(v) => write!(f, "Int({})",v),
            TokenKind::CurrentInst => write!(f, "$"),
            TokenKind::OpenSquare => write!(f, "["),
            TokenKind::CloseSquare => write!(f, "]"),
            TokenKind::DoubleDot   => write!(f, ":"),
        }
    }
}
