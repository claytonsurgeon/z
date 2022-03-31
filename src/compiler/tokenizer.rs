use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
   Invalid,
   Skip,
   Newline,
   //
   String,
   Number,
   Integer,
   Decimal,

   //
   Arrow, // ->
   ParenLF,
   ParenRT,
   SquarenLF,
   SquarenRT,
   BracketLF,
   BracketRT,

   Control,

   Post,

   Typ, // :
   Key, // ;
   Com, // ,
   Ref,
   Dot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Meta {
   pub row: usize,
   pub col: usize,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
   pub kind: Kind,
   pub text: String,
   pub meta: Meta,
}

pub fn tokenizer(input: &String) -> Result<Vec<Token>, String> {
   lazy_static! {
      static ref SPEC: Vec<(Kind, Regex)> =
         vec![
            (Kind::Newline, Regex::new(r"^\n").unwrap()),
            // insignificant whitespace
            (Kind::Skip, Regex::new(r"^[\t\v\f\r ]+").unwrap()),

            // Comments
            (Kind::Skip, Regex::new(r"^;;.*").unwrap()),

            (Kind::Arrow, Regex::new(r"^(→|->)").unwrap()),
            (Kind::Post, Regex::new(r"^\|").unwrap()),

            // (Kind::Number, Regex::new(r"^[[:digit:]]([^[:space:]|{}()\[\]])*").unwrap()),
            // (Kind::Control, Regex::new(r"^([^[:space:].|{}()\[\]])+\-\{").unwrap()),

            (Kind::Com, Regex::new(r"^,").unwrap()),
            (Kind::Typ, Regex::new(r"^([^[:space:],|{}()\[\]])+:").unwrap()),
            (Kind::Key, Regex::new(r"^([^[:space:],|{}()\[\]])+;").unwrap()),
            // (Kind::Dot, Regex::new(r"^\.").unwrap()),
            (Kind::Ref, Regex::new(r"^([^[:space:],|{}()\[\]])+").unwrap()),




            // parens
            (Kind::ParenLF, Regex::new(r"^\(").unwrap()),
            (Kind::ParenRT, Regex::new(r"^\)").unwrap()),

            (Kind::SquarenLF, Regex::new(r"^\[").unwrap()),
            (Kind::SquarenRT, Regex::new(r"^\]").unwrap()),

            (Kind::BracketLF, Regex::new(r"^\{").unwrap()),
            (Kind::BracketRT, Regex::new(r"^\}").unwrap()),


            (Kind::String, Regex::new(r#"^"[^"]*("|$)"#).unwrap()),



            (Kind::Invalid, Regex::new(r"^.").unwrap()),
         ];
   }
   // decide what is a word aka reference and what is a number
   lazy_static! {
      static ref WORD: Vec<(Kind, Regex)> = vec![
         (
            Kind::Decimal,
            Regex::new(r"^[[:digit:]]+\.[[:digit:]]*$").unwrap()
         ),
         (
            Kind::Decimal,
            Regex::new(r"^[[:digit:]]*\.[[:digit:]]+$").unwrap()
         ),
         (Kind::Integer, Regex::new(r"^[[:digit:]]+$").unwrap()),
         // (Kind::Ref, Regex::new(r"^.").unwrap()),
      ];
   }

   let mut tokens: Vec<Token> = Vec::new();
   let mut cursor = 0;
   let mut row = 1;
   let mut col = 1;
   let length = input.len();

   'outer: while cursor < length {
      for (kind, re) in &SPEC[..] {
         match re.find(&input[cursor..]) {
            Some(mat) => {
               let token_text = &input[cursor..cursor + mat.end()];
               let text = token_text.to_string();
               let mut t = Token {
                  kind: *kind,
                  text,
                  meta: Meta { col, row },
               };
               col += mat.end();

               match kind {
                  Kind::Newline => {
                     row += 1;
                     col = 1;
                  }
                  Kind::Skip => {}
                  Kind::Ref => {
                     // let x = true;
                     for (kind, re) in &WORD[..] {
                        if re.is_match(&t.text) {
                           t.kind = *kind;
                           break;
                        }
                     }
                     tokens.push(t);
                  }
                  _ => {
                     tokens.push(t);
                  }
               }

               cursor += mat.end();
               continue 'outer;
            }
            None => {}
         }
      }
   }
   Ok(tokens)
}
