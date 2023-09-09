
//https://pest.rs/#editor

alpha              =  { 'a'..'z' | 'A'..'Z' }
digit              =  { '0'..'9' }
word               =  { (alpha | digit)+ }
angular_convention =  {
  
  | "feat"
  | "build"
  | "chore"
  | "ci"
  | "docs"
  | "ci"
  | "style"
  | "refactor"
  | "perf"
  | "test"
}
colonspace         =  { ": " }
exclamation        =  { "!" }
lpar               =  { "(" }
rpar               =  { ")" }
scope              = _{ lpar ~ (word) ~ rpar }
word_list          = _{ !digit ~ word ~ (" " ~ word)+ }
commit             = _{ angular_convention ~ scope ~ colonspace ~ word_list }
