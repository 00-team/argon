mod def;
mod parse;

#[derive(Debug, Clone)]
pub enum ApiKind {
    Unknown,
    Str,
    Int,
    Bool,
    File,
    Null,
    Recursive,
    Array(Box<ApiType>),
    Object(Vec<(String, ApiType)>),
    /// for example in `typescript`:
    /// ```typescript
    /// type Union = number | string
    /// ```
    Union(Vec<ApiType>),
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
    pub user_defined: bool,
}

impl ApiType {
    pub const NULL: Self = Self::new(ApiKind::Null);
    pub const RECURSIVE: Self = Self::new(ApiKind::Recursive);

    pub const fn new(kind: ApiKind) -> Self {
        Self { name: None, kind, user_defined: false }
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn is_nullable(&self) -> bool {
        if let ApiKind::Tuple(tup) = &self.kind {
            for at in tup {
                if matches!(at.kind, ApiKind::Null) {
                    return true;
                }
            }
        }

        false
    }
}
