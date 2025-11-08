#[doc = r" Error types."]
pub mod error {
    #[doc = r" Error from a `TryFrom` or `FromStr` implementation."]
    pub struct ConversionError(::std::borrow::Cow<'static, str>);
    impl ::std::error::Error for ConversionError {}
    impl ::std::fmt::Display for ConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
            ::std::fmt::Display::fmt(&self.0, f)
        }
    }
    impl ::std::fmt::Debug for ConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
            ::std::fmt::Debug::fmt(&self.0, f)
        }
    }
    impl From<&'static str> for ConversionError {
        fn from(value: &'static str) -> Self {
            Self(value.into())
        }
    }
    impl From<String> for ConversionError {
        fn from(value: String) -> Self {
            Self(value.into())
        }
    }
}
#[doc = "Tests anyOf for flexible union type generation"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://schemas.qollective.io/test/anyof-union/v1.0.0\","]
#[doc = "  \"title\": \"AnyOf Union Test Schema\","]
#[doc = "  \"description\": \"Tests anyOf for flexible union type generation\","]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/Container\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/FlexibleValue\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum AnyOfUnionTestSchema {
    Container(Container),
    FlexibleValue(FlexibleValue),
}
impl ::std::convert::From<&Self> for AnyOfUnionTestSchema {
    fn from(value: &AnyOfUnionTestSchema) -> Self {
        value.clone()
    }
}
impl ::std::convert::From<Container> for AnyOfUnionTestSchema {
    fn from(value: Container) -> Self {
        Self::Container(value)
    }
}
impl ::std::convert::From<FlexibleValue> for AnyOfUnionTestSchema {
    fn from(value: FlexibleValue) -> Self {
        Self::FlexibleValue(value)
    }
}
#[doc = "`Container`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    },"]
#[doc = "    \"value\": {"]
#[doc = "      \"$ref\": \"#/$defs/FlexibleValue\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct Container {
    pub id: ::uuid::Uuid,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub value: ::std::option::Option<FlexibleValue>,
}
impl ::std::convert::From<&Container> for Container {
    fn from(value: &Container) -> Self {
        value.clone()
    }
}
impl Container {
    pub fn builder() -> builder::Container {
        Default::default()
    }
}
#[doc = "A flexible value that can be multiple types"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"A flexible value that can be multiple types\","]
#[doc = "  \"anyOf\": ["]
#[doc = "    {"]
#[doc = "      \"description\": \"String value\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"description\": \"Numeric value\","]
#[doc = "      \"type\": \"number\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"description\": \"Boolean value\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum FlexibleValue {
    Variant0(::std::string::String),
    Variant1(f64),
    Variant2(bool),
}
impl ::std::convert::From<&Self> for FlexibleValue {
    fn from(value: &FlexibleValue) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for FlexibleValue {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if let Ok(v) = value.parse() {
            Ok(Self::Variant0(v))
        } else if let Ok(v) = value.parse() {
            Ok(Self::Variant1(v))
        } else if let Ok(v) = value.parse() {
            Ok(Self::Variant2(v))
        } else {
            Err("string conversion failed for all variants".into())
        }
    }
}
impl ::std::convert::TryFrom<&str> for FlexibleValue {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for FlexibleValue {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for FlexibleValue {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::fmt::Display for FlexibleValue {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::Variant0(x) => x.fmt(f),
            Self::Variant1(x) => x.fmt(f),
            Self::Variant2(x) => x.fmt(f),
        }
    }
}
impl ::std::convert::From<f64> for FlexibleValue {
    fn from(value: f64) -> Self {
        Self::Variant1(value)
    }
}
impl ::std::convert::From<bool> for FlexibleValue {
    fn from(value: bool) -> Self {
        Self::Variant2(value)
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct Container {
        id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
        value: ::std::result::Result<
            ::std::option::Option<super::FlexibleValue>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for Container {
        fn default() -> Self {
            Self {
                id: Err("no value supplied for id".to_string()),
                value: Ok(Default::default()),
            }
        }
    }
    impl Container {
        pub fn id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::uuid::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for id: {}", e));
            self
        }
        pub fn value<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::FlexibleValue>>,
            T::Error: ::std::fmt::Display,
        {
            self.value = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for value: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<Container> for super::Container {
        type Error = super::error::ConversionError;
        fn try_from(
            value: Container,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                id: value.id?,
                value: value.value?,
            })
        }
    }
    impl ::std::convert::From<super::Container> for Container {
        fn from(value: super::Container) -> Self {
            Self {
                id: Ok(value.id),
                value: Ok(value.value),
            }
        }
    }
}
