//! Maps a Rust type to the Hungarian prefix the convention requires.

use std::collections::HashSet;

use syn::punctuated::Punctuated;
use syn::{GenericArgument, PathArguments, Token, Type, TypeParamBound, TypePath};

use crate::type_scope::TypeScope;

/// Last-segment type names and the prefix each requires.
const VEC_TYPE_PREFIXES: &[(&str, &str)] = &[
    ("bool", "b_"),
    ("i8", "i_"),
    ("i16", "i_"),
    ("i32", "i_"),
    ("i64", "i_"),
    ("i128", "i_"),
    ("isize", "i_"),
    ("u8", "u_"),
    ("u16", "u_"),
    ("u32", "u_"),
    ("u64", "u_"),
    ("u128", "u_"),
    ("usize", "u_"),
    ("f32", "f_"),
    ("f64", "f_"),
    ("char", "ch_"),
    ("String", "str_"),
    ("str", "str_"),
    ("OsString", "str_"),
    ("OsStr", "str_"),
    ("Bytes", "by_"),
    ("BytesMut", "by_"),
    ("Punctuated", "vec_"),
    ("HashMap", "map_"),
    ("Map", "map_"),
    ("BTreeMap", "map_"),
    ("IndexMap", "map_"),
    ("HashSet", "set_"),
    ("BTreeSet", "set_"),
    ("IndexSet", "set_"),
    ("Option", "opt_"),
    ("Result", "res_"),
    ("OffsetDateTime", "dt_"),
    ("Timestamp", "dt_"),
    ("PrimitiveDateTime", "dt_"),
    ("UtcDateTime", "dt_"),
    ("DateTime", "dt_"),
    ("SystemTime", "dt_"),
    ("Instant", "dt_"),
    ("Duration", "dur_"),
    ("Uuid", "uuid_"),
    ("Path", "path_"),
    ("PathBuf", "path_"),
    ("Url", "url_"),
    ("Sender", "tx_"),
    ("UnboundedSender", "tx_"),
    ("SyncSender", "tx_"),
    ("Receiver", "rx_"),
    ("UnboundedReceiver", "rx_"),
    ("Mutex", "mtx_"),
    ("RwLock", "mtx_"),
    ("JoinHandle", "jh_"),
];

/// Wrapper types that take the prefix of their first type argument.
const VEC_STR_TRANSPARENT: &[&str] = &["Arc", "Rc", "Box", "Cow", "RefCell", "Cell", "Pin", "Weak"];

/// Enums from outside the workspace that a source scan cannot discover.
const VEC_STR_EXTERNAL_ENUMS: &[&str] = &[
    "Ordering",
    "IpAddr",
    "SocketAddr",
    "SeekFrom",
    "ErrorKind",
    "Component",
    "Value",
    "Level",
    "Type",
    "Pat",
    "Item",
    "Expr",
    "Stmt",
    "FnArg",
    "Fields",
    "Meta",
    "GenericArgument",
    "GenericParam",
    "PathArguments",
    "TypeParamBound",
    "WherePredicate",
    "ImplItem",
    "TraitItem",
    "ReturnType",
    "Visibility",
];

/// Decides which prefix a binding of a given type must carry.
#[derive(Debug, Clone, Default)]
pub struct TypeClassifier {
    /// Names of enums declared in the scanned sources.
    set_str_enums: HashSet<String>,
}

impl TypeClassifier {
    /// Creates a classifier that knows the given workspace enums.
    pub fn new(set_str_enums: HashSet<String>) -> Self {
        Self { set_str_enums }
    }

    /// Returns the prefix a type requires, or `None` when any prefix is acceptable.
    pub fn expected_for_type(&self, e_ty: &Type, o_scope: &TypeScope) -> Option<&'static str> {
        match e_ty {
            Type::Reference(o_ref) => self.expected_for_type(&o_ref.elem, o_scope),
            Type::Paren(o_paren) => self.expected_for_type(&o_paren.elem, o_scope),
            Type::Group(o_group) => self.expected_for_type(&o_group.elem, o_scope),
            Type::Slice(o_slice) => Some(self.sequence_prefix(Some(&o_slice.elem))),
            Type::Array(o_array) => Some(self.sequence_prefix(Some(&o_array.elem))),
            Type::FnPtr(_) => Some("cb_"),
            Type::ImplTrait(o_impl) => self.bounds_prefix(&o_impl.bounds, o_scope, None),
            Type::TraitObject(o_dyn) => self.bounds_prefix(&o_dyn.bounds, o_scope, Some("o_")),
            Type::Path(o_path) => self.path_prefix(o_path, o_scope),
            _ => None,
        }
    }

    /// Returns the prefix for a path type such as `Vec<u8>` or `Option<T>`.
    fn path_prefix(&self, o_path: &TypePath, o_scope: &TypeScope) -> Option<&'static str> {
        let o_last = o_path.path.segments.last()?;
        let str_name = o_last.ident.to_string();
        if o_path.path.segments.len() == 1 && o_scope.has_generic(&str_name) {
            return o_scope.generic_prefix(&str_name);
        }
        if str_name == "Self" {
            return o_scope.self_prefix();
        }
        if VEC_STR_TRANSPARENT.contains(&str_name.as_str()) {
            return self.wrapped_prefix(&o_last.arguments, o_scope);
        }
        if str_name == "Vec" || str_name == "VecDeque" {
            return Some(self.sequence_prefix(self.first_type_arg(&o_last.arguments)));
        }
        Some(self.named_prefix(&str_name))
    }

    /// Returns the prefix of the wrapped type, or `o_` when nothing is wrapped.
    fn wrapped_prefix(&self, e_args: &PathArguments, o_scope: &TypeScope) -> Option<&'static str> {
        match self.first_type_arg(e_args) {
            Some(e_inner) => self.expected_for_type(e_inner, o_scope),
            None => Some("o_"),
        }
    }

    /// Returns the first type argument inside angle brackets, if any.
    fn first_type_arg<'t>(&self, e_args: &'t PathArguments) -> Option<&'t Type> {
        let PathArguments::AngleBracketed(o_angle) = e_args else {
            return None;
        };
        o_angle.args.iter().find_map(|e_arg| match e_arg {
            GenericArgument::Type(e_ty) => Some(e_ty),
            _ => None,
        })
    }

    /// Returns `by_` for byte sequences and `vec_` for any other sequence.
    fn sequence_prefix(&self, opt_e_elem: Option<&Type>) -> &'static str {
        if opt_e_elem.is_some_and(|e_elem| self.is_u8(e_elem)) {
            return "by_";
        }
        "vec_"
    }

    /// Returns whether a type is exactly `u8`.
    fn is_u8(&self, e_ty: &Type) -> bool {
        matches!(e_ty, Type::Path(o_path) if o_path.path.is_ident("u8"))
    }

    /// Returns the prefix for `impl` or `dyn` bounds.
    fn bounds_prefix(
        &self,
        vec_e_bounds: &Punctuated<TypeParamBound, Token![+]>,
        o_scope: &TypeScope,
        opt_str_fallback: Option<&'static str>,
    ) -> Option<&'static str> {
        if o_scope.has_closure_bound(vec_e_bounds) {
            return Some("cb_");
        }
        let b_is_error = vec_e_bounds
            .iter()
            .filter_map(|e_bound| o_scope.bound_name(e_bound))
            .any(|str_name| str_name == "Error");
        if b_is_error {
            return Some("err_");
        }
        opt_str_fallback
    }

    /// Returns the prefix for a named type that is not a wrapper or sequence.
    fn named_prefix(&self, str_name: &str) -> &'static str {
        let opt_str_known = VEC_TYPE_PREFIXES
            .iter()
            .find(|o_pair| o_pair.0 == str_name)
            .map(|o_pair| o_pair.1);
        if let Some(str_known) = opt_str_known {
            return str_known;
        }
        if str_name.ends_with("Error") {
            return "err_";
        }
        if self.is_enum(str_name) {
            return "e_";
        }
        "o_"
    }

    /// Returns whether a type name is a known enum.
    fn is_enum(&self, str_name: &str) -> bool {
        self.set_str_enums.contains(str_name) || VEC_STR_EXTERNAL_ENUMS.contains(&str_name)
    }
}
