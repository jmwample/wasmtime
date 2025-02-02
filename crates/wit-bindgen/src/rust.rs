use crate::{types::TypeInfo, Ownership};
use heck::*;
use wit_parser::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TypeMode {
    Owned,
    AllBorrowed(&'static str),
}

pub trait RustGenerator<'a> {
    fn resolve(&self) -> &'a Resolve;

    fn push_str(&mut self, s: &str);
    fn info(&self, ty: TypeId) -> TypeInfo;
    fn path_to_interface(&self, interface: InterfaceId) -> Option<String>;

    /// This determines whether we generate owning types or (where appropriate)
    /// borrowing types.
    ///
    /// For example, when generating a type which is only used as a parameter to
    /// a guest-exported function, there is no need for it to own its fields.
    /// However, constructing deeply-nested borrows (e.g. `&[&[&[&str]]]]` for
    /// `list<list<list<string>>>`) can be very awkward, so by default we
    /// generate owning types and use only shallow borrowing at the top level
    /// inside function signatures.
    fn ownership(&self) -> Ownership;

    fn print_ty(&mut self, ty: &Type, mode: TypeMode) {
        match ty {
            Type::Id(t) => self.print_tyid(*t, mode),
            Type::Bool => self.push_str("bool"),
            Type::U8 => self.push_str("u8"),
            Type::U16 => self.push_str("u16"),
            Type::U32 => self.push_str("u32"),
            Type::U64 => self.push_str("u64"),
            Type::S8 => self.push_str("i8"),
            Type::S16 => self.push_str("i16"),
            Type::S32 => self.push_str("i32"),
            Type::S64 => self.push_str("i64"),
            Type::Float32 => self.push_str("f32"),
            Type::Float64 => self.push_str("f64"),
            Type::Char => self.push_str("char"),
            Type::String => match mode {
                TypeMode::AllBorrowed(lt) => {
                    self.push_str("&");
                    if lt != "'_" {
                        self.push_str(lt);
                        self.push_str(" ");
                    }
                    self.push_str("str");
                }
                TypeMode::Owned => self.push_str("String"),
            },
        }
    }

    fn print_optional_ty(&mut self, ty: Option<&Type>, mode: TypeMode) {
        match ty {
            Some(ty) => self.print_ty(ty, mode),
            None => self.push_str("()"),
        }
    }

    fn print_tyid(&mut self, id: TypeId, mode: TypeMode) {
        let info = self.info(id);
        let lt = self.lifetime_for(&info, mode);
        let ty = &self.resolve().types[id];
        if ty.name.is_some() {
            // If this type has a list internally, no lifetime is being printed,
            // but we're in a borrowed mode, then that means we're in a borrowed
            // context and don't want ownership of the type but we're using an
            // owned type definition. Inject a `&` in front to indicate that, at
            // the API level, ownership isn't required.
            if info.has_list && lt.is_none() {
                if let TypeMode::AllBorrowed(lt) = mode {
                    self.push_str("&");
                    if lt != "'_" {
                        self.push_str(lt);
                        self.push_str(" ");
                    }
                }
            }
            let name = if lt.is_some() {
                self.param_name(id)
            } else {
                self.result_name(id)
            };
            if let TypeOwner::Interface(id) = ty.owner {
                if let Some(path) = self.path_to_interface(id) {
                    self.push_str(&path);
                    self.push_str("::");
                }
            }
            self.push_str(&name);

            // If the type recursively owns data and it's a
            // variant/record/list, then we need to place the
            // lifetime parameter on the type as well.
            if info.has_list && needs_generics(self.resolve(), &ty.kind) {
                self.print_generics(lt);
            }

            return;

            fn needs_generics(resolve: &Resolve, ty: &TypeDefKind) -> bool {
                match ty {
                    TypeDefKind::Variant(_)
                    | TypeDefKind::Record(_)
                    | TypeDefKind::Option(_)
                    | TypeDefKind::Result(_)
                    | TypeDefKind::Future(_)
                    | TypeDefKind::Stream(_)
                    | TypeDefKind::List(_)
                    | TypeDefKind::Flags(_)
                    | TypeDefKind::Enum(_)
                    | TypeDefKind::Tuple(_)
                    | TypeDefKind::Handle(_)
                    | TypeDefKind::Resource => true,
                    TypeDefKind::Type(Type::Id(t)) => {
                        needs_generics(resolve, &resolve.types[*t].kind)
                    }
                    TypeDefKind::Type(Type::String) => true,
                    TypeDefKind::Type(_) => false,
                    TypeDefKind::Unknown => unreachable!(),
                }
            }
        }

        match &ty.kind {
            TypeDefKind::List(t) => self.print_list(t, mode),

            TypeDefKind::Option(t) => {
                self.push_str("Option<");
                self.print_ty(t, mode);
                self.push_str(">");
            }

            TypeDefKind::Result(r) => {
                self.push_str("Result<");
                self.print_optional_ty(r.ok.as_ref(), mode);
                self.push_str(",");
                self.print_optional_ty(r.err.as_ref(), mode);
                self.push_str(">");
            }

            TypeDefKind::Variant(_) => panic!("unsupported anonymous variant"),

            // Tuple-like records are mapped directly to Rust tuples of
            // types. Note the trailing comma after each member to
            // appropriately handle 1-tuples.
            TypeDefKind::Tuple(t) => {
                self.push_str("(");
                for ty in t.types.iter() {
                    self.print_ty(ty, mode);
                    self.push_str(",");
                }
                self.push_str(")");
            }
            TypeDefKind::Record(_) => {
                panic!("unsupported anonymous type reference: record")
            }
            TypeDefKind::Flags(_) => {
                panic!("unsupported anonymous type reference: flags")
            }
            TypeDefKind::Enum(_) => {
                panic!("unsupported anonymous type reference: enum")
            }
            TypeDefKind::Future(ty) => {
                self.push_str("Future<");
                self.print_optional_ty(ty.as_ref(), mode);
                self.push_str(">");
            }
            TypeDefKind::Stream(stream) => {
                self.push_str("Stream<");
                self.print_optional_ty(stream.element.as_ref(), mode);
                self.push_str(",");
                self.print_optional_ty(stream.end.as_ref(), mode);
                self.push_str(">");
            }

            TypeDefKind::Handle(_) => todo!("#6722"),
            TypeDefKind::Resource => todo!("#6722"),

            TypeDefKind::Type(t) => self.print_ty(t, mode),
            TypeDefKind::Unknown => unreachable!(),
        }
    }

    fn print_list(&mut self, ty: &Type, mode: TypeMode) {
        let next_mode = if matches!(self.ownership(), Ownership::Owning) {
            TypeMode::Owned
        } else {
            mode
        };
        match mode {
            TypeMode::AllBorrowed(lt) => {
                self.push_str("&");
                if lt != "'_" {
                    self.push_str(lt);
                    self.push_str(" ");
                }
                self.push_str("[");
                self.print_ty(ty, next_mode);
                self.push_str("]");
            }
            TypeMode::Owned => {
                self.push_str("Vec<");
                self.print_ty(ty, next_mode);
                self.push_str(">");
            }
        }
    }

    fn print_generics(&mut self, lifetime: Option<&str>) {
        if lifetime.is_none() {
            return;
        }
        self.push_str("<");
        if let Some(lt) = lifetime {
            self.push_str(lt);
            self.push_str(",");
        }
        self.push_str(">");
    }

    fn modes_of(&self, ty: TypeId) -> Vec<(String, TypeMode)> {
        let info = self.info(ty);
        if !info.owned && !info.borrowed {
            return Vec::new();
        }
        let mut result = Vec::new();
        let first_mode =
            if info.owned || !info.borrowed || matches!(self.ownership(), Ownership::Owning) {
                TypeMode::Owned
            } else {
                assert!(!self.uses_two_names(&info));
                TypeMode::AllBorrowed("'a")
            };
        result.push((self.result_name(ty), first_mode));
        if self.uses_two_names(&info) {
            result.push((self.param_name(ty), TypeMode::AllBorrowed("'a")));
        }
        result
    }

    fn param_name(&self, ty: TypeId) -> String {
        let info = self.info(ty);
        let name = self.resolve().types[ty]
            .name
            .as_ref()
            .unwrap()
            .to_upper_camel_case();
        if self.uses_two_names(&info) {
            format!("{}Param", name)
        } else {
            name
        }
    }

    fn result_name(&self, ty: TypeId) -> String {
        let info = self.info(ty);
        let name = self.resolve().types[ty]
            .name
            .as_ref()
            .unwrap()
            .to_upper_camel_case();
        if self.uses_two_names(&info) {
            format!("{}Result", name)
        } else {
            name
        }
    }

    fn uses_two_names(&self, info: &TypeInfo) -> bool {
        info.has_list
            && info.borrowed
            && info.owned
            && matches!(
                self.ownership(),
                Ownership::Borrowing {
                    duplicate_if_necessary: true
                }
            )
    }

    fn lifetime_for(&self, info: &TypeInfo, mode: TypeMode) -> Option<&'static str> {
        if matches!(self.ownership(), Ownership::Owning) {
            return None;
        }
        let lt = match mode {
            TypeMode::AllBorrowed(s) => s,
            _ => return None,
        };
        // No lifetimes needed unless this has a list.
        if !info.has_list {
            return None;
        }
        // If two names are used then this type will have an owned and a
        // borrowed copy and the borrowed copy is being used, so it needs a
        // lifetime. Otherwise if it's only borrowed and not owned then this can
        // also use a lifetime since it's not needed in two contexts and only
        // the borrowed version of the structure was generated.
        if self.uses_two_names(info) || (info.borrowed && !info.owned) {
            Some(lt)
        } else {
            None
        }
    }
}

/// Translate `name` to a Rust `snake_case` identifier.
pub fn to_rust_ident(name: &str) -> String {
    match name {
        // Escape Rust keywords.
        // Source: https://doc.rust-lang.org/reference/keywords.html
        "as" => "as_".into(),
        "break" => "break_".into(),
        "const" => "const_".into(),
        "continue" => "continue_".into(),
        "crate" => "crate_".into(),
        "else" => "else_".into(),
        "enum" => "enum_".into(),
        "extern" => "extern_".into(),
        "false" => "false_".into(),
        "fn" => "fn_".into(),
        "for" => "for_".into(),
        "if" => "if_".into(),
        "impl" => "impl_".into(),
        "in" => "in_".into(),
        "let" => "let_".into(),
        "loop" => "loop_".into(),
        "match" => "match_".into(),
        "mod" => "mod_".into(),
        "move" => "move_".into(),
        "mut" => "mut_".into(),
        "pub" => "pub_".into(),
        "ref" => "ref_".into(),
        "return" => "return_".into(),
        "self" => "self_".into(),
        "static" => "static_".into(),
        "struct" => "struct_".into(),
        "super" => "super_".into(),
        "trait" => "trait_".into(),
        "true" => "true_".into(),
        "type" => "type_".into(),
        "unsafe" => "unsafe_".into(),
        "use" => "use_".into(),
        "where" => "where_".into(),
        "while" => "while_".into(),
        "async" => "async_".into(),
        "await" => "await_".into(),
        "dyn" => "dyn_".into(),
        "abstract" => "abstract_".into(),
        "become" => "become_".into(),
        "box" => "box_".into(),
        "do" => "do_".into(),
        "final" => "final_".into(),
        "macro" => "macro_".into(),
        "override" => "override_".into(),
        "priv" => "priv_".into(),
        "typeof" => "typeof_".into(),
        "unsized" => "unsized_".into(),
        "virtual" => "virtual_".into(),
        "yield" => "yield_".into(),
        "try" => "try_".into(),
        s => s.to_snake_case(),
    }
}

/// Translate `name` to a Rust `UpperCamelCase` identifier.
pub fn to_rust_upper_camel_case(name: &str) -> String {
    match name {
        // We use `Host` as the name of the trait for host implementations
        // to fill in, so rename it if "Host" is used as a regular identifier.
        "host" => "Host_".into(),
        s => s.to_upper_camel_case(),
    }
}
