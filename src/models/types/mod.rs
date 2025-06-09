mod def;
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

pub type ApiObject = Vec<(String, ApiType)>;
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
}

impl From<ApiPrim> for ApiKind {
    fn from(value: ApiPrim) -> Self {
        Self::Prim(value)
    }
}
