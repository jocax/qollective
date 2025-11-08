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
#[doc = "Tests additionalProperties for HashMap generation"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://schemas.qollective.io/test/additional-properties/v1.0.0\","]
#[doc = "  \"title\": \"Additional Properties Test Schema\","]
#[doc = "  \"description\": \"Tests additionalProperties for HashMap generation\","]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/Configuration\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/Node\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/NodeMap\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/StringMap\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/WordList\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum AdditionalPropertiesTestSchema {
    Configuration(Configuration),
    Node(Node),
    NodeMap(NodeMap),
    StringMap(StringMap),
    WordList(WordList),
}
impl ::std::convert::From<&Self> for AdditionalPropertiesTestSchema {
    fn from(value: &AdditionalPropertiesTestSchema) -> Self {
        value.clone()
    }
}
impl ::std::convert::From<Configuration> for AdditionalPropertiesTestSchema {
    fn from(value: Configuration) -> Self {
        Self::Configuration(value)
    }
}
impl ::std::convert::From<Node> for AdditionalPropertiesTestSchema {
    fn from(value: Node) -> Self {
        Self::Node(value)
    }
}
impl ::std::convert::From<NodeMap> for AdditionalPropertiesTestSchema {
    fn from(value: NodeMap) -> Self {
        Self::NodeMap(value)
    }
}
impl ::std::convert::From<StringMap> for AdditionalPropertiesTestSchema {
    fn from(value: StringMap) -> Self {
        Self::StringMap(value)
    }
}
impl ::std::convert::From<WordList> for AdditionalPropertiesTestSchema {
    fn from(value: WordList) -> Self {
        Self::WordList(value)
    }
}
#[doc = "`Configuration`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"name\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"$ref\": \"#/$defs/StringMap\""]
#[doc = "    },"]
#[doc = "    \"name\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 100,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct Configuration {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub metadata: ::std::option::Option<StringMap>,
    pub name: ConfigurationName,
}
impl ::std::convert::From<&Configuration> for Configuration {
    fn from(value: &Configuration) -> Self {
        value.clone()
    }
}
impl Configuration {
    pub fn builder() -> builder::Configuration {
        Default::default()
    }
}
#[doc = "`ConfigurationName`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 100,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ConfigurationName(::std::string::String);
impl ::std::ops::Deref for ConfigurationName {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ConfigurationName> for ::std::string::String {
    fn from(value: ConfigurationName) -> Self {
        value.0
    }
}
impl ::std::convert::From<&ConfigurationName> for ConfigurationName {
    fn from(value: &ConfigurationName) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for ConfigurationName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ConfigurationName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ConfigurationName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ConfigurationName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ConfigurationName {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`Node`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"id\","]
#[doc = "    \"label\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    },"]
#[doc = "    \"label\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 100,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    },"]
#[doc = "    \"weight\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct Node {
    pub id: ::uuid::Uuid,
    pub label: NodeLabel,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub weight: ::std::option::Option<f64>,
}
impl ::std::convert::From<&Node> for Node {
    fn from(value: &Node) -> Self {
        value.clone()
    }
}
impl Node {
    pub fn builder() -> builder::Node {
        Default::default()
    }
}
#[doc = "`NodeLabel`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 100,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct NodeLabel(::std::string::String);
impl ::std::ops::Deref for NodeLabel {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<NodeLabel> for ::std::string::String {
    fn from(value: NodeLabel) -> Self {
        value.0
    }
}
impl ::std::convert::From<&NodeLabel> for NodeLabel {
    fn from(value: &NodeLabel) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for NodeLabel {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 100usize {
            return Err("longer than 100 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for NodeLabel {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for NodeLabel {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for NodeLabel {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for NodeLabel {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "A map of nodes where keys are node IDs"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"A map of nodes where keys are node IDs\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"additionalProperties\": {"]
#[doc = "    \"$ref\": \"#/$defs/Node\""]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
#[serde(transparent)]
pub struct NodeMap(pub ::std::collections::HashMap<::std::string::String, Node>);
impl ::std::ops::Deref for NodeMap {
    type Target = ::std::collections::HashMap<::std::string::String, Node>;
    fn deref(&self) -> &::std::collections::HashMap<::std::string::String, Node> {
        &self.0
    }
}
impl ::std::convert::From<NodeMap> for ::std::collections::HashMap<::std::string::String, Node> {
    fn from(value: NodeMap) -> Self {
        value.0
    }
}
impl ::std::convert::From<&NodeMap> for NodeMap {
    fn from(value: &NodeMap) -> Self {
        value.clone()
    }
}
impl ::std::convert::From<::std::collections::HashMap<::std::string::String, Node>> for NodeMap {
    fn from(value: ::std::collections::HashMap<::std::string::String, Node>) -> Self {
        Self(value)
    }
}
#[doc = "A map with string values"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"A map with string values\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"additionalProperties\": {"]
#[doc = "    \"type\": \"string\""]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
#[serde(transparent)]
pub struct StringMap(pub ::std::collections::HashMap<::std::string::String, ::std::string::String>);
impl ::std::ops::Deref for StringMap {
    type Target = ::std::collections::HashMap<::std::string::String, ::std::string::String>;
    fn deref(&self) -> &::std::collections::HashMap<::std::string::String, ::std::string::String> {
        &self.0
    }
}
impl ::std::convert::From<StringMap>
    for ::std::collections::HashMap<::std::string::String, ::std::string::String>
{
    fn from(value: StringMap) -> Self {
        value.0
    }
}
impl ::std::convert::From<&StringMap> for StringMap {
    fn from(value: &StringMap) -> Self {
        value.clone()
    }
}
impl ::std::convert::From<::std::collections::HashMap<::std::string::String, ::std::string::String>>
    for StringMap
{
    fn from(
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) -> Self {
        Self(value)
    }
}
#[doc = "A map where each key has an array of strings"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"A map where each key has an array of strings\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"additionalProperties\": {"]
#[doc = "    \"type\": \"array\","]
#[doc = "    \"items\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
#[serde(transparent)]
pub struct WordList(
    pub ::std::collections::HashMap<::std::string::String, ::std::vec::Vec<::std::string::String>>,
);
impl ::std::ops::Deref for WordList {
    type Target =
        ::std::collections::HashMap<::std::string::String, ::std::vec::Vec<::std::string::String>>;
    fn deref(
        &self,
    ) -> &::std::collections::HashMap<::std::string::String, ::std::vec::Vec<::std::string::String>>
    {
        &self.0
    }
}
impl ::std::convert::From<WordList>
    for ::std::collections::HashMap<::std::string::String, ::std::vec::Vec<::std::string::String>>
{
    fn from(value: WordList) -> Self {
        value.0
    }
}
impl ::std::convert::From<&WordList> for WordList {
    fn from(value: &WordList) -> Self {
        value.clone()
    }
}
impl
    ::std::convert::From<
        ::std::collections::HashMap<::std::string::String, ::std::vec::Vec<::std::string::String>>,
    > for WordList
{
    fn from(
        value: ::std::collections::HashMap<
            ::std::string::String,
            ::std::vec::Vec<::std::string::String>,
        >,
    ) -> Self {
        Self(value)
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct Configuration {
        metadata:
            ::std::result::Result<::std::option::Option<super::StringMap>, ::std::string::String>,
        name: ::std::result::Result<super::ConfigurationName, ::std::string::String>,
    }
    impl ::std::default::Default for Configuration {
        fn default() -> Self {
            Self {
                metadata: Ok(Default::default()),
                name: Err("no value supplied for name".to_string()),
            }
        }
    }
    impl Configuration {
        pub fn metadata<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::StringMap>>,
            T::Error: ::std::fmt::Display,
        {
            self.metadata = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for metadata: {}", e));
            self
        }
        pub fn name<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ConfigurationName>,
            T::Error: ::std::fmt::Display,
        {
            self.name = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for name: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<Configuration> for super::Configuration {
        type Error = super::error::ConversionError;
        fn try_from(
            value: Configuration,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                metadata: value.metadata?,
                name: value.name?,
            })
        }
    }
    impl ::std::convert::From<super::Configuration> for Configuration {
        fn from(value: super::Configuration) -> Self {
            Self {
                metadata: Ok(value.metadata),
                name: Ok(value.name),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Node {
        id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
        label: ::std::result::Result<super::NodeLabel, ::std::string::String>,
        weight: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
    }
    impl ::std::default::Default for Node {
        fn default() -> Self {
            Self {
                id: Err("no value supplied for id".to_string()),
                label: Err("no value supplied for label".to_string()),
                weight: Ok(Default::default()),
            }
        }
    }
    impl Node {
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
        pub fn label<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::NodeLabel>,
            T::Error: ::std::fmt::Display,
        {
            self.label = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for label: {}", e));
            self
        }
        pub fn weight<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<f64>>,
            T::Error: ::std::fmt::Display,
        {
            self.weight = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for weight: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<Node> for super::Node {
        type Error = super::error::ConversionError;
        fn try_from(value: Node) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                id: value.id?,
                label: value.label?,
                weight: value.weight?,
            })
        }
    }
    impl ::std::convert::From<super::Node> for Node {
        fn from(value: super::Node) -> Self {
            Self {
                id: Ok(value.id),
                label: Ok(value.label),
                weight: Ok(value.weight),
            }
        }
    }
}
