#[ link(name = "frscript",
        vers = "0.0",
        uuid = "62527026-e47c-44ce-be34-7137e8194772") ];

#[ desc = "FRP Scripting Language" ];
#[ license = "Zlib/libpng" ];
#[ author = "tiffany" ];

#[ crate_type="lib" ];

#[feature(globs)];

pub mod parse;
pub mod grammar;
pub mod ast;
pub mod eval;
pub mod context;
pub mod typechecker;
pub mod macro;
pub mod stdlib;

