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
#[doc = "`BooleanValue`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"type\","]
#[doc = "    \"value\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"type\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"value\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct BooleanValue {
    #[serde(rename = "type")]
    pub type_: ::std::string::String,
    pub value: bool,
}
impl ::std::convert::From<&BooleanValue> for BooleanValue {
    fn from(value: &BooleanValue) -> Self {
        value.clone()
    }
}
impl BooleanValue {
    pub fn builder() -> builder::BooleanValue {
        Default::default()
    }
}
#[doc = "A configuration value that can be string, number, or boolean"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"A configuration value that can be string, number, or boolean\","]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/StringValue\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/NumberValue\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/BooleanValue\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum ConfigValue {
    StringValue(StringValue),
    NumberValue(NumberValue),
    BooleanValue(BooleanValue),
}
impl ::std::convert::From<&Self> for ConfigValue {
    fn from(value: &ConfigValue) -> Self {
        value.clone()
    }
}
impl ::std::convert::From<StringValue> for ConfigValue {
    fn from(value: StringValue) -> Self {
        Self::StringValue(value)
    }
}
impl ::std::convert::From<NumberValue> for ConfigValue {
    fn from(value: NumberValue) -> Self {
        Self::NumberValue(value)
    }
}
impl ::std::convert::From<BooleanValue> for ConfigValue {
    fn from(value: BooleanValue) -> Self {
        Self::BooleanValue(value)
    }
}
#[doc = "`NumberValue`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"type\","]
#[doc = "    \"value\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"type\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"number\""]
#[doc = "    },"]
#[doc = "    \"value\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct NumberValue {
    #[serde(rename = "type")]
    pub type_: ::std::string::String,
    pub value: f64,
}
impl ::std::convert::From<&NumberValue> for NumberValue {
    fn from(value: &NumberValue) -> Self {
        value.clone()
    }
}
impl NumberValue {
    pub fn builder() -> builder::NumberValue {
        Default::default()
    }
}
#[doc = "Tests oneOf for union type generation in Rust enums"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://schemas.qollective.io/test/oneof-union/v1.0.0\","]
#[doc = "  \"title\": \"OneOf Union Test Schema\","]
#[doc = "  \"description\": \"Tests oneOf for union type generation in Rust enums\","]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/BooleanValue\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/ConfigValue\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/NumberValue\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/StringValue\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum OneOfUnionTestSchema {
    BooleanValue(BooleanValue),
    ConfigValue(ConfigValue),
    NumberValue(NumberValue),
    StringValue(StringValue),
}
impl ::std::convert::From<&Self> for OneOfUnionTestSchema {
    fn from(value: &OneOfUnionTestSchema) -> Self {
        value.clone()
    }
}
impl ::std::convert::From<BooleanValue> for OneOfUnionTestSchema {
    fn from(value: BooleanValue) -> Self {
        Self::BooleanValue(value)
    }
}
impl ::std::convert::From<ConfigValue> for OneOfUnionTestSchema {
    fn from(value: ConfigValue) -> Self {
        Self::ConfigValue(value)
    }
}
impl ::std::convert::From<NumberValue> for OneOfUnionTestSchema {
    fn from(value: NumberValue) -> Self {
        Self::NumberValue(value)
    }
}
impl ::std::convert::From<StringValue> for OneOfUnionTestSchema {
    fn from(value: StringValue) -> Self {
        Self::StringValue(value)
    }
}
#[doc = "`StringValue`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"type\","]
#[doc = "    \"value\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"type\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"string\""]
#[doc = "    },"]
#[doc = "    \"value\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct StringValue {
    #[serde(rename = "type")]
    pub type_: ::std::string::String,
    pub value: ::std::string::String,
}
impl ::std::convert::From<&StringValue> for StringValue {
    fn from(value: &StringValue) -> Self {
        value.clone()
    }
}
impl StringValue {
    pub fn builder() -> builder::StringValue {
        Default::default()
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct BooleanValue {
        type_: ::std::result::Result<::std::string::String, ::std::string::String>,
        value: ::std::result::Result<bool, ::std::string::String>,
    }
    impl ::std::default::Default for BooleanValue {
        fn default() -> Self {
            Self {
                type_: Err("no value supplied for type_".to_string()),
                value: Err("no value supplied for value".to_string()),
            }
        }
    }
    impl BooleanValue {
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {}", e));
            self
        }
        pub fn value<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.value = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for value: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<BooleanValue> for super::BooleanValue {
        type Error = super::error::ConversionError;
        fn try_from(
            value: BooleanValue,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                type_: value.type_?,
                value: value.value?,
            })
        }
    }
    impl ::std::convert::From<super::BooleanValue> for BooleanValue {
        fn from(value: super::BooleanValue) -> Self {
            Self {
                type_: Ok(value.type_),
                value: Ok(value.value),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct NumberValue {
        type_: ::std::result::Result<::std::string::String, ::std::string::String>,
        value: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for NumberValue {
        fn default() -> Self {
            Self {
                type_: Err("no value supplied for type_".to_string()),
                value: Err("no value supplied for value".to_string()),
            }
        }
    }
    impl NumberValue {
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {}", e));
            self
        }
        pub fn value<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.value = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for value: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<NumberValue> for super::NumberValue {
        type Error = super::error::ConversionError;
        fn try_from(
            value: NumberValue,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                type_: value.type_?,
                value: value.value?,
            })
        }
    }
    impl ::std::convert::From<super::NumberValue> for NumberValue {
        fn from(value: super::NumberValue) -> Self {
            Self {
                type_: Ok(value.type_),
                value: Ok(value.value),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct StringValue {
        type_: ::std::result::Result<::std::string::String, ::std::string::String>,
        value: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for StringValue {
        fn default() -> Self {
            Self {
                type_: Err("no value supplied for type_".to_string()),
                value: Err("no value supplied for value".to_string()),
            }
        }
    }
    impl StringValue {
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {}", e));
            self
        }
        pub fn value<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.value = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for value: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<StringValue> for super::StringValue {
        type Error = super::error::ConversionError;
        fn try_from(
            value: StringValue,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                type_: value.type_?,
                value: value.value?,
            })
        }
    }
    impl ::std::convert::From<super::StringValue> for StringValue {
        fn from(value: super::StringValue) -> Self {
            Self {
                type_: Ok(value.type_),
                value: Ok(value.value),
            }
        }
    }
}
