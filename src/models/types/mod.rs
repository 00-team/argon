mod def;
mod kotlin;
mod parse;

#[derive(Debug, Clone)]
pub enum ApiPrim {
    Str,
    Int,
    Float,
    Bool,
    File,
    Null,
    Option(Box<ApiType>),
}

pub type ApiObject = Vec<(String, ApiType, bool)>;
pub type ApiUnion = Vec<ApiType>;

#[derive(Debug, Clone)]
pub enum ApiKind {
    Unknown,
    Recursive,
    Ref(String),
    Prim(ApiPrim),

    Array(Box<ApiType>),
    Object(ApiObject),
    /// for example in `typescript`:
    /// ```typescript
    /// type Map = {
    ///     [k: string]: string | number | {obj: boolean}
    /// }
    /// ```
    Map(Box<ApiType>),
    /// for example in `typescript`:
    /// ```typescript
    /// type Union = number | string
    /// ```
    Union(ApiUnion),
    /// for example in `typescript`:
    /// ```typescript
    /// type Combo = { a: string } & { b: number }
    /// ```
    Combo(Vec<ApiType>),
    /// for example in `typescript`:
    /// ```typescript
    /// type StrEnum = 'a' | 'b' | 'c'
    /// ```
    StrEnum(Vec<String>),
    /// for example in `typescript`:
    /// ```typescript
    /// type Tuple = [number, string, { a: boolean }]
    /// ```
    Tuple(Vec<ApiType>),
}

#[derive(Debug, Clone)]
pub struct ApiType {
    pub name: Option<String>,
    pub kind: ApiKind,
}

impl ApiType {
    pub const fn new(name: Option<String>, kind: ApiKind) -> Self {
        Self { name, kind }
    }

    pub const fn is_prim(&self) -> bool {
        matches!(self.kind, ApiKind::Prim(_))
    }

    pub const fn is_null(&self) -> bool {
        matches!(self.kind, ApiKind::Prim(ApiPrim::Null))
    }

    pub const fn is_option(&self) -> bool {
        matches!(self.kind, ApiKind::Prim(ApiPrim::Option(_)))
    }

    pub const fn is_file(&self) -> bool {
        let ApiKind::Prim(prim) = &self.kind else { return false };
        if let ApiPrim::Option(opt) = prim {
            return opt.is_file();
        }
        matches!(prim, ApiPrim::File)
    }

    pub const fn has_from_json(&self) -> bool {
        matches!(
            self.kind,
            ApiKind::Object(_)
                | ApiKind::Union(_)
                | ApiKind::Combo(_)
                | ApiKind::Ref(_)
        )
    }
}

impl From<ApiPrim> for ApiKind {
    fn from(value: ApiPrim) -> Self {
        Self::Prim(value)
    }
}
