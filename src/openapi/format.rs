use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase", untagged)]
pub enum SchemaFormat {
    /// Use to define additional detail about the value.
    KnownFormat(KnownFormat),
    /// Can be used to provide additional detail about the value when [`SchemaFormat::KnownFormat`]
    /// is not suitable.
    Custom(String),
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum KnownFormat {
    /// 8 bit integer.
    Int8,
    /// 16 bit integer.
    Int16,
    /// 32 bit integer.
    Int32,
    /// 64 bit integer.
    Int64,
    /// 8 bit unsigned integer.
    UInt8,
    /// 16 bit unsigned integer.
    UInt16,
    /// 32 bit unsigned integer.
    UInt32,
    /// 64 bit unsigned integer.
    UInt64,
    /// floating point number.
    Float,
    /// double (floating point) number.
    Double,
    /// base64 encoded chars.
    Byte,
    /// binary data (octet).
    Binary,
    /// ISO-8601 full time format [RFC3339](https://xml2rfc.ietf.org/public/rfc/html/rfc3339.html#anchor14).
    Time,
    /// ISO-8601 full date [RFC3339](https://xml2rfc.ietf.org/public/rfc/html/rfc3339.html#anchor14).
    Date,
    /// ISO-8601 full date time [RFC3339](https://xml2rfc.ietf.org/public/rfc/html/rfc3339.html#anchor14).
    DateTime,
    /// duration format from [RFC3339 Appendix-A](https://datatracker.ietf.org/doc/html/rfc3339#appendix-A).
    Duration,
    /// Hint to UI to obscure input.
    Password,
    /// Used with [`String`] values to indicate value is in UUID format.
    ///
    /// **uuid** feature need to be enabled.
    Uuid,
    /// Used with [`String`] values to indicate value is in ULID format.
    Ulid,
    /// Used with [`String`] values to indicate value is in Url format according to
    /// [RFC3986](https://datatracker.ietf.org/doc/html/rfc3986).
    Uri,
    /// A string instance is valid against this attribute if it is a valid URI Reference
    /// (either a URI or a relative-reference) according to
    /// [RFC3986](https://datatracker.ietf.org/doc/html/rfc3986).
    UriReference,
    /// A string instance is valid against this attribute if it is a
    /// valid IRI, according to [RFC3987](https://datatracker.ietf.org/doc/html/rfc3987).
    Iri,
    /// A string instance is valid against this attribute if it is a valid IRI Reference
    /// (either an IRI or a relative-reference)
    /// according to [RFC3987](https://datatracker.ietf.org/doc/html/rfc3987).
    IriReference,
    /// As defined in "Mailbox" rule [RFC5321](https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.2).
    Email,
    /// As defined by extended "Mailbox" rule [RFC6531](https://datatracker.ietf.org/doc/html/rfc6531#section-3.3).
    IdnEmail,
    /// As defined by [RFC1123](https://datatracker.ietf.org/doc/html/rfc1123#section-2.1), including host names
    /// produced using the Punycode algorithm
    /// specified in [RFC5891](https://datatracker.ietf.org/doc/html/rfc5891#section-4.4).
    Hostname,
    /// As defined by either [RFC1123](https://datatracker.ietf.org/doc/html/rfc1123#section-2.1) as for hostname,
    /// or an internationalized hostname as defined by [RFC5890](https://datatracker.ietf.org/doc/html/rfc5890#section-2.3.2.3).
    IdnHostname,
    /// An IPv4 address according to [RFC2673](https://datatracker.ietf.org/doc/html/rfc2673#section-3.2).
    Ipv4,
    /// An IPv6 address according to [RFC4291](https://datatracker.ietf.org/doc/html/rfc4291#section-2.2).
    Ipv6,
    /// A string instance is a valid URI Template if it is according to
    /// [RFC6570](https://datatracker.ietf.org/doc/html/rfc6570).
    ///
    /// _**Note!**_ There are no separate IRL template.
    UriTemplate,
    /// A valid JSON string representation of a JSON Pointer according to [RFC6901](https://datatracker.ietf.org/doc/html/rfc6901#section-5).
    JsonPointer,
    /// A valid relative JSON Pointer according to [draft-handrews-relative-json-pointer-01](https://datatracker.ietf.org/doc/html/draft-handrews-relative-json-pointer-01).
    RelativeJsonPointer,
    /// Regular expression, which SHOULD be valid according to the
    /// [ECMA-262](https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#ref-ecma262).
    Regex,
}
