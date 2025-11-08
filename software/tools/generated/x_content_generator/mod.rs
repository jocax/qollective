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
#[doc = "Target age group for content generation"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Target age group for content generation\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"6-8\","]
#[doc = "    \"9-11\","]
#[doc = "    \"12-14\","]
#[doc = "    \"15-17\","]
#[doc = "    \"+18\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum AgeGroup {
    #[serde(rename = "6-8")]
    X68,
    #[serde(rename = "9-11")]
    X911,
    #[serde(rename = "12-14")]
    X1214,
    #[serde(rename = "15-17")]
    X1517,
    #[serde(rename = "+18")]
    X18,
}
impl ::std::convert::From<&Self> for AgeGroup {
    fn from(value: &AgeGroup) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for AgeGroup {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::X68 => write!(f, "6-8"),
            Self::X911 => write!(f, "9-11"),
            Self::X1214 => write!(f, "12-14"),
            Self::X1517 => write!(f, "15-17"),
            Self::X18 => write!(f, "+18"),
        }
    }
}
impl ::std::str::FromStr for AgeGroup {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "6-8" => Ok(Self::X68),
            "9-11" => Ok(Self::X911),
            "12-14" => Ok(Self::X1214),
            "15-17" => Ok(Self::X1517),
            "+18" => Ok(Self::X18),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for AgeGroup {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for AgeGroup {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for AgeGroup {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "External API version"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"External API version\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"v1\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum ApiVersion {
    #[serde(rename = "v1")]
    V1,
}
impl ::std::convert::From<&Self> for ApiVersion {
    fn from(value: &ApiVersion) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::V1 => write!(f, "v1"),
        }
    }
}
impl ::std::str::FromStr for ApiVersion {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "v1" => Ok(Self::V1),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ApiVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ApiVersion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ApiVersion {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`BatchInfo`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"batch_id\","]
#[doc = "    \"batch_index\","]
#[doc = "    \"batch_size\","]
#[doc = "    \"total_batches\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"batch_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    },"]
#[doc = "    \"batch_index\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 255.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"batch_size\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 255.0,"]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"total_batches\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct BatchInfo {
    pub batch_id: ::uuid::Uuid,
    pub batch_index: u8,
    pub batch_size: ::std::num::NonZeroU64,
    pub total_batches: ::std::num::NonZeroU64,
}
impl ::std::convert::From<&BatchInfo> for BatchInfo {
    fn from(value: &BatchInfo) -> Self {
        value.clone()
    }
}
impl BatchInfo {
    pub fn builder() -> builder::BatchInfo {
        Default::default()
    }
}
#[doc = "`Choice`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"id\","]
#[doc = "    \"next_node_id\","]
#[doc = "    \"text\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    },"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"type\": \"object\""]
#[doc = "    },"]
#[doc = "    \"next_node_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    },"]
#[doc = "    \"text\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 200,"]
#[doc = "      \"minLength\": 10"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct Choice {
    pub id: ::uuid::Uuid,
    #[serde(default, skip_serializing_if = "::serde_json::Map::is_empty")]
    pub metadata: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    pub next_node_id: ::uuid::Uuid,
    pub text: ChoiceText,
}
impl ::std::convert::From<&Choice> for Choice {
    fn from(value: &Choice) -> Self {
        value.clone()
    }
}
impl Choice {
    pub fn builder() -> builder::Choice {
        Default::default()
    }
}
#[doc = "`ChoiceText`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 200,"]
#[doc = "  \"minLength\": 10"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ChoiceText(::std::string::String);
impl ::std::ops::Deref for ChoiceText {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ChoiceText> for ::std::string::String {
    fn from(value: ChoiceText) -> Self {
        value.0
    }
}
impl ::std::convert::From<&ChoiceText> for ChoiceText {
    fn from(value: &ChoiceText) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for ChoiceText {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 200usize {
            return Err("longer than 200 characters".into());
        }
        if value.chars().count() < 10usize {
            return Err("shorter than 10 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ChoiceText {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ChoiceText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ChoiceText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ChoiceText {
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
#[doc = "`ConstraintResult`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"correction_capability\","]
#[doc = "    \"corrections\","]
#[doc = "    \"missing_elements\","]
#[doc = "    \"required_elements_present\","]
#[doc = "    \"theme_consistency_score\","]
#[doc = "    \"vocabulary_violations\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"correction_capability\": {"]
#[doc = "      \"$ref\": \"#/$defs/CorrectionCapability\""]
#[doc = "    },"]
#[doc = "    \"corrections\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/CorrectionSuggestion\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"missing_elements\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"required_elements_present\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"theme_consistency_score\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"vocabulary_violations\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/VocabularyViolation\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct ConstraintResult {
    pub correction_capability: CorrectionCapability,
    pub corrections: ::std::vec::Vec<CorrectionSuggestion>,
    pub missing_elements: ::std::vec::Vec<::std::string::String>,
    pub required_elements_present: bool,
    pub theme_consistency_score: f64,
    pub vocabulary_violations: ::std::vec::Vec<VocabularyViolation>,
}
impl ::std::convert::From<&ConstraintResult> for ConstraintResult {
    fn from(value: &ConstraintResult) -> Self {
        value.clone()
    }
}
impl ConstraintResult {
    pub fn builder() -> builder::ConstraintResult {
        Default::default()
    }
}
#[doc = "`Content`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"choices\","]
#[doc = "    \"convergence_point\","]
#[doc = "    \"next_nodes\","]
#[doc = "    \"node_id\","]
#[doc = "    \"text\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"choices\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/Choice\""]
#[doc = "      },"]
#[doc = "      \"maxItems\": 4,"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"convergence_point\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"educational_content\": {"]
#[doc = "      \"$ref\": \"#/$defs/EducationalContent\""]
#[doc = "    },"]
#[doc = "    \"next_nodes\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"format\": \"uuid\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"node_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    },"]
#[doc = "    \"text\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 1000,"]
#[doc = "      \"minLength\": 50"]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"interactive_story_node\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct Content {
    pub choices: ::std::vec::Vec<Choice>,
    pub convergence_point: bool,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub educational_content: ::std::option::Option<EducationalContent>,
    pub next_nodes: ::std::vec::Vec<::uuid::Uuid>,
    pub node_id: ::uuid::Uuid,
    pub text: ContentText,
    #[serde(rename = "type")]
    pub type_: ::std::string::String,
}
impl ::std::convert::From<&Content> for Content {
    fn from(value: &Content) -> Self {
        value.clone()
    }
}
impl Content {
    pub fn builder() -> builder::Content {
        Default::default()
    }
}
#[doc = "`ContentNode`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"content\","]
#[doc = "    \"id\","]
#[doc = "    \"incoming_edges\","]
#[doc = "    \"outgoing_edges\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"content\": {"]
#[doc = "      \"$ref\": \"#/$defs/Content\""]
#[doc = "    },"]
#[doc = "    \"generation_metadata\": {"]
#[doc = "      \"type\": \"object\""]
#[doc = "    },"]
#[doc = "    \"id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    },"]
#[doc = "    \"incoming_edges\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"outgoing_edges\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct ContentNode {
    pub content: Content,
    #[serde(default, skip_serializing_if = "::serde_json::Map::is_empty")]
    pub generation_metadata: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    pub id: ::uuid::Uuid,
    pub incoming_edges: u64,
    pub outgoing_edges: u64,
}
impl ::std::convert::From<&ContentNode> for ContentNode {
    fn from(value: &ContentNode) -> Self {
        value.clone()
    }
}
impl ContentNode {
    pub fn builder() -> builder::ContentNode {
        Default::default()
    }
}
#[doc = "`ContentReference`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"content\","]
#[doc = "    \"temp_node_id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"content\": {"]
#[doc = "      \"$ref\": \"#/$defs/Content\""]
#[doc = "    },"]
#[doc = "    \"temp_node_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct ContentReference {
    pub content: Content,
    pub temp_node_id: ::uuid::Uuid,
}
impl ::std::convert::From<&ContentReference> for ContentReference {
    fn from(value: &ContentReference) -> Self {
        value.clone()
    }
}
impl ContentReference {
    pub fn builder() -> builder::ContentReference {
        Default::default()
    }
}
#[doc = "`ContentText`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 1000,"]
#[doc = "  \"minLength\": 50"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ContentText(::std::string::String);
impl ::std::ops::Deref for ContentText {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ContentText> for ::std::string::String {
    fn from(value: ContentText) -> Self {
        value.0
    }
}
impl ::std::convert::From<&ContentText> for ContentText {
    fn from(value: &ContentText) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for ContentText {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 1000usize {
            return Err("longer than 1000 characters".into());
        }
        if value.chars().count() < 50usize {
            return Err("shorter than 50 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ContentText {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ContentText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ContentText {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ContentText {
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
#[doc = "Pattern for how story branches converge"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Pattern for how story branches converge\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"SingleConvergence\","]
#[doc = "    \"MultipleConvergence\","]
#[doc = "    \"EndOnly\","]
#[doc = "    \"PureBranching\","]
#[doc = "    \"ParallelPaths\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum ConvergencePattern {
    SingleConvergence,
    MultipleConvergence,
    EndOnly,
    PureBranching,
    ParallelPaths,
}
impl ::std::convert::From<&Self> for ConvergencePattern {
    fn from(value: &ConvergencePattern) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for ConvergencePattern {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::SingleConvergence => write!(f, "SingleConvergence"),
            Self::MultipleConvergence => write!(f, "MultipleConvergence"),
            Self::EndOnly => write!(f, "EndOnly"),
            Self::PureBranching => write!(f, "PureBranching"),
            Self::ParallelPaths => write!(f, "ParallelPaths"),
        }
    }
}
impl ::std::str::FromStr for ConvergencePattern {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "SingleConvergence" => Ok(Self::SingleConvergence),
            "MultipleConvergence" => Ok(Self::MultipleConvergence),
            "EndOnly" => Ok(Self::EndOnly),
            "PureBranching" => Ok(Self::PureBranching),
            "ParallelPaths" => Ok(Self::ParallelPaths),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ConvergencePattern {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ConvergencePattern {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ConvergencePattern {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Validation service correction capability"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Validation service correction capability\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"CanFixLocally\","]
#[doc = "    \"NeedsRevision\","]
#[doc = "    \"NoFixPossible\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum CorrectionCapability {
    CanFixLocally,
    NeedsRevision,
    NoFixPossible,
}
impl ::std::convert::From<&Self> for CorrectionCapability {
    fn from(value: &CorrectionCapability) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for CorrectionCapability {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::CanFixLocally => write!(f, "CanFixLocally"),
            Self::NeedsRevision => write!(f, "NeedsRevision"),
            Self::NoFixPossible => write!(f, "NoFixPossible"),
        }
    }
}
impl ::std::str::FromStr for CorrectionCapability {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "CanFixLocally" => Ok(Self::CanFixLocally),
            "NeedsRevision" => Ok(Self::NeedsRevision),
            "NoFixPossible" => Ok(Self::NoFixPossible),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CorrectionCapability {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CorrectionCapability {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CorrectionCapability {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`CorrectionSuggestion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"field\","]
#[doc = "    \"issue\","]
#[doc = "    \"severity\","]
#[doc = "    \"suggestion\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"field\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"issue\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"severity\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"low\","]
#[doc = "        \"medium\","]
#[doc = "        \"high\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"suggestion\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct CorrectionSuggestion {
    pub field: ::std::string::String,
    pub issue: ::std::string::String,
    pub severity: CorrectionSuggestionSeverity,
    pub suggestion: ::std::string::String,
}
impl ::std::convert::From<&CorrectionSuggestion> for CorrectionSuggestion {
    fn from(value: &CorrectionSuggestion) -> Self {
        value.clone()
    }
}
impl CorrectionSuggestion {
    pub fn builder() -> builder::CorrectionSuggestion {
        Default::default()
    }
}
#[doc = "`CorrectionSuggestionSeverity`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"low\","]
#[doc = "    \"medium\","]
#[doc = "    \"high\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum CorrectionSuggestionSeverity {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
}
impl ::std::convert::From<&Self> for CorrectionSuggestionSeverity {
    fn from(value: &CorrectionSuggestionSeverity) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for CorrectionSuggestionSeverity {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
        }
    }
}
impl ::std::str::FromStr for CorrectionSuggestionSeverity {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CorrectionSuggestionSeverity {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CorrectionSuggestionSeverity {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CorrectionSuggestionSeverity {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`CorrectionSummary`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"attempts\","]
#[doc = "    \"correction_type\","]
#[doc = "    \"node_id\","]
#[doc = "    \"success\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"attempts\": {"]
#[doc = "      \"description\": \"Number of attempts to correct this node\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"correction_type\": {"]
#[doc = "      \"description\": \"Type of correction applied\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"LocalFix\","]
#[doc = "        \"Regenerate\","]
#[doc = "        \"Skip\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"node_id\": {"]
#[doc = "      \"description\": \"ID of node that was corrected\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"success\": {"]
#[doc = "      \"description\": \"Whether the correction succeeded\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct CorrectionSummary {
    #[doc = "Number of attempts to correct this node"]
    pub attempts: ::std::num::NonZeroU64,
    #[doc = "Type of correction applied"]
    pub correction_type: CorrectionSummaryCorrectionType,
    #[doc = "ID of node that was corrected"]
    pub node_id: ::std::string::String,
    #[doc = "Whether the correction succeeded"]
    pub success: bool,
}
impl ::std::convert::From<&CorrectionSummary> for CorrectionSummary {
    fn from(value: &CorrectionSummary) -> Self {
        value.clone()
    }
}
impl CorrectionSummary {
    pub fn builder() -> builder::CorrectionSummary {
        Default::default()
    }
}
#[doc = "Type of correction applied"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Type of correction applied\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"LocalFix\","]
#[doc = "    \"Regenerate\","]
#[doc = "    \"Skip\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum CorrectionSummaryCorrectionType {
    LocalFix,
    Regenerate,
    Skip,
}
impl ::std::convert::From<&Self> for CorrectionSummaryCorrectionType {
    fn from(value: &CorrectionSummaryCorrectionType) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for CorrectionSummaryCorrectionType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::LocalFix => write!(f, "LocalFix"),
            Self::Regenerate => write!(f, "Regenerate"),
            Self::Skip => write!(f, "Skip"),
        }
    }
}
impl ::std::str::FromStr for CorrectionSummaryCorrectionType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "LocalFix" => Ok(Self::LocalFix),
            "Regenerate" => Ok(Self::Regenerate),
            "Skip" => Ok(Self::Skip),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for CorrectionSummaryCorrectionType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for CorrectionSummaryCorrectionType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for CorrectionSummaryCorrectionType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`Dag`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"convergence_points\","]
#[doc = "    \"edges\","]
#[doc = "    \"nodes\","]
#[doc = "    \"start_node_id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"convergence_points\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"format\": \"uuid\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"edges\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/Edge\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"nodes\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"additionalProperties\": {"]
#[doc = "        \"$ref\": \"#/$defs/ContentNode\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"start_node_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct Dag {
    pub convergence_points: ::std::vec::Vec<::uuid::Uuid>,
    pub edges: ::std::vec::Vec<Edge>,
    pub nodes: ::std::collections::HashMap<::std::string::String, ContentNode>,
    pub start_node_id: ::uuid::Uuid,
}
impl ::std::convert::From<&Dag> for Dag {
    fn from(value: &Dag) -> Self {
        value.clone()
    }
}
impl Dag {
    pub fn builder() -> builder::Dag {
        Default::default()
    }
}
#[doc = "`DagStructureConfig`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"branching_factor\","]
#[doc = "    \"convergence_pattern\","]
#[doc = "    \"max_depth\","]
#[doc = "    \"node_count\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"branching_factor\": {"]
#[doc = "      \"description\": \"Number of choices per decision node\","]
#[doc = "      \"default\": 2,"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 4.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"convergence_pattern\": {"]
#[doc = "      \"$ref\": \"#/$defs/ConvergencePattern\""]
#[doc = "    },"]
#[doc = "    \"convergence_point_ratio\": {"]
#[doc = "      \"description\": \"Position of convergence as ratio (0.5 = midpoint). Required for SingleConvergence, MultipleConvergence, and EndOnly. Must be omitted for PureBranching and ParallelPaths\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"max_depth\": {"]
#[doc = "      \"description\": \"Maximum depth of DAG tree\","]
#[doc = "      \"default\": 2,"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 20.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"node_count\": {"]
#[doc = "      \"description\": \"Total number of nodes in story DAG\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 100.0,"]
#[doc = "      \"minimum\": 4.0"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct DagStructureConfig {
    #[doc = "Number of choices per decision node"]
    pub branching_factor: i64,
    pub convergence_pattern: ConvergencePattern,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub convergence_point_ratio: ::std::option::Option<f64>,
    #[doc = "Maximum depth of DAG tree"]
    pub max_depth: i64,
    #[doc = "Total number of nodes in story DAG"]
    pub node_count: i64,
}
impl ::std::convert::From<&DagStructureConfig> for DagStructureConfig {
    fn from(value: &DagStructureConfig) -> Self {
        value.clone()
    }
}
impl DagStructureConfig {
    pub fn builder() -> builder::DagStructureConfig {
        Default::default()
    }
}
#[doc = "`Edge`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"choice_id\","]
#[doc = "    \"from_node_id\","]
#[doc = "    \"to_node_id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"choice_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    },"]
#[doc = "    \"from_node_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    },"]
#[doc = "    \"to_node_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    },"]
#[doc = "    \"weight\": {"]
#[doc = "      \"type\": \"number\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct Edge {
    pub choice_id: ::uuid::Uuid,
    pub from_node_id: ::uuid::Uuid,
    pub to_node_id: ::uuid::Uuid,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub weight: ::std::option::Option<f64>,
}
impl ::std::convert::From<&Edge> for Edge {
    fn from(value: &Edge) -> Self {
        value.clone()
    }
}
impl Edge {
    pub fn builder() -> builder::Edge {
        Default::default()
    }
}
#[doc = "`EducationalContent`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"educational_facts\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"learning_objective\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"topic\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"vocabulary_words\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      },"]
#[doc = "      \"maxItems\": 20"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct EducationalContent {
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub educational_facts: ::std::vec::Vec<::std::string::String>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub learning_objective: ::std::option::Option<::std::string::String>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub topic: ::std::option::Option<::std::string::String>,
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub vocabulary_words: ::std::vec::Vec<::std::string::String>,
}
impl ::std::convert::From<&EducationalContent> for EducationalContent {
    fn from(value: &EducationalContent) -> Self {
        value.clone()
    }
}
impl ::std::default::Default for EducationalContent {
    fn default() -> Self {
        Self {
            educational_facts: Default::default(),
            learning_objective: Default::default(),
            topic: Default::default(),
            vocabulary_words: Default::default(),
        }
    }
}
impl EducationalContent {
    pub fn builder() -> builder::EducationalContent {
        Default::default()
    }
}
#[doc = "`ExternalError`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"error_code\","]
#[doc = "    \"error_message\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"error_code\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"error_message\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"retry_possible\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"timestamp\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"date-time\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct ExternalError {
    pub error_code: ::std::string::String,
    pub error_message: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub retry_possible: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub timestamp: ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
}
impl ::std::convert::From<&ExternalError> for ExternalError {
    fn from(value: &ExternalError) -> Self {
        value.clone()
    }
}
impl ExternalError {
    pub fn builder() -> builder::ExternalError {
        Default::default()
    }
}
#[doc = "`ExternalGenerationRequestV1`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"age_group\","]
#[doc = "    \"language\","]
#[doc = "    \"theme\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"age_group\": {"]
#[doc = "      \"$ref\": \"#/$defs/AgeGroup\""]
#[doc = "    },"]
#[doc = "    \"language\": {"]
#[doc = "      \"$ref\": \"#/$defs/Language\""]
#[doc = "    },"]
#[doc = "    \"theme\": {"]
#[doc = "      \"description\": \"Story theme (e.g., 'underwater adventure')\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 200,"]
#[doc = "      \"minLength\": 5"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ExternalGenerationRequestV1 {
    pub age_group: AgeGroup,
    pub language: Language,
    #[doc = "Story theme (e.g., 'underwater adventure')"]
    pub theme: ExternalGenerationRequestV1Theme,
}
impl ::std::convert::From<&ExternalGenerationRequestV1> for ExternalGenerationRequestV1 {
    fn from(value: &ExternalGenerationRequestV1) -> Self {
        value.clone()
    }
}
impl ExternalGenerationRequestV1 {
    pub fn builder() -> builder::ExternalGenerationRequestV1 {
        Default::default()
    }
}
#[doc = "Story theme (e.g., 'underwater adventure')"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Story theme (e.g., 'underwater adventure')\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 200,"]
#[doc = "  \"minLength\": 5"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct ExternalGenerationRequestV1Theme(::std::string::String);
impl ::std::ops::Deref for ExternalGenerationRequestV1Theme {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<ExternalGenerationRequestV1Theme> for ::std::string::String {
    fn from(value: ExternalGenerationRequestV1Theme) -> Self {
        value.0
    }
}
impl ::std::convert::From<&ExternalGenerationRequestV1Theme> for ExternalGenerationRequestV1Theme {
    fn from(value: &ExternalGenerationRequestV1Theme) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for ExternalGenerationRequestV1Theme {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 200usize {
            return Err("longer than 200 characters".into());
        }
        if value.chars().count() < 5usize {
            return Err("shorter than 5 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for ExternalGenerationRequestV1Theme {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ExternalGenerationRequestV1Theme {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ExternalGenerationRequestV1Theme {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for ExternalGenerationRequestV1Theme {
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
#[doc = "`ExternalGenerationResponseV1`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"job_id\","]
#[doc = "    \"metadata\","]
#[doc = "    \"status\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"error\": {"]
#[doc = "      \"description\": \"Error details (if failed)\","]
#[doc = "      \"$ref\": \"#/$defs/ExternalError\""]
#[doc = "    },"]
#[doc = "    \"job_id\": {"]
#[doc = "      \"description\": \"Job identifier for tracking\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    },"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"description\": \"Generation statistics and metadata\","]
#[doc = "      \"type\": \"object\""]
#[doc = "    },"]
#[doc = "    \"status\": {"]
#[doc = "      \"description\": \"Final job status\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"completed\","]
#[doc = "        \"failed\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"trail_data\": {"]
#[doc = "      \"description\": \"Trail structure ready for DB insert (if completed)\","]
#[doc = "      \"$ref\": \"#/$defs/TrailInsertData\""]
#[doc = "    },"]
#[doc = "    \"trail_steps_data\": {"]
#[doc = "      \"description\": \"Trail steps ready for DB insert (if completed)\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/TrailStepInsertData\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct ExternalGenerationResponseV1 {
    #[doc = "Error details (if failed)"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub error: ::std::option::Option<ExternalError>,
    #[doc = "Job identifier for tracking"]
    pub job_id: ::uuid::Uuid,
    #[doc = "Generation statistics and metadata"]
    pub metadata: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    #[doc = "Final job status"]
    pub status: ExternalGenerationResponseV1Status,
    #[doc = "Trail structure ready for DB insert (if completed)"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub trail_data: ::std::option::Option<TrailInsertData>,
    #[doc = "Trail steps ready for DB insert (if completed)"]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub trail_steps_data: ::std::vec::Vec<TrailStepInsertData>,
}
impl ::std::convert::From<&ExternalGenerationResponseV1> for ExternalGenerationResponseV1 {
    fn from(value: &ExternalGenerationResponseV1) -> Self {
        value.clone()
    }
}
impl ExternalGenerationResponseV1 {
    pub fn builder() -> builder::ExternalGenerationResponseV1 {
        Default::default()
    }
}
#[doc = "Final job status"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Final job status\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"completed\","]
#[doc = "    \"failed\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum ExternalGenerationResponseV1Status {
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
}
impl ::std::convert::From<&Self> for ExternalGenerationResponseV1Status {
    fn from(value: &ExternalGenerationResponseV1Status) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for ExternalGenerationResponseV1Status {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}
impl ::std::str::FromStr for ExternalGenerationResponseV1Status {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ExternalGenerationResponseV1Status {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ExternalGenerationResponseV1Status {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ExternalGenerationResponseV1Status {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`ExternalJobStatus`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"job_id\","]
#[doc = "    \"progress_percentage\","]
#[doc = "    \"status\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"current_phase\": {"]
#[doc = "      \"description\": \"Human-readable phase description\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"estimated_completion_seconds\": {"]
#[doc = "      \"description\": \"Estimated time to completion\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"job_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    },"]
#[doc = "    \"progress_percentage\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 100.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"status\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"pending\","]
#[doc = "        \"in_progress\","]
#[doc = "        \"completed\","]
#[doc = "        \"failed\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct ExternalJobStatus {
    #[doc = "Human-readable phase description"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub current_phase: ::std::option::Option<::std::string::String>,
    #[doc = "Estimated time to completion"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub estimated_completion_seconds: ::std::option::Option<u64>,
    pub job_id: ::uuid::Uuid,
    pub progress_percentage: i64,
    pub status: ExternalJobStatusStatus,
}
impl ::std::convert::From<&ExternalJobStatus> for ExternalJobStatus {
    fn from(value: &ExternalJobStatus) -> Self {
        value.clone()
    }
}
impl ExternalJobStatus {
    pub fn builder() -> builder::ExternalJobStatus {
        Default::default()
    }
}
#[doc = "`ExternalJobStatusStatus`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"pending\","]
#[doc = "    \"in_progress\","]
#[doc = "    \"completed\","]
#[doc = "    \"failed\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum ExternalJobStatusStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
}
impl ::std::convert::From<&Self> for ExternalJobStatusStatus {
    fn from(value: &ExternalJobStatusStatus) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for ExternalJobStatusStatus {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Pending => write!(f, "pending"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}
impl ::std::str::FromStr for ExternalJobStatusStatus {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "pending" => Ok(Self::Pending),
            "in_progress" => Ok(Self::InProgress),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ExternalJobStatusStatus {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ExternalJobStatusStatus {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ExternalJobStatusStatus {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`GatewayMappingConfig`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"defaults\","]
#[doc = "    \"validation\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"defaults\": {"]
#[doc = "      \"$ref\": \"#/$defs/MappingDefaults\""]
#[doc = "    },"]
#[doc = "    \"validation\": {"]
#[doc = "      \"$ref\": \"#/$defs/MappingValidation\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct GatewayMappingConfig {
    pub defaults: MappingDefaults,
    pub validation: MappingValidation,
}
impl ::std::convert::From<&GatewayMappingConfig> for GatewayMappingConfig {
    fn from(value: &GatewayMappingConfig) -> Self {
        value.clone()
    }
}
impl GatewayMappingConfig {
    pub fn builder() -> builder::GatewayMappingConfig {
        Default::default()
    }
}
#[doc = "`GenerationError`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"error_code\","]
#[doc = "    \"error_message\","]
#[doc = "    \"retry_possible\","]
#[doc = "    \"timestamp\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"error_code\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"error_message\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"retry_possible\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"timestamp\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"date-time\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct GenerationError {
    pub error_code: ::std::string::String,
    pub error_message: ::std::string::String,
    pub retry_possible: bool,
    pub timestamp: ::chrono::DateTime<::chrono::offset::Utc>,
}
impl ::std::convert::From<&GenerationError> for GenerationError {
    fn from(value: &GenerationError) -> Self {
        value.clone()
    }
}
impl GenerationError {
    pub fn builder() -> builder::GenerationError {
        Default::default()
    }
}
#[doc = "`GenerationMetadata`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"ai_model_version\","]
#[doc = "    \"generated_at\","]
#[doc = "    \"generation_duration_seconds\","]
#[doc = "    \"negotiation_rounds_executed\","]
#[doc = "    \"orchestrator_version\","]
#[doc = "    \"resolved_node_count\","]
#[doc = "    \"total_word_count\","]
#[doc = "    \"validation_pass_rate\","]
#[doc = "    \"validation_rounds\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"ai_model_version\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"corrections_applied\": {"]
#[doc = "      \"description\": \"List of all correction attempts during negotiation\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/CorrectionSummary\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"generated_at\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"date-time\""]
#[doc = "    },"]
#[doc = "    \"generation_duration_seconds\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"negotiation_rounds_executed\": {"]
#[doc = "      \"description\": \"Number of negotiation rounds executed after initial validation\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 10.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"orchestrator_version\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"resolved_node_count\": {"]
#[doc = "      \"description\": \"Actual node count used after resolving preset/explicit values (Priority: explicit > preset > defaults)\","]
#[doc = "      \"type\": \"integer\""]
#[doc = "    },"]
#[doc = "    \"total_word_count\": {"]
#[doc = "      \"type\": \"integer\""]
#[doc = "    },"]
#[doc = "    \"unresolved_validation_issues\": {"]
#[doc = "      \"description\": \"Issues remaining after max negotiation rounds exceeded (empty if all resolved)\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/ValidationIssueSummary\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"validation_pass_rate\": {"]
#[doc = "      \"description\": \"Percentage of nodes that passed validation (nodes_passed / total_nodes)\","]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"validation_rounds\": {"]
#[doc = "      \"description\": \"Total number of validation rounds executed (includes initial + negotiation rounds)\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 10.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct GenerationMetadata {
    pub ai_model_version: ::std::string::String,
    #[doc = "List of all correction attempts during negotiation"]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub corrections_applied: ::std::vec::Vec<CorrectionSummary>,
    pub generated_at: ::chrono::DateTime<::chrono::offset::Utc>,
    pub generation_duration_seconds: u64,
    #[doc = "Number of negotiation rounds executed after initial validation"]
    pub negotiation_rounds_executed: i64,
    pub orchestrator_version: ::std::string::String,
    #[doc = "Actual node count used after resolving preset/explicit values (Priority: explicit > preset > defaults)"]
    pub resolved_node_count: i64,
    pub total_word_count: i64,
    #[doc = "Issues remaining after max negotiation rounds exceeded (empty if all resolved)"]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub unresolved_validation_issues: ::std::vec::Vec<ValidationIssueSummary>,
    pub validation_pass_rate: f64,
    #[doc = "Total number of validation rounds executed (includes initial + negotiation rounds)"]
    pub validation_rounds: i64,
}
impl ::std::convert::From<&GenerationMetadata> for GenerationMetadata {
    fn from(value: &GenerationMetadata) -> Self {
        value.clone()
    }
}
impl GenerationMetadata {
    pub fn builder() -> builder::GenerationMetadata {
        Default::default()
    }
}
#[doc = "Current phase of generation pipeline"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Current phase of generation pipeline\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"PromptGeneration\","]
#[doc = "    \"Structure\","]
#[doc = "    \"Generation\","]
#[doc = "    \"Validation\","]
#[doc = "    \"Assembly\","]
#[doc = "    \"Complete\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum GenerationPhase {
    PromptGeneration,
    Structure,
    Generation,
    Validation,
    Assembly,
    Complete,
}
impl ::std::convert::From<&Self> for GenerationPhase {
    fn from(value: &GenerationPhase) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for GenerationPhase {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::PromptGeneration => write!(f, "PromptGeneration"),
            Self::Structure => write!(f, "Structure"),
            Self::Generation => write!(f, "Generation"),
            Self::Validation => write!(f, "Validation"),
            Self::Assembly => write!(f, "Assembly"),
            Self::Complete => write!(f, "Complete"),
        }
    }
}
impl ::std::str::FromStr for GenerationPhase {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "PromptGeneration" => Ok(Self::PromptGeneration),
            "Structure" => Ok(Self::Structure),
            "Generation" => Ok(Self::Generation),
            "Validation" => Ok(Self::Validation),
            "Assembly" => Ok(Self::Assembly),
            "Complete" => Ok(Self::Complete),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for GenerationPhase {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for GenerationPhase {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for GenerationPhase {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`GenerationRequest`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"age_group\","]
#[doc = "    \"language\","]
#[doc = "    \"tenant_id\","]
#[doc = "    \"theme\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"age_group\": {"]
#[doc = "      \"$ref\": \"#/$defs/AgeGroup\""]
#[doc = "    },"]
#[doc = "    \"author_id\": {"]
#[doc = "      \"description\": \"TaleTrails user GID from JWT\","]
#[doc = "      \"type\": ["]
#[doc = "        \"string\","]
#[doc = "        \"null\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"dag_config\": {"]
#[doc = "      \"description\": \"Custom DAG configuration (Tier 2: Advanced). Mutually exclusive with story_structure\","]
#[doc = "      \"$ref\": \"#/$defs/DagStructureConfig\""]
#[doc = "    },"]
#[doc = "    \"educational_goals\": {"]
#[doc = "      \"default\": [],"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      },"]
#[doc = "      \"maxItems\": 10"]
#[doc = "    },"]
#[doc = "    \"language\": {"]
#[doc = "      \"$ref\": \"#/$defs/Language\""]
#[doc = "    },"]
#[doc = "    \"node_count\": {"]
#[doc = "      \"description\": \"Must be even number for convergence calculation\","]
#[doc = "      \"default\": 30,"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 255.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"prompt_packages\": {"]
#[doc = "      \"description\": \"Cached prompts from prompt-helper (Phase 0.5)\","]
#[doc = "      \"type\": ["]
#[doc = "        \"object\","]
#[doc = "        \"null\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"required_elements\": {"]
#[doc = "      \"default\": [],"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      },"]
#[doc = "      \"maxItems\": 5"]
#[doc = "    },"]
#[doc = "    \"story_structure\": {"]
#[doc = "      \"description\": \"Predefined story structure preset (Tier 1: Simple). Mutually exclusive with dag_config. Takes priority if both provided\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"guided\","]
#[doc = "        \"adventure\","]
#[doc = "        \"epic\","]
#[doc = "        \"choose_your_path\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"tags\": {"]
#[doc = "      \"default\": [],"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"maxLength\": 50"]
#[doc = "      },"]
#[doc = "      \"maxItems\": 20"]
#[doc = "    },"]
#[doc = "    \"tenant_id\": {"]
#[doc = "      \"description\": \"TaleTrails tenant GID (extracted by Qollective)\","]
#[doc = "      \"type\": ["]
#[doc = "        \"string\","]
#[doc = "        \"null\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"theme\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 200,"]
#[doc = "      \"minLength\": 5"]
#[doc = "    },"]
#[doc = "    \"validation_policy\": {"]
#[doc = "      \"description\": \"Validation policy for content generation (optional)\","]
#[doc = "      \"$ref\": \"#/$defs/ValidationPolicy\""]
#[doc = "    },"]
#[doc = "    \"vocabulary_level\": {"]
#[doc = "      \"default\": \"basic\","]
#[doc = "      \"$ref\": \"#/$defs/VocabularyLevel\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct GenerationRequest {
    pub age_group: AgeGroup,
    #[doc = "TaleTrails user GID from JWT"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub author_id: ::std::option::Option<::std::string::String>,
    #[doc = "Custom DAG configuration (Tier 2: Advanced). Mutually exclusive with story_structure"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub dag_config: ::std::option::Option<DagStructureConfig>,
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub educational_goals: ::std::vec::Vec<::std::string::String>,
    pub language: Language,
    #[doc = "Must be even number for convergence calculation"]
    #[serde(default = "defaults::default_u64::<u8, 30>")]
    pub node_count: u8,
    #[doc = "Cached prompts from prompt-helper (Phase 0.5)"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub prompt_packages:
        ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub required_elements: ::std::vec::Vec<::std::string::String>,
    #[doc = "Predefined story structure preset (Tier 1: Simple). Mutually exclusive with dag_config. Takes priority if both provided"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub story_structure: ::std::option::Option<GenerationRequestStoryStructure>,
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub tags: ::std::vec::Vec<GenerationRequestTagsItem>,
    #[doc = "TaleTrails tenant GID (extracted by Qollective)"]
    pub tenant_id: ::std::option::Option<::std::string::String>,
    pub theme: GenerationRequestTheme,
    #[doc = "Validation policy for content generation (optional)"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub validation_policy: ::std::option::Option<ValidationPolicy>,
    #[serde(default = "defaults::generation_request_vocabulary_level")]
    pub vocabulary_level: VocabularyLevel,
}
impl ::std::convert::From<&GenerationRequest> for GenerationRequest {
    fn from(value: &GenerationRequest) -> Self {
        value.clone()
    }
}
impl GenerationRequest {
    pub fn builder() -> builder::GenerationRequest {
        Default::default()
    }
}
#[doc = "Predefined story structure preset (Tier 1: Simple). Mutually exclusive with dag_config. Takes priority if both provided"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Predefined story structure preset (Tier 1: Simple). Mutually exclusive with dag_config. Takes priority if both provided\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"guided\","]
#[doc = "    \"adventure\","]
#[doc = "    \"epic\","]
#[doc = "    \"choose_your_path\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum GenerationRequestStoryStructure {
    #[serde(rename = "guided")]
    Guided,
    #[serde(rename = "adventure")]
    Adventure,
    #[serde(rename = "epic")]
    Epic,
    #[serde(rename = "choose_your_path")]
    ChooseYourPath,
}
impl ::std::convert::From<&Self> for GenerationRequestStoryStructure {
    fn from(value: &GenerationRequestStoryStructure) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for GenerationRequestStoryStructure {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Guided => write!(f, "guided"),
            Self::Adventure => write!(f, "adventure"),
            Self::Epic => write!(f, "epic"),
            Self::ChooseYourPath => write!(f, "choose_your_path"),
        }
    }
}
impl ::std::str::FromStr for GenerationRequestStoryStructure {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "guided" => Ok(Self::Guided),
            "adventure" => Ok(Self::Adventure),
            "epic" => Ok(Self::Epic),
            "choose_your_path" => Ok(Self::ChooseYourPath),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for GenerationRequestStoryStructure {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for GenerationRequestStoryStructure {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for GenerationRequestStoryStructure {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`GenerationRequestTagsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 50"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct GenerationRequestTagsItem(::std::string::String);
impl ::std::ops::Deref for GenerationRequestTagsItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<GenerationRequestTagsItem> for ::std::string::String {
    fn from(value: GenerationRequestTagsItem) -> Self {
        value.0
    }
}
impl ::std::convert::From<&GenerationRequestTagsItem> for GenerationRequestTagsItem {
    fn from(value: &GenerationRequestTagsItem) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for GenerationRequestTagsItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 50usize {
            return Err("longer than 50 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for GenerationRequestTagsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for GenerationRequestTagsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for GenerationRequestTagsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for GenerationRequestTagsItem {
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
#[doc = "`GenerationRequestTheme`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 200,"]
#[doc = "  \"minLength\": 5"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct GenerationRequestTheme(::std::string::String);
impl ::std::ops::Deref for GenerationRequestTheme {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<GenerationRequestTheme> for ::std::string::String {
    fn from(value: GenerationRequestTheme) -> Self {
        value.0
    }
}
impl ::std::convert::From<&GenerationRequestTheme> for GenerationRequestTheme {
    fn from(value: &GenerationRequestTheme) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for GenerationRequestTheme {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 200usize {
            return Err("longer than 200 characters".into());
        }
        if value.chars().count() < 5usize {
            return Err("shorter than 5 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for GenerationRequestTheme {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for GenerationRequestTheme {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for GenerationRequestTheme {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for GenerationRequestTheme {
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
#[doc = "`GenerationResponse`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"progress_percentage\","]
#[doc = "    \"request_id\","]
#[doc = "    \"status\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"errors\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/GenerationError\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"execution_trace\": {"]
#[doc = "      \"description\": \"Complete execution trace with all service invocations and timing\","]
#[doc = "      \"$ref\": \"#/$defs/PipelineExecutionTrace\""]
#[doc = "    },"]
#[doc = "    \"generation_metadata\": {"]
#[doc = "      \"$ref\": \"#/$defs/GenerationMetadata\""]
#[doc = "    },"]
#[doc = "    \"progress_percentage\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 100.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"prompt_generation_metadata\": {"]
#[doc = "      \"$ref\": \"#/$defs/PromptGenerationSummary\""]
#[doc = "    },"]
#[doc = "    \"request_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    },"]
#[doc = "    \"status\": {"]
#[doc = "      \"$ref\": \"#/$defs/GenerationStatus\""]
#[doc = "    },"]
#[doc = "    \"trail\": {"]
#[doc = "      \"$ref\": \"#/$defs/Trail\""]
#[doc = "    },"]
#[doc = "    \"trail_steps\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/TrailStep\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct GenerationResponse {
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub errors: ::std::vec::Vec<GenerationError>,
    #[doc = "Complete execution trace with all service invocations and timing"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub execution_trace: ::std::option::Option<PipelineExecutionTrace>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub generation_metadata: ::std::option::Option<GenerationMetadata>,
    pub progress_percentage: i64,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub prompt_generation_metadata: ::std::option::Option<PromptGenerationSummary>,
    pub request_id: ::uuid::Uuid,
    pub status: GenerationStatus,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub trail: ::std::option::Option<Trail>,
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub trail_steps: ::std::vec::Vec<TrailStep>,
}
impl ::std::convert::From<&GenerationResponse> for GenerationResponse {
    fn from(value: &GenerationResponse) -> Self {
        value.clone()
    }
}
impl GenerationResponse {
    pub fn builder() -> builder::GenerationResponse {
        Default::default()
    }
}
#[doc = "Status of content generation"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Status of content generation\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"pending\","]
#[doc = "    \"in_progress\","]
#[doc = "    \"completed\","]
#[doc = "    \"failed\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum GenerationStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
}
impl ::std::convert::From<&Self> for GenerationStatus {
    fn from(value: &GenerationStatus) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for GenerationStatus {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Pending => write!(f, "pending"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}
impl ::std::str::FromStr for GenerationStatus {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "pending" => Ok(Self::Pending),
            "in_progress" => Ok(Self::InProgress),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for GenerationStatus {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for GenerationStatus {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for GenerationStatus {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Content language: de (German), en (English)"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Content language: de (German), en (English)\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"de\","]
#[doc = "    \"en\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum Language {
    #[serde(rename = "de")]
    De,
    #[serde(rename = "en")]
    En,
}
impl ::std::convert::From<&Self> for Language {
    fn from(value: &Language) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for Language {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::De => write!(f, "de"),
            Self::En => write!(f, "en"),
        }
    }
}
impl ::std::str::FromStr for Language {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "de" => Ok(Self::De),
            "en" => Ok(Self::En),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for Language {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Language {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Language {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`LlmConfig`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"frequency_penalty\","]
#[doc = "    \"max_tokens\","]
#[doc = "    \"presence_penalty\","]
#[doc = "    \"stop_sequences\","]
#[doc = "    \"temperature\","]
#[doc = "    \"top_p\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"frequency_penalty\": {"]
#[doc = "      \"default\": 0.0,"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 2.0,"]
#[doc = "      \"minimum\": -2.0"]
#[doc = "    },"]
#[doc = "    \"max_tokens\": {"]
#[doc = "      \"default\": 2000,"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 32000.0,"]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"presence_penalty\": {"]
#[doc = "      \"default\": 0.0,"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 2.0,"]
#[doc = "      \"minimum\": -2.0"]
#[doc = "    },"]
#[doc = "    \"stop_sequences\": {"]
#[doc = "      \"default\": [],"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"temperature\": {"]
#[doc = "      \"default\": 0.7,"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 2.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"top_p\": {"]
#[doc = "      \"default\": 0.9,"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct LlmConfig {
    pub frequency_penalty: f64,
    pub max_tokens: ::std::num::NonZeroU64,
    pub presence_penalty: f64,
    pub stop_sequences: ::std::vec::Vec<::std::string::String>,
    pub temperature: f64,
    pub top_p: f64,
}
impl ::std::convert::From<&LlmConfig> for LlmConfig {
    fn from(value: &LlmConfig) -> Self {
        value.clone()
    }
}
impl LlmConfig {
    pub fn builder() -> builder::LlmConfig {
        Default::default()
    }
}
#[doc = "`MappingDefaults`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"educational_goals\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"additionalProperties\": {"]
#[doc = "        \"type\": \"array\","]
#[doc = "        \"items\": {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        }"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"node_count_12_14\": {"]
#[doc = "      \"default\": 16,"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 50.0,"]
#[doc = "      \"minimum\": 6.0"]
#[doc = "    },"]
#[doc = "    \"node_count_15_17\": {"]
#[doc = "      \"default\": 24,"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 60.0,"]
#[doc = "      \"minimum\": 6.0"]
#[doc = "    },"]
#[doc = "    \"node_count_18_plus\": {"]
#[doc = "      \"default\": 30,"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 90.0,"]
#[doc = "      \"minimum\": 6.0"]
#[doc = "    },"]
#[doc = "    \"node_count_6_8\": {"]
#[doc = "      \"default\": 12,"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 20.0,"]
#[doc = "      \"minimum\": 6.0"]
#[doc = "    },"]
#[doc = "    \"node_count_9_11\": {"]
#[doc = "      \"default\": 12,"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 30.0,"]
#[doc = "      \"minimum\": 6.0"]
#[doc = "    },"]
#[doc = "    \"required_elements\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"additionalProperties\": {"]
#[doc = "        \"type\": \"array\","]
#[doc = "        \"items\": {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        }"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"vocabulary_level_12_14\": {"]
#[doc = "      \"default\": \"intermediate\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"vocabulary_level_15_17\": {"]
#[doc = "      \"default\": \"intermediate\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"vocabulary_level_18_plus\": {"]
#[doc = "      \"default\": \"advanced\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"vocabulary_level_6_8\": {"]
#[doc = "      \"default\": \"basic\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"vocabulary_level_9_11\": {"]
#[doc = "      \"default\": \"basic\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct MappingDefaults {
    #[serde(
        default,
        skip_serializing_if = ":: std :: collections :: HashMap::is_empty"
    )]
    pub educational_goals:
        ::std::collections::HashMap<::std::string::String, ::std::vec::Vec<::std::string::String>>,
    #[serde(default = "defaults::default_u64::<i64, 16>")]
    pub node_count_12_14: i64,
    #[serde(default = "defaults::default_u64::<i64, 24>")]
    pub node_count_15_17: i64,
    #[serde(default = "defaults::default_u64::<i64, 30>")]
    pub node_count_18_plus: i64,
    #[serde(default = "defaults::default_u64::<i64, 12>")]
    pub node_count_6_8: i64,
    #[serde(default = "defaults::default_u64::<i64, 12>")]
    pub node_count_9_11: i64,
    #[serde(
        default,
        skip_serializing_if = ":: std :: collections :: HashMap::is_empty"
    )]
    pub required_elements:
        ::std::collections::HashMap<::std::string::String, ::std::vec::Vec<::std::string::String>>,
    #[serde(default = "defaults::mapping_defaults_vocabulary_level_12_14")]
    pub vocabulary_level_12_14: ::std::string::String,
    #[serde(default = "defaults::mapping_defaults_vocabulary_level_15_17")]
    pub vocabulary_level_15_17: ::std::string::String,
    #[serde(default = "defaults::mapping_defaults_vocabulary_level_18_plus")]
    pub vocabulary_level_18_plus: ::std::string::String,
    #[serde(default = "defaults::mapping_defaults_vocabulary_level_6_8")]
    pub vocabulary_level_6_8: ::std::string::String,
    #[serde(default = "defaults::mapping_defaults_vocabulary_level_9_11")]
    pub vocabulary_level_9_11: ::std::string::String,
}
impl ::std::convert::From<&MappingDefaults> for MappingDefaults {
    fn from(value: &MappingDefaults) -> Self {
        value.clone()
    }
}
impl ::std::default::Default for MappingDefaults {
    fn default() -> Self {
        Self {
            educational_goals: Default::default(),
            node_count_12_14: defaults::default_u64::<i64, 16>(),
            node_count_15_17: defaults::default_u64::<i64, 24>(),
            node_count_18_plus: defaults::default_u64::<i64, 30>(),
            node_count_6_8: defaults::default_u64::<i64, 12>(),
            node_count_9_11: defaults::default_u64::<i64, 12>(),
            required_elements: Default::default(),
            vocabulary_level_12_14: defaults::mapping_defaults_vocabulary_level_12_14(),
            vocabulary_level_15_17: defaults::mapping_defaults_vocabulary_level_15_17(),
            vocabulary_level_18_plus: defaults::mapping_defaults_vocabulary_level_18_plus(),
            vocabulary_level_6_8: defaults::mapping_defaults_vocabulary_level_6_8(),
            vocabulary_level_9_11: defaults::mapping_defaults_vocabulary_level_9_11(),
        }
    }
}
impl MappingDefaults {
    pub fn builder() -> builder::MappingDefaults {
        Default::default()
    }
}
#[doc = "`MappingError`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"error_type\","]
#[doc = "    \"message\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"error_type\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"ValidationError\","]
#[doc = "        \"ConfigurationError\","]
#[doc = "        \"InternalResponseError\","]
#[doc = "        \"FormatConversionError\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"message\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct MappingError {
    pub error_type: MappingErrorErrorType,
    pub message: ::std::string::String,
}
impl ::std::convert::From<&MappingError> for MappingError {
    fn from(value: &MappingError) -> Self {
        value.clone()
    }
}
impl MappingError {
    pub fn builder() -> builder::MappingError {
        Default::default()
    }
}
#[doc = "`MappingErrorErrorType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"ValidationError\","]
#[doc = "    \"ConfigurationError\","]
#[doc = "    \"InternalResponseError\","]
#[doc = "    \"FormatConversionError\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum MappingErrorErrorType {
    ValidationError,
    ConfigurationError,
    InternalResponseError,
    FormatConversionError,
}
impl ::std::convert::From<&Self> for MappingErrorErrorType {
    fn from(value: &MappingErrorErrorType) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for MappingErrorErrorType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::ValidationError => write!(f, "ValidationError"),
            Self::ConfigurationError => write!(f, "ConfigurationError"),
            Self::InternalResponseError => write!(f, "InternalResponseError"),
            Self::FormatConversionError => write!(f, "FormatConversionError"),
        }
    }
}
impl ::std::str::FromStr for MappingErrorErrorType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "ValidationError" => Ok(Self::ValidationError),
            "ConfigurationError" => Ok(Self::ConfigurationError),
            "InternalResponseError" => Ok(Self::InternalResponseError),
            "FormatConversionError" => Ok(Self::FormatConversionError),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for MappingErrorErrorType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for MappingErrorErrorType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for MappingErrorErrorType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`MappingValidation`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"allowed_age_groups\": {"]
#[doc = "      \"default\": ["]
#[doc = "        \"6-8\","]
#[doc = "        \"9-11\","]
#[doc = "        \"12-14\","]
#[doc = "        \"15-17\","]
#[doc = "        \"+18\""]
#[doc = "      ],"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"allowed_languages\": {"]
#[doc = "      \"default\": ["]
#[doc = "        \"de\","]
#[doc = "        \"en\""]
#[doc = "      ],"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"max_theme_length\": {"]
#[doc = "      \"default\": 200,"]
#[doc = "      \"type\": \"integer\""]
#[doc = "    },"]
#[doc = "    \"min_theme_length\": {"]
#[doc = "      \"default\": 5,"]
#[doc = "      \"type\": \"integer\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct MappingValidation {
    #[serde(default = "defaults::mapping_validation_allowed_age_groups")]
    pub allowed_age_groups: ::std::vec::Vec<::std::string::String>,
    #[serde(default = "defaults::mapping_validation_allowed_languages")]
    pub allowed_languages: ::std::vec::Vec<::std::string::String>,
    #[serde(default = "defaults::default_u64::<i64, 200>")]
    pub max_theme_length: i64,
    #[serde(default = "defaults::default_u64::<i64, 5>")]
    pub min_theme_length: i64,
}
impl ::std::convert::From<&MappingValidation> for MappingValidation {
    fn from(value: &MappingValidation) -> Self {
        value.clone()
    }
}
impl ::std::default::Default for MappingValidation {
    fn default() -> Self {
        Self {
            allowed_age_groups: defaults::mapping_validation_allowed_age_groups(),
            allowed_languages: defaults::mapping_validation_allowed_languages(),
            max_theme_length: defaults::default_u64::<i64, 200>(),
            min_theme_length: defaults::default_u64::<i64, 5>(),
        }
    }
}
impl MappingValidation {
    pub fn builder() -> builder::MappingValidation {
        Default::default()
    }
}
#[doc = "MCP service identifier"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"MCP service identifier\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"StoryGenerator\","]
#[doc = "    \"QualityControl\","]
#[doc = "    \"ConstraintEnforcer\","]
#[doc = "    \"PromptHelper\","]
#[doc = "    \"Orchestrator\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum McpServiceType {
    StoryGenerator,
    QualityControl,
    ConstraintEnforcer,
    PromptHelper,
    Orchestrator,
}
impl ::std::convert::From<&Self> for McpServiceType {
    fn from(value: &McpServiceType) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for McpServiceType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::StoryGenerator => write!(f, "StoryGenerator"),
            Self::QualityControl => write!(f, "QualityControl"),
            Self::ConstraintEnforcer => write!(f, "ConstraintEnforcer"),
            Self::PromptHelper => write!(f, "PromptHelper"),
            Self::Orchestrator => write!(f, "Orchestrator"),
        }
    }
}
impl ::std::str::FromStr for McpServiceType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "StoryGenerator" => Ok(Self::StoryGenerator),
            "QualityControl" => Ok(Self::QualityControl),
            "ConstraintEnforcer" => Ok(Self::ConstraintEnforcer),
            "PromptHelper" => Ok(Self::PromptHelper),
            "Orchestrator" => Ok(Self::Orchestrator),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for McpServiceType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for McpServiceType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for McpServiceType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`NodeContext`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"incoming_edges\","]
#[doc = "    \"is_convergence_point\","]
#[doc = "    \"node_id\","]
#[doc = "    \"node_position\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"incoming_edges\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"is_convergence_point\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"node_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    },"]
#[doc = "    \"node_position\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"previous_content\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct NodeContext {
    pub incoming_edges: u64,
    pub is_convergence_point: bool,
    pub node_id: ::uuid::Uuid,
    pub node_position: ::std::num::NonZeroU64,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub previous_content: ::std::option::Option<::std::string::String>,
}
impl ::std::convert::From<&NodeContext> for NodeContext {
    fn from(value: &NodeContext) -> Self {
        value.clone()
    }
}
impl NodeContext {
    pub fn builder() -> builder::NodeContext {
        Default::default()
    }
}
#[doc = "`PipelineExecutionTrace`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"phases_completed\","]
#[doc = "    \"request_id\","]
#[doc = "    \"service_invocations\","]
#[doc = "    \"total_duration_ms\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"events_published\": {"]
#[doc = "      \"description\": \"All pipeline events published to NATS (optional, for debugging)\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"object\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"phases_completed\": {"]
#[doc = "      \"description\": \"List of pipeline phases completed in order\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/GenerationPhase\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"request_id\": {"]
#[doc = "      \"description\": \"Request ID for this execution\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    },"]
#[doc = "    \"service_invocations\": {"]
#[doc = "      \"description\": \"Complete history of all MCP service calls with timing\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/ServiceInvocation\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"total_duration_ms\": {"]
#[doc = "      \"description\": \"Total pipeline execution time in milliseconds\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct PipelineExecutionTrace {
    #[doc = "All pipeline events published to NATS (optional, for debugging)"]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub events_published:
        ::std::vec::Vec<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
    #[doc = "List of pipeline phases completed in order"]
    pub phases_completed: ::std::vec::Vec<GenerationPhase>,
    #[doc = "Request ID for this execution"]
    pub request_id: ::uuid::Uuid,
    #[doc = "Complete history of all MCP service calls with timing"]
    pub service_invocations: ::std::vec::Vec<ServiceInvocation>,
    #[doc = "Total pipeline execution time in milliseconds"]
    pub total_duration_ms: u64,
}
impl ::std::convert::From<&PipelineExecutionTrace> for PipelineExecutionTrace {
    fn from(value: &PipelineExecutionTrace) -> Self {
        value.clone()
    }
}
impl PipelineExecutionTrace {
    pub fn builder() -> builder::PipelineExecutionTrace {
        Default::default()
    }
}
#[doc = "Method used to generate prompts"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Method used to generate prompts\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"LLMGenerated\","]
#[doc = "    \"TemplateFallback\","]
#[doc = "    \"Cached\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum PromptGenerationMethod {
    #[serde(rename = "LLMGenerated")]
    LlmGenerated,
    TemplateFallback,
    Cached,
}
impl ::std::convert::From<&Self> for PromptGenerationMethod {
    fn from(value: &PromptGenerationMethod) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for PromptGenerationMethod {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::LlmGenerated => write!(f, "LLMGenerated"),
            Self::TemplateFallback => write!(f, "TemplateFallback"),
            Self::Cached => write!(f, "Cached"),
        }
    }
}
impl ::std::str::FromStr for PromptGenerationMethod {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "LLMGenerated" => Ok(Self::LlmGenerated),
            "TemplateFallback" => Ok(Self::TemplateFallback),
            "Cached" => Ok(Self::Cached),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for PromptGenerationMethod {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for PromptGenerationMethod {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for PromptGenerationMethod {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`PromptGenerationRequest`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"generation_request\","]
#[doc = "    \"service_target\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"batch_info\": {"]
#[doc = "      \"$ref\": \"#/$defs/BatchInfo\""]
#[doc = "    },"]
#[doc = "    \"generation_request\": {"]
#[doc = "      \"$ref\": \"#/$defs/GenerationRequest\""]
#[doc = "    },"]
#[doc = "    \"node_context\": {"]
#[doc = "      \"$ref\": \"#/$defs/NodeContext\""]
#[doc = "    },"]
#[doc = "    \"service_target\": {"]
#[doc = "      \"$ref\": \"#/$defs/MCPServiceType\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct PromptGenerationRequest {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub batch_info: ::std::option::Option<BatchInfo>,
    pub generation_request: GenerationRequest,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub node_context: ::std::option::Option<NodeContext>,
    pub service_target: McpServiceType,
}
impl ::std::convert::From<&PromptGenerationRequest> for PromptGenerationRequest {
    fn from(value: &PromptGenerationRequest) -> Self {
        value.clone()
    }
}
impl PromptGenerationRequest {
    pub fn builder() -> builder::PromptGenerationRequest {
        Default::default()
    }
}
#[doc = "`PromptGenerationSummary`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"fallback_count\","]
#[doc = "    \"llm_generated_count\","]
#[doc = "    \"prompt_generation_duration_ms\","]
#[doc = "    \"prompts_generated_at\","]
#[doc = "    \"prompts_used\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"fallback_count\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"llm_generated_count\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"prompt_generation_duration_ms\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"prompts_generated_at\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"date-time\""]
#[doc = "    },"]
#[doc = "    \"prompts_used\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"additionalProperties\": {"]
#[doc = "        \"$ref\": \"#/$defs/PromptPackage\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct PromptGenerationSummary {
    pub fallback_count: u64,
    pub llm_generated_count: u64,
    pub prompt_generation_duration_ms: u64,
    pub prompts_generated_at: ::chrono::DateTime<::chrono::offset::Utc>,
    pub prompts_used: ::std::collections::HashMap<::std::string::String, PromptPackage>,
}
impl ::std::convert::From<&PromptGenerationSummary> for PromptGenerationSummary {
    fn from(value: &PromptGenerationSummary) -> Self {
        value.clone()
    }
}
impl PromptGenerationSummary {
    pub fn builder() -> builder::PromptGenerationSummary {
        Default::default()
    }
}
#[doc = "`PromptMetadata`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"age_group_context\","]
#[doc = "    \"generated_at\","]
#[doc = "    \"generation_method\","]
#[doc = "    \"language_context\","]
#[doc = "    \"service_target\","]
#[doc = "    \"template_version\","]
#[doc = "    \"theme_context\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"age_group_context\": {"]
#[doc = "      \"$ref\": \"#/$defs/AgeGroup\""]
#[doc = "    },"]
#[doc = "    \"generated_at\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"date-time\""]
#[doc = "    },"]
#[doc = "    \"generation_method\": {"]
#[doc = "      \"$ref\": \"#/$defs/PromptGenerationMethod\""]
#[doc = "    },"]
#[doc = "    \"language_context\": {"]
#[doc = "      \"$ref\": \"#/$defs/Language\""]
#[doc = "    },"]
#[doc = "    \"service_target\": {"]
#[doc = "      \"$ref\": \"#/$defs/MCPServiceType\""]
#[doc = "    },"]
#[doc = "    \"template_version\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"theme_context\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct PromptMetadata {
    pub age_group_context: AgeGroup,
    pub generated_at: ::chrono::DateTime<::chrono::offset::Utc>,
    pub generation_method: PromptGenerationMethod,
    pub language_context: Language,
    pub service_target: McpServiceType,
    pub template_version: ::std::string::String,
    pub theme_context: ::std::string::String,
}
impl ::std::convert::From<&PromptMetadata> for PromptMetadata {
    fn from(value: &PromptMetadata) -> Self {
        value.clone()
    }
}
impl PromptMetadata {
    pub fn builder() -> builder::PromptMetadata {
        Default::default()
    }
}
#[doc = "`PromptPackage`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"fallback_used\","]
#[doc = "    \"language\","]
#[doc = "    \"llm_config\","]
#[doc = "    \"llm_model\","]
#[doc = "    \"prompt_metadata\","]
#[doc = "    \"system_prompt\","]
#[doc = "    \"user_prompt\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"fallback_used\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"language\": {"]
#[doc = "      \"$ref\": \"#/$defs/Language\""]
#[doc = "    },"]
#[doc = "    \"llm_config\": {"]
#[doc = "      \"$ref\": \"#/$defs/LLMConfig\""]
#[doc = "    },"]
#[doc = "    \"llm_model\": {"]
#[doc = "      \"description\": \"LLM model identifier\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"prompt_metadata\": {"]
#[doc = "      \"$ref\": \"#/$defs/PromptMetadata\""]
#[doc = "    },"]
#[doc = "    \"system_prompt\": {"]
#[doc = "      \"description\": \"LLM system instruction\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"user_prompt\": {"]
#[doc = "      \"description\": \"LLM user message with context\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct PromptPackage {
    pub fallback_used: bool,
    pub language: Language,
    pub llm_config: LlmConfig,
    #[doc = "LLM model identifier"]
    pub llm_model: ::std::string::String,
    pub prompt_metadata: PromptMetadata,
    #[doc = "LLM system instruction"]
    pub system_prompt: ::std::string::String,
    #[doc = "LLM user message with context"]
    pub user_prompt: ::std::string::String,
}
impl ::std::convert::From<&PromptPackage> for PromptPackage {
    fn from(value: &PromptPackage) -> Self {
        value.clone()
    }
}
impl PromptPackage {
    pub fn builder() -> builder::PromptPackage {
        Default::default()
    }
}
#[doc = "How to merge custom restricted words with config defaults"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"How to merge custom restricted words with config defaults\","]
#[doc = "  \"default\": \"Merge\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"Replace\","]
#[doc = "    \"Merge\","]
#[doc = "    \"ConfigOnly\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum RestrictedWordsMergeMode {
    Replace,
    Merge,
    ConfigOnly,
}
impl ::std::convert::From<&Self> for RestrictedWordsMergeMode {
    fn from(value: &RestrictedWordsMergeMode) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for RestrictedWordsMergeMode {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Replace => write!(f, "Replace"),
            Self::Merge => write!(f, "Merge"),
            Self::ConfigOnly => write!(f, "ConfigOnly"),
        }
    }
}
impl ::std::str::FromStr for RestrictedWordsMergeMode {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "Replace" => Ok(Self::Replace),
            "Merge" => Ok(Self::Merge),
            "ConfigOnly" => Ok(Self::ConfigOnly),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for RestrictedWordsMergeMode {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for RestrictedWordsMergeMode {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for RestrictedWordsMergeMode {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::default::Default for RestrictedWordsMergeMode {
    fn default() -> Self {
        RestrictedWordsMergeMode::Merge
    }
}
#[doc = "`ServiceInvocation`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"duration_ms\","]
#[doc = "    \"phase\","]
#[doc = "    \"service_name\","]
#[doc = "    \"started_at\","]
#[doc = "    \"success\","]
#[doc = "    \"tool_name\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"batch_id\": {"]
#[doc = "      \"description\": \"Batch ID if part of batch processing\","]
#[doc = "      \"type\": \"integer\""]
#[doc = "    },"]
#[doc = "    \"duration_ms\": {"]
#[doc = "      \"description\": \"Duration of service call in milliseconds\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"error_message\": {"]
#[doc = "      \"description\": \"Error message if service call failed\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"node_id\": {"]
#[doc = "      \"description\": \"Node ID being processed (for content generation and validation)\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    },"]
#[doc = "    \"phase\": {"]
#[doc = "      \"description\": \"Pipeline phase during which this service was invoked\","]
#[doc = "      \"$ref\": \"#/$defs/GenerationPhase\""]
#[doc = "    },"]
#[doc = "    \"service_name\": {"]
#[doc = "      \"description\": \"MCP service name (prompt-helper, story-generator, quality-control, constraint-enforcer)\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"started_at\": {"]
#[doc = "      \"description\": \"When the service call started\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"date-time\""]
#[doc = "    },"]
#[doc = "    \"success\": {"]
#[doc = "      \"description\": \"Whether the service call succeeded\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"tool_name\": {"]
#[doc = "      \"description\": \"MCP tool invoked (generate_structure, validate_content, etc.)\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct ServiceInvocation {
    #[doc = "Batch ID if part of batch processing"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub batch_id: ::std::option::Option<i64>,
    #[doc = "Duration of service call in milliseconds"]
    pub duration_ms: u64,
    #[doc = "Error message if service call failed"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub error_message: ::std::option::Option<::std::string::String>,
    #[doc = "Node ID being processed (for content generation and validation)"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub node_id: ::std::option::Option<::uuid::Uuid>,
    #[doc = "Pipeline phase during which this service was invoked"]
    pub phase: GenerationPhase,
    #[doc = "MCP service name (prompt-helper, story-generator, quality-control, constraint-enforcer)"]
    pub service_name: ::std::string::String,
    #[doc = "When the service call started"]
    pub started_at: ::chrono::DateTime<::chrono::offset::Utc>,
    #[doc = "Whether the service call succeeded"]
    pub success: bool,
    #[doc = "MCP tool invoked (generate_structure, validate_content, etc.)"]
    pub tool_name: ::std::string::String,
}
impl ::std::convert::From<&ServiceInvocation> for ServiceInvocation {
    fn from(value: &ServiceInvocation) -> Self {
        value.clone()
    }
}
impl ServiceInvocation {
    pub fn builder() -> builder::ServiceInvocation {
        Default::default()
    }
}
#[doc = "Complete schema for TaleTrail AI content generation system with envelope-first architecture"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://schemas.qollective.io/taletrail/content-generator/v1.0.0\","]
#[doc = "  \"title\": \"TaleTrail Content Generator Schema\","]
#[doc = "  \"description\": \"Complete schema for TaleTrail AI content generation system with envelope-first architecture\","]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/AgeGroup\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/ApiVersion\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/BatchInfo\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/Choice\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/ConstraintResult\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/Content\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/ContentNode\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/ContentReference\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/ConvergencePattern\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/CorrectionCapability\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/CorrectionSuggestion\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/CorrectionSummary\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/DAG\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/DagStructureConfig\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/Edge\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/EducationalContent\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/ExternalError\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/ExternalGenerationRequestV1\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/ExternalGenerationResponseV1\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/ExternalJobStatus\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/GatewayMappingConfig\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/GenerationError\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/GenerationMetadata\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/GenerationPhase\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/GenerationRequest\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/GenerationResponse\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/GenerationStatus\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/LLMConfig\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/Language\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/MCPServiceType\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/MappingDefaults\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/MappingError\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/MappingValidation\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/NodeContext\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/PipelineExecutionTrace\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/PromptGenerationMethod\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/PromptGenerationRequest\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/PromptGenerationSummary\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/PromptMetadata\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/PromptPackage\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/RestrictedWordsMergeMode\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/ServiceInvocation\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TaleTrailCustomMetadata\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/Trail\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TrailInsertData\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TrailStatus\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TrailStep\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TrailStepInsertData\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/ValidationIssueSummary\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/ValidationPolicy\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/ValidationResult\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/VocabularyLevel\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/VocabularyViolation\""]
#[doc = "    }"]
#[doc = "  ],"]
#[doc = "  \"qollective\": {"]
#[doc = "    \"envelope\": {"]
#[doc = "      \"enabled\": true,"]
#[doc = "      \"meta_sections\": {"]
#[doc = "        \"debug\": false,"]
#[doc = "        \"monitoring\": true,"]
#[doc = "        \"performance\": true,"]
#[doc = "        \"security\": true,"]
#[doc = "        \"tracing\": true"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"generation\": {"]
#[doc = "      \"features\": ["]
#[doc = "        \"tenant-extraction\","]
#[doc = "        \"validation\","]
#[doc = "        \"tls\","]
#[doc = "        \"mcp-client\","]
#[doc = "        \"mcp-server\","]
#[doc = "        \"nats\","]
#[doc = "        \"rest-server\""]
#[doc = "      ],"]
#[doc = "      \"outputDir\": \"./shared-types/src/generated\","]
#[doc = "      \"targets\": ["]
#[doc = "        \"rust-rest\","]
#[doc = "        \"rust-nats\","]
#[doc = "        \"rust-mcp\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"tenant_extraction\": {"]
#[doc = "      \"enabled\": true,"]
#[doc = "      \"jwt_extraction\": {"]
#[doc = "        \"permissions_claim\": \"permissions\","]
#[doc = "        \"tenant_claim\": \"tenant_id\","]
#[doc = "        \"user_claim\": \"sub\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"validation\": {"]
#[doc = "      \"enabled\": true,"]
#[doc = "      \"strict_mode\": true"]
#[doc = "    },"]
#[doc = "    \"version\": \"1.0.0\""]
#[doc = "  },"]
#[doc = "  \"version\": \"1.0.0\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum TaleTrailContentGeneratorSchema {
    AgeGroup(AgeGroup),
    ApiVersion(ApiVersion),
    BatchInfo(BatchInfo),
    Choice(Choice),
    ConstraintResult(ConstraintResult),
    Content(Content),
    ContentNode(ContentNode),
    ContentReference(ContentReference),
    ConvergencePattern(ConvergencePattern),
    CorrectionCapability(CorrectionCapability),
    CorrectionSuggestion(CorrectionSuggestion),
    CorrectionSummary(CorrectionSummary),
    Dag(Dag),
    DagStructureConfig(DagStructureConfig),
    Edge(Edge),
    EducationalContent(EducationalContent),
    ExternalError(ExternalError),
    ExternalGenerationRequestV1(ExternalGenerationRequestV1),
    ExternalGenerationResponseV1(ExternalGenerationResponseV1),
    ExternalJobStatus(ExternalJobStatus),
    GatewayMappingConfig(GatewayMappingConfig),
    GenerationError(GenerationError),
    GenerationMetadata(GenerationMetadata),
    GenerationPhase(GenerationPhase),
    GenerationRequest(GenerationRequest),
    GenerationResponse(GenerationResponse),
    GenerationStatus(GenerationStatus),
    LlmConfig(LlmConfig),
    Language(Language),
    McpServiceType(McpServiceType),
    MappingDefaults(MappingDefaults),
    MappingError(MappingError),
    MappingValidation(MappingValidation),
    NodeContext(NodeContext),
    PipelineExecutionTrace(PipelineExecutionTrace),
    PromptGenerationMethod(PromptGenerationMethod),
    PromptGenerationRequest(PromptGenerationRequest),
    PromptGenerationSummary(PromptGenerationSummary),
    PromptMetadata(PromptMetadata),
    PromptPackage(PromptPackage),
    RestrictedWordsMergeMode(RestrictedWordsMergeMode),
    ServiceInvocation(ServiceInvocation),
    TaleTrailCustomMetadata(TaleTrailCustomMetadata),
    Trail(Trail),
    TrailInsertData(TrailInsertData),
    TrailStatus(TrailStatus),
    TrailStep(TrailStep),
    TrailStepInsertData(TrailStepInsertData),
    ValidationIssueSummary(ValidationIssueSummary),
    ValidationPolicy(ValidationPolicy),
    ValidationResult(ValidationResult),
    VocabularyLevel(VocabularyLevel),
    VocabularyViolation(VocabularyViolation),
}
impl ::std::convert::From<&Self> for TaleTrailContentGeneratorSchema {
    fn from(value: &TaleTrailContentGeneratorSchema) -> Self {
        value.clone()
    }
}
impl ::std::convert::From<AgeGroup> for TaleTrailContentGeneratorSchema {
    fn from(value: AgeGroup) -> Self {
        Self::AgeGroup(value)
    }
}
impl ::std::convert::From<ApiVersion> for TaleTrailContentGeneratorSchema {
    fn from(value: ApiVersion) -> Self {
        Self::ApiVersion(value)
    }
}
impl ::std::convert::From<BatchInfo> for TaleTrailContentGeneratorSchema {
    fn from(value: BatchInfo) -> Self {
        Self::BatchInfo(value)
    }
}
impl ::std::convert::From<Choice> for TaleTrailContentGeneratorSchema {
    fn from(value: Choice) -> Self {
        Self::Choice(value)
    }
}
impl ::std::convert::From<ConstraintResult> for TaleTrailContentGeneratorSchema {
    fn from(value: ConstraintResult) -> Self {
        Self::ConstraintResult(value)
    }
}
impl ::std::convert::From<Content> for TaleTrailContentGeneratorSchema {
    fn from(value: Content) -> Self {
        Self::Content(value)
    }
}
impl ::std::convert::From<ContentNode> for TaleTrailContentGeneratorSchema {
    fn from(value: ContentNode) -> Self {
        Self::ContentNode(value)
    }
}
impl ::std::convert::From<ContentReference> for TaleTrailContentGeneratorSchema {
    fn from(value: ContentReference) -> Self {
        Self::ContentReference(value)
    }
}
impl ::std::convert::From<ConvergencePattern> for TaleTrailContentGeneratorSchema {
    fn from(value: ConvergencePattern) -> Self {
        Self::ConvergencePattern(value)
    }
}
impl ::std::convert::From<CorrectionCapability> for TaleTrailContentGeneratorSchema {
    fn from(value: CorrectionCapability) -> Self {
        Self::CorrectionCapability(value)
    }
}
impl ::std::convert::From<CorrectionSuggestion> for TaleTrailContentGeneratorSchema {
    fn from(value: CorrectionSuggestion) -> Self {
        Self::CorrectionSuggestion(value)
    }
}
impl ::std::convert::From<CorrectionSummary> for TaleTrailContentGeneratorSchema {
    fn from(value: CorrectionSummary) -> Self {
        Self::CorrectionSummary(value)
    }
}
impl ::std::convert::From<Dag> for TaleTrailContentGeneratorSchema {
    fn from(value: Dag) -> Self {
        Self::Dag(value)
    }
}
impl ::std::convert::From<DagStructureConfig> for TaleTrailContentGeneratorSchema {
    fn from(value: DagStructureConfig) -> Self {
        Self::DagStructureConfig(value)
    }
}
impl ::std::convert::From<Edge> for TaleTrailContentGeneratorSchema {
    fn from(value: Edge) -> Self {
        Self::Edge(value)
    }
}
impl ::std::convert::From<EducationalContent> for TaleTrailContentGeneratorSchema {
    fn from(value: EducationalContent) -> Self {
        Self::EducationalContent(value)
    }
}
impl ::std::convert::From<ExternalError> for TaleTrailContentGeneratorSchema {
    fn from(value: ExternalError) -> Self {
        Self::ExternalError(value)
    }
}
impl ::std::convert::From<ExternalGenerationRequestV1> for TaleTrailContentGeneratorSchema {
    fn from(value: ExternalGenerationRequestV1) -> Self {
        Self::ExternalGenerationRequestV1(value)
    }
}
impl ::std::convert::From<ExternalGenerationResponseV1> for TaleTrailContentGeneratorSchema {
    fn from(value: ExternalGenerationResponseV1) -> Self {
        Self::ExternalGenerationResponseV1(value)
    }
}
impl ::std::convert::From<ExternalJobStatus> for TaleTrailContentGeneratorSchema {
    fn from(value: ExternalJobStatus) -> Self {
        Self::ExternalJobStatus(value)
    }
}
impl ::std::convert::From<GatewayMappingConfig> for TaleTrailContentGeneratorSchema {
    fn from(value: GatewayMappingConfig) -> Self {
        Self::GatewayMappingConfig(value)
    }
}
impl ::std::convert::From<GenerationError> for TaleTrailContentGeneratorSchema {
    fn from(value: GenerationError) -> Self {
        Self::GenerationError(value)
    }
}
impl ::std::convert::From<GenerationMetadata> for TaleTrailContentGeneratorSchema {
    fn from(value: GenerationMetadata) -> Self {
        Self::GenerationMetadata(value)
    }
}
impl ::std::convert::From<GenerationPhase> for TaleTrailContentGeneratorSchema {
    fn from(value: GenerationPhase) -> Self {
        Self::GenerationPhase(value)
    }
}
impl ::std::convert::From<GenerationRequest> for TaleTrailContentGeneratorSchema {
    fn from(value: GenerationRequest) -> Self {
        Self::GenerationRequest(value)
    }
}
impl ::std::convert::From<GenerationResponse> for TaleTrailContentGeneratorSchema {
    fn from(value: GenerationResponse) -> Self {
        Self::GenerationResponse(value)
    }
}
impl ::std::convert::From<GenerationStatus> for TaleTrailContentGeneratorSchema {
    fn from(value: GenerationStatus) -> Self {
        Self::GenerationStatus(value)
    }
}
impl ::std::convert::From<LlmConfig> for TaleTrailContentGeneratorSchema {
    fn from(value: LlmConfig) -> Self {
        Self::LlmConfig(value)
    }
}
impl ::std::convert::From<Language> for TaleTrailContentGeneratorSchema {
    fn from(value: Language) -> Self {
        Self::Language(value)
    }
}
impl ::std::convert::From<McpServiceType> for TaleTrailContentGeneratorSchema {
    fn from(value: McpServiceType) -> Self {
        Self::McpServiceType(value)
    }
}
impl ::std::convert::From<MappingDefaults> for TaleTrailContentGeneratorSchema {
    fn from(value: MappingDefaults) -> Self {
        Self::MappingDefaults(value)
    }
}
impl ::std::convert::From<MappingError> for TaleTrailContentGeneratorSchema {
    fn from(value: MappingError) -> Self {
        Self::MappingError(value)
    }
}
impl ::std::convert::From<MappingValidation> for TaleTrailContentGeneratorSchema {
    fn from(value: MappingValidation) -> Self {
        Self::MappingValidation(value)
    }
}
impl ::std::convert::From<NodeContext> for TaleTrailContentGeneratorSchema {
    fn from(value: NodeContext) -> Self {
        Self::NodeContext(value)
    }
}
impl ::std::convert::From<PipelineExecutionTrace> for TaleTrailContentGeneratorSchema {
    fn from(value: PipelineExecutionTrace) -> Self {
        Self::PipelineExecutionTrace(value)
    }
}
impl ::std::convert::From<PromptGenerationMethod> for TaleTrailContentGeneratorSchema {
    fn from(value: PromptGenerationMethod) -> Self {
        Self::PromptGenerationMethod(value)
    }
}
impl ::std::convert::From<PromptGenerationRequest> for TaleTrailContentGeneratorSchema {
    fn from(value: PromptGenerationRequest) -> Self {
        Self::PromptGenerationRequest(value)
    }
}
impl ::std::convert::From<PromptGenerationSummary> for TaleTrailContentGeneratorSchema {
    fn from(value: PromptGenerationSummary) -> Self {
        Self::PromptGenerationSummary(value)
    }
}
impl ::std::convert::From<PromptMetadata> for TaleTrailContentGeneratorSchema {
    fn from(value: PromptMetadata) -> Self {
        Self::PromptMetadata(value)
    }
}
impl ::std::convert::From<PromptPackage> for TaleTrailContentGeneratorSchema {
    fn from(value: PromptPackage) -> Self {
        Self::PromptPackage(value)
    }
}
impl ::std::convert::From<RestrictedWordsMergeMode> for TaleTrailContentGeneratorSchema {
    fn from(value: RestrictedWordsMergeMode) -> Self {
        Self::RestrictedWordsMergeMode(value)
    }
}
impl ::std::convert::From<ServiceInvocation> for TaleTrailContentGeneratorSchema {
    fn from(value: ServiceInvocation) -> Self {
        Self::ServiceInvocation(value)
    }
}
impl ::std::convert::From<TaleTrailCustomMetadata> for TaleTrailContentGeneratorSchema {
    fn from(value: TaleTrailCustomMetadata) -> Self {
        Self::TaleTrailCustomMetadata(value)
    }
}
impl ::std::convert::From<Trail> for TaleTrailContentGeneratorSchema {
    fn from(value: Trail) -> Self {
        Self::Trail(value)
    }
}
impl ::std::convert::From<TrailInsertData> for TaleTrailContentGeneratorSchema {
    fn from(value: TrailInsertData) -> Self {
        Self::TrailInsertData(value)
    }
}
impl ::std::convert::From<TrailStatus> for TaleTrailContentGeneratorSchema {
    fn from(value: TrailStatus) -> Self {
        Self::TrailStatus(value)
    }
}
impl ::std::convert::From<TrailStep> for TaleTrailContentGeneratorSchema {
    fn from(value: TrailStep) -> Self {
        Self::TrailStep(value)
    }
}
impl ::std::convert::From<TrailStepInsertData> for TaleTrailContentGeneratorSchema {
    fn from(value: TrailStepInsertData) -> Self {
        Self::TrailStepInsertData(value)
    }
}
impl ::std::convert::From<ValidationIssueSummary> for TaleTrailContentGeneratorSchema {
    fn from(value: ValidationIssueSummary) -> Self {
        Self::ValidationIssueSummary(value)
    }
}
impl ::std::convert::From<ValidationPolicy> for TaleTrailContentGeneratorSchema {
    fn from(value: ValidationPolicy) -> Self {
        Self::ValidationPolicy(value)
    }
}
impl ::std::convert::From<ValidationResult> for TaleTrailContentGeneratorSchema {
    fn from(value: ValidationResult) -> Self {
        Self::ValidationResult(value)
    }
}
impl ::std::convert::From<VocabularyLevel> for TaleTrailContentGeneratorSchema {
    fn from(value: VocabularyLevel) -> Self {
        Self::VocabularyLevel(value)
    }
}
impl ::std::convert::From<VocabularyViolation> for TaleTrailContentGeneratorSchema {
    fn from(value: VocabularyViolation) -> Self {
        Self::VocabularyViolation(value)
    }
}
#[doc = "Custom metadata extensions for TaleTrail content generation (stored in Meta.extensions)"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Custom metadata extensions for TaleTrail content generation (stored in Meta.extensions)\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"batch_id\": {"]
#[doc = "      \"description\": \"Batch identifier for grouping related generation operations\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    },"]
#[doc = "    \"correlation_id\": {"]
#[doc = "      \"description\": \"Correlation ID for tracking request chains across services\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    },"]
#[doc = "    \"generation_phase\": {"]
#[doc = "      \"description\": \"Current phase of the generation pipeline\","]
#[doc = "      \"$ref\": \"#/$defs/GenerationPhase\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct TaleTrailCustomMetadata {
    #[doc = "Batch identifier for grouping related generation operations"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub batch_id: ::std::option::Option<::uuid::Uuid>,
    #[doc = "Correlation ID for tracking request chains across services"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub correlation_id: ::std::option::Option<::uuid::Uuid>,
    #[doc = "Current phase of the generation pipeline"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub generation_phase: ::std::option::Option<GenerationPhase>,
}
impl ::std::convert::From<&TaleTrailCustomMetadata> for TaleTrailCustomMetadata {
    fn from(value: &TaleTrailCustomMetadata) -> Self {
        value.clone()
    }
}
impl ::std::default::Default for TaleTrailCustomMetadata {
    fn default() -> Self {
        Self {
            batch_id: Default::default(),
            correlation_id: Default::default(),
            generation_phase: Default::default(),
        }
    }
}
impl TaleTrailCustomMetadata {
    pub fn builder() -> builder::TaleTrailCustomMetadata {
        Default::default()
    }
}
#[doc = "`Trail`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"is_public\","]
#[doc = "    \"metadata\","]
#[doc = "    \"status\","]
#[doc = "    \"title\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"category\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"story\""]
#[doc = "    },"]
#[doc = "    \"description\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 2000"]
#[doc = "    },"]
#[doc = "    \"is_public\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"description\": \"Generated metadata: generation_params, word_count, ai_model, etc.\","]
#[doc = "      \"type\": \"object\""]
#[doc = "    },"]
#[doc = "    \"price_coins\": {"]
#[doc = "      \"type\": ["]
#[doc = "        \"integer\","]
#[doc = "        \"null\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"status\": {"]
#[doc = "      \"$ref\": \"#/$defs/TrailStatus\""]
#[doc = "    },"]
#[doc = "    \"tags\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"title\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 255,"]
#[doc = "      \"minLength\": 5"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct Trail {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub category: ::std::option::Option<::std::string::String>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub description: ::std::option::Option<TrailDescription>,
    pub is_public: bool,
    #[doc = "Generated metadata: generation_params, word_count, ai_model, etc."]
    pub metadata: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub price_coins: ::std::option::Option<i64>,
    pub status: TrailStatus,
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub tags: ::std::vec::Vec<::std::string::String>,
    pub title: TrailTitle,
}
impl ::std::convert::From<&Trail> for Trail {
    fn from(value: &Trail) -> Self {
        value.clone()
    }
}
impl Trail {
    pub fn builder() -> builder::Trail {
        Default::default()
    }
}
#[doc = "`TrailDescription`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 2000"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct TrailDescription(::std::string::String);
impl ::std::ops::Deref for TrailDescription {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TrailDescription> for ::std::string::String {
    fn from(value: TrailDescription) -> Self {
        value.0
    }
}
impl ::std::convert::From<&TrailDescription> for TrailDescription {
    fn from(value: &TrailDescription) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for TrailDescription {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 2000usize {
            return Err("longer than 2000 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for TrailDescription {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TrailDescription {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TrailDescription {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TrailDescription {
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
#[doc = "`TrailInsertData`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"is_public\","]
#[doc = "    \"metadata\","]
#[doc = "    \"status\","]
#[doc = "    \"title\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"description\": {"]
#[doc = "      \"description\": \"Trail description\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 2000"]
#[doc = "    },"]
#[doc = "    \"is_public\": {"]
#[doc = "      \"description\": \"Public visibility\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"description\": \"Trail metadata JSON (generation params, word count, etc.)\","]
#[doc = "      \"type\": \"object\""]
#[doc = "    },"]
#[doc = "    \"price_coins\": {"]
#[doc = "      \"description\": \"Price in coins (null for free content)\","]
#[doc = "      \"type\": ["]
#[doc = "        \"integer\","]
#[doc = "        \"null\""]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"status\": {"]
#[doc = "      \"$ref\": \"#/$defs/TrailStatus\""]
#[doc = "    },"]
#[doc = "    \"tags\": {"]
#[doc = "      \"description\": \"Categorization tags\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\","]
#[doc = "        \"maxLength\": 50"]
#[doc = "      },"]
#[doc = "      \"maxItems\": 20"]
#[doc = "    },"]
#[doc = "    \"title\": {"]
#[doc = "      \"description\": \"Trail title\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 255,"]
#[doc = "      \"minLength\": 5"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct TrailInsertData {
    #[doc = "Trail description"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub description: ::std::option::Option<TrailInsertDataDescription>,
    #[doc = "Public visibility"]
    pub is_public: bool,
    #[doc = "Trail metadata JSON (generation params, word count, etc.)"]
    pub metadata: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    #[doc = "Price in coins (null for free content)"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub price_coins: ::std::option::Option<i64>,
    pub status: TrailStatus,
    #[doc = "Categorization tags"]
    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub tags: ::std::vec::Vec<TrailInsertDataTagsItem>,
    #[doc = "Trail title"]
    pub title: TrailInsertDataTitle,
}
impl ::std::convert::From<&TrailInsertData> for TrailInsertData {
    fn from(value: &TrailInsertData) -> Self {
        value.clone()
    }
}
impl TrailInsertData {
    pub fn builder() -> builder::TrailInsertData {
        Default::default()
    }
}
#[doc = "Trail description"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Trail description\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 2000"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct TrailInsertDataDescription(::std::string::String);
impl ::std::ops::Deref for TrailInsertDataDescription {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TrailInsertDataDescription> for ::std::string::String {
    fn from(value: TrailInsertDataDescription) -> Self {
        value.0
    }
}
impl ::std::convert::From<&TrailInsertDataDescription> for TrailInsertDataDescription {
    fn from(value: &TrailInsertDataDescription) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for TrailInsertDataDescription {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 2000usize {
            return Err("longer than 2000 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for TrailInsertDataDescription {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TrailInsertDataDescription {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TrailInsertDataDescription {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TrailInsertDataDescription {
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
#[doc = "`TrailInsertDataTagsItem`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 50"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct TrailInsertDataTagsItem(::std::string::String);
impl ::std::ops::Deref for TrailInsertDataTagsItem {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TrailInsertDataTagsItem> for ::std::string::String {
    fn from(value: TrailInsertDataTagsItem) -> Self {
        value.0
    }
}
impl ::std::convert::From<&TrailInsertDataTagsItem> for TrailInsertDataTagsItem {
    fn from(value: &TrailInsertDataTagsItem) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for TrailInsertDataTagsItem {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 50usize {
            return Err("longer than 50 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for TrailInsertDataTagsItem {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TrailInsertDataTagsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TrailInsertDataTagsItem {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TrailInsertDataTagsItem {
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
#[doc = "Trail title"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Trail title\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 255,"]
#[doc = "  \"minLength\": 5"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct TrailInsertDataTitle(::std::string::String);
impl ::std::ops::Deref for TrailInsertDataTitle {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TrailInsertDataTitle> for ::std::string::String {
    fn from(value: TrailInsertDataTitle) -> Self {
        value.0
    }
}
impl ::std::convert::From<&TrailInsertDataTitle> for TrailInsertDataTitle {
    fn from(value: &TrailInsertDataTitle) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for TrailInsertDataTitle {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 255usize {
            return Err("longer than 255 characters".into());
        }
        if value.chars().count() < 5usize {
            return Err("shorter than 5 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for TrailInsertDataTitle {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TrailInsertDataTitle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TrailInsertDataTitle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TrailInsertDataTitle {
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
#[doc = "Publication status of trail (matches DB enum)"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Publication status of trail (matches DB enum)\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"DRAFT\","]
#[doc = "    \"PUBLISHED\","]
#[doc = "    \"ARCHIVED\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum TrailStatus {
    #[serde(rename = "DRAFT")]
    Draft,
    #[serde(rename = "PUBLISHED")]
    Published,
    #[serde(rename = "ARCHIVED")]
    Archived,
}
impl ::std::convert::From<&Self> for TrailStatus {
    fn from(value: &TrailStatus) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for TrailStatus {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Draft => write!(f, "DRAFT"),
            Self::Published => write!(f, "PUBLISHED"),
            Self::Archived => write!(f, "ARCHIVED"),
        }
    }
}
impl ::std::str::FromStr for TrailStatus {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "DRAFT" => Ok(Self::Draft),
            "PUBLISHED" => Ok(Self::Published),
            "ARCHIVED" => Ok(Self::Archived),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for TrailStatus {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TrailStatus {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TrailStatus {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`TrailStep`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"content_reference\","]
#[doc = "    \"is_required\","]
#[doc = "    \"metadata\","]
#[doc = "    \"step_order\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"content_reference\": {"]
#[doc = "      \"$ref\": \"#/$defs/ContentReference\""]
#[doc = "    },"]
#[doc = "    \"description\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 1000"]
#[doc = "    },"]
#[doc = "    \"is_required\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"description\": \"word_count, node_id, convergence_point, etc.\","]
#[doc = "      \"type\": \"object\""]
#[doc = "    },"]
#[doc = "    \"step_order\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"title\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 255"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct TrailStep {
    pub content_reference: ContentReference,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub description: ::std::option::Option<TrailStepDescription>,
    pub is_required: bool,
    #[doc = "word_count, node_id, convergence_point, etc."]
    pub metadata: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    pub step_order: ::std::num::NonZeroU64,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub title: ::std::option::Option<TrailStepTitle>,
}
impl ::std::convert::From<&TrailStep> for TrailStep {
    fn from(value: &TrailStep) -> Self {
        value.clone()
    }
}
impl TrailStep {
    pub fn builder() -> builder::TrailStep {
        Default::default()
    }
}
#[doc = "`TrailStepDescription`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 1000"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct TrailStepDescription(::std::string::String);
impl ::std::ops::Deref for TrailStepDescription {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TrailStepDescription> for ::std::string::String {
    fn from(value: TrailStepDescription) -> Self {
        value.0
    }
}
impl ::std::convert::From<&TrailStepDescription> for TrailStepDescription {
    fn from(value: &TrailStepDescription) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for TrailStepDescription {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 1000usize {
            return Err("longer than 1000 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for TrailStepDescription {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TrailStepDescription {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TrailStepDescription {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TrailStepDescription {
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
#[doc = "`TrailStepInsertData`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"content_data\","]
#[doc = "    \"is_required\","]
#[doc = "    \"metadata\","]
#[doc = "    \"step_order\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"content_data\": {"]
#[doc = "      \"description\": \"Interactive story node content\","]
#[doc = "      \"type\": \"object\""]
#[doc = "    },"]
#[doc = "    \"description\": {"]
#[doc = "      \"description\": \"Step description\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 1000"]
#[doc = "    },"]
#[doc = "    \"is_required\": {"]
#[doc = "      \"description\": \"Whether step is required\","]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"description\": \"Step metadata (word count, node_id, convergence point, etc.)\","]
#[doc = "      \"type\": \"object\""]
#[doc = "    },"]
#[doc = "    \"step_order\": {"]
#[doc = "      \"description\": \"Sequential order of step\","]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 1.0"]
#[doc = "    },"]
#[doc = "    \"title\": {"]
#[doc = "      \"description\": \"Step title\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 255"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct TrailStepInsertData {
    #[doc = "Interactive story node content"]
    pub content_data: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    #[doc = "Step description"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub description: ::std::option::Option<TrailStepInsertDataDescription>,
    #[doc = "Whether step is required"]
    pub is_required: bool,
    #[doc = "Step metadata (word count, node_id, convergence point, etc.)"]
    pub metadata: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    #[doc = "Sequential order of step"]
    pub step_order: ::std::num::NonZeroU64,
    #[doc = "Step title"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub title: ::std::option::Option<TrailStepInsertDataTitle>,
}
impl ::std::convert::From<&TrailStepInsertData> for TrailStepInsertData {
    fn from(value: &TrailStepInsertData) -> Self {
        value.clone()
    }
}
impl TrailStepInsertData {
    pub fn builder() -> builder::TrailStepInsertData {
        Default::default()
    }
}
#[doc = "Step description"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Step description\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 1000"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct TrailStepInsertDataDescription(::std::string::String);
impl ::std::ops::Deref for TrailStepInsertDataDescription {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TrailStepInsertDataDescription> for ::std::string::String {
    fn from(value: TrailStepInsertDataDescription) -> Self {
        value.0
    }
}
impl ::std::convert::From<&TrailStepInsertDataDescription> for TrailStepInsertDataDescription {
    fn from(value: &TrailStepInsertDataDescription) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for TrailStepInsertDataDescription {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 1000usize {
            return Err("longer than 1000 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for TrailStepInsertDataDescription {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TrailStepInsertDataDescription {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TrailStepInsertDataDescription {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TrailStepInsertDataDescription {
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
#[doc = "Step title"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Step title\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 255"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct TrailStepInsertDataTitle(::std::string::String);
impl ::std::ops::Deref for TrailStepInsertDataTitle {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TrailStepInsertDataTitle> for ::std::string::String {
    fn from(value: TrailStepInsertDataTitle) -> Self {
        value.0
    }
}
impl ::std::convert::From<&TrailStepInsertDataTitle> for TrailStepInsertDataTitle {
    fn from(value: &TrailStepInsertDataTitle) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for TrailStepInsertDataTitle {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 255usize {
            return Err("longer than 255 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for TrailStepInsertDataTitle {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TrailStepInsertDataTitle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TrailStepInsertDataTitle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TrailStepInsertDataTitle {
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
#[doc = "`TrailStepTitle`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 255"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct TrailStepTitle(::std::string::String);
impl ::std::ops::Deref for TrailStepTitle {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TrailStepTitle> for ::std::string::String {
    fn from(value: TrailStepTitle) -> Self {
        value.0
    }
}
impl ::std::convert::From<&TrailStepTitle> for TrailStepTitle {
    fn from(value: &TrailStepTitle) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for TrailStepTitle {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 255usize {
            return Err("longer than 255 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for TrailStepTitle {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TrailStepTitle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TrailStepTitle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TrailStepTitle {
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
#[doc = "`TrailTitle`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 255,"]
#[doc = "  \"minLength\": 5"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct TrailTitle(::std::string::String);
impl ::std::ops::Deref for TrailTitle {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TrailTitle> for ::std::string::String {
    fn from(value: TrailTitle) -> Self {
        value.0
    }
}
impl ::std::convert::From<&TrailTitle> for TrailTitle {
    fn from(value: &TrailTitle) -> Self {
        value.clone()
    }
}
impl ::std::str::FromStr for TrailTitle {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 255usize {
            return Err("longer than 255 characters".into());
        }
        if value.chars().count() < 5usize {
            return Err("shorter than 5 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for TrailTitle {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TrailTitle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TrailTitle {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TrailTitle {
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
#[doc = "`ValidationIssueSummary`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"description\","]
#[doc = "    \"issue_type\","]
#[doc = "    \"node_id\","]
#[doc = "    \"severity\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"description\": {"]
#[doc = "      \"description\": \"Human-readable issue description\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"issue_type\": {"]
#[doc = "      \"description\": \"Type of validation issue (e.g., 'age_appropriateness', 'word_count')\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"node_id\": {"]
#[doc = "      \"description\": \"ID of node with unresolved issue\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"severity\": {"]
#[doc = "      \"description\": \"Issue severity level\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"Critical\","]
#[doc = "        \"Warning\","]
#[doc = "        \"Info\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct ValidationIssueSummary {
    #[doc = "Human-readable issue description"]
    pub description: ::std::string::String,
    #[doc = "Type of validation issue (e.g., 'age_appropriateness', 'word_count')"]
    pub issue_type: ::std::string::String,
    #[doc = "ID of node with unresolved issue"]
    pub node_id: ::std::string::String,
    #[doc = "Issue severity level"]
    pub severity: ValidationIssueSummarySeverity,
}
impl ::std::convert::From<&ValidationIssueSummary> for ValidationIssueSummary {
    fn from(value: &ValidationIssueSummary) -> Self {
        value.clone()
    }
}
impl ValidationIssueSummary {
    pub fn builder() -> builder::ValidationIssueSummary {
        Default::default()
    }
}
#[doc = "Issue severity level"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Issue severity level\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"Critical\","]
#[doc = "    \"Warning\","]
#[doc = "    \"Info\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum ValidationIssueSummarySeverity {
    Critical,
    Warning,
    Info,
}
impl ::std::convert::From<&Self> for ValidationIssueSummarySeverity {
    fn from(value: &ValidationIssueSummarySeverity) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for ValidationIssueSummarySeverity {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Critical => write!(f, "Critical"),
            Self::Warning => write!(f, "Warning"),
            Self::Info => write!(f, "Info"),
        }
    }
}
impl ::std::str::FromStr for ValidationIssueSummarySeverity {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "Critical" => Ok(Self::Critical),
            "Warning" => Ok(Self::Warning),
            "Info" => Ok(Self::Info),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ValidationIssueSummarySeverity {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ValidationIssueSummarySeverity {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ValidationIssueSummarySeverity {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Validation policy for content generation"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Validation policy for content generation\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"custom_restricted_words\": {"]
#[doc = "      \"description\": \"Custom restricted words per language (overrides or merges with config defaults)\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"additionalProperties\": {"]
#[doc = "        \"type\": \"array\","]
#[doc = "        \"items\": {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        }"]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"enable_validation\": {"]
#[doc = "      \"description\": \"Whether to enable content validation\","]
#[doc = "      \"default\": true,"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"merge_mode\": {"]
#[doc = "      \"description\": \"How to handle custom_restricted_words vs config defaults\","]
#[doc = "      \"$ref\": \"#/$defs/RestrictedWordsMergeMode\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct ValidationPolicy {
    #[doc = "Custom restricted words per language (overrides or merges with config defaults)"]
    #[serde(
        default,
        skip_serializing_if = ":: std :: collections :: HashMap::is_empty"
    )]
    pub custom_restricted_words:
        ::std::collections::HashMap<::std::string::String, ::std::vec::Vec<::std::string::String>>,
    #[doc = "Whether to enable content validation"]
    #[serde(default = "defaults::default_bool::<true>")]
    pub enable_validation: bool,
    #[doc = "How to handle custom_restricted_words vs config defaults"]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub merge_mode: ::std::option::Option<RestrictedWordsMergeMode>,
}
impl ::std::convert::From<&ValidationPolicy> for ValidationPolicy {
    fn from(value: &ValidationPolicy) -> Self {
        value.clone()
    }
}
impl ::std::default::Default for ValidationPolicy {
    fn default() -> Self {
        Self {
            custom_restricted_words: Default::default(),
            enable_validation: defaults::default_bool::<true>(),
            merge_mode: Default::default(),
        }
    }
}
impl ValidationPolicy {
    pub fn builder() -> builder::ValidationPolicy {
        Default::default()
    }
}
#[doc = "`ValidationResult`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"age_appropriate_score\","]
#[doc = "    \"correction_capability\","]
#[doc = "    \"corrections\","]
#[doc = "    \"educational_value_score\","]
#[doc = "    \"is_valid\","]
#[doc = "    \"safety_issues\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"age_appropriate_score\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"correction_capability\": {"]
#[doc = "      \"$ref\": \"#/$defs/CorrectionCapability\""]
#[doc = "    },"]
#[doc = "    \"corrections\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/CorrectionSuggestion\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"educational_value_score\": {"]
#[doc = "      \"type\": \"number\","]
#[doc = "      \"maximum\": 1.0,"]
#[doc = "      \"minimum\": 0.0"]
#[doc = "    },"]
#[doc = "    \"is_valid\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"safety_issues\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct ValidationResult {
    pub age_appropriate_score: f64,
    pub correction_capability: CorrectionCapability,
    pub corrections: ::std::vec::Vec<CorrectionSuggestion>,
    pub educational_value_score: f64,
    pub is_valid: bool,
    pub safety_issues: ::std::vec::Vec<::std::string::String>,
}
impl ::std::convert::From<&ValidationResult> for ValidationResult {
    fn from(value: &ValidationResult) -> Self {
        value.clone()
    }
}
impl ValidationResult {
    pub fn builder() -> builder::ValidationResult {
        Default::default()
    }
}
#[doc = "Vocabulary complexity level"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"description\": \"Vocabulary complexity level\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"basic\","]
#[doc = "    \"intermediate\","]
#[doc = "    \"advanced\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum VocabularyLevel {
    #[serde(rename = "basic")]
    Basic,
    #[serde(rename = "intermediate")]
    Intermediate,
    #[serde(rename = "advanced")]
    Advanced,
}
impl ::std::convert::From<&Self> for VocabularyLevel {
    fn from(value: &VocabularyLevel) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for VocabularyLevel {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Basic => write!(f, "basic"),
            Self::Intermediate => write!(f, "intermediate"),
            Self::Advanced => write!(f, "advanced"),
        }
    }
}
impl ::std::str::FromStr for VocabularyLevel {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "basic" => Ok(Self::Basic),
            "intermediate" => Ok(Self::Intermediate),
            "advanced" => Ok(Self::Advanced),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for VocabularyLevel {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for VocabularyLevel {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for VocabularyLevel {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`VocabularyViolation`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"current_level\","]
#[doc = "    \"node_id\","]
#[doc = "    \"suggestions\","]
#[doc = "    \"target_level\","]
#[doc = "    \"word\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"current_level\": {"]
#[doc = "      \"$ref\": \"#/$defs/VocabularyLevel\""]
#[doc = "    },"]
#[doc = "    \"node_id\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"uuid\""]
#[doc = "    },"]
#[doc = "    \"suggestions\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"target_level\": {"]
#[doc = "      \"$ref\": \"#/$defs/VocabularyLevel\""]
#[doc = "    },"]
#[doc = "    \"word\": {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug, PartialEq)]
pub struct VocabularyViolation {
    pub current_level: VocabularyLevel,
    pub node_id: ::uuid::Uuid,
    pub suggestions: ::std::vec::Vec<::std::string::String>,
    pub target_level: VocabularyLevel,
    pub word: ::std::string::String,
}
impl ::std::convert::From<&VocabularyViolation> for VocabularyViolation {
    fn from(value: &VocabularyViolation) -> Self {
        value.clone()
    }
}
impl VocabularyViolation {
    pub fn builder() -> builder::VocabularyViolation {
        Default::default()
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct BatchInfo {
        batch_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
        batch_index: ::std::result::Result<u8, ::std::string::String>,
        batch_size: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        total_batches: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
    }
    impl ::std::default::Default for BatchInfo {
        fn default() -> Self {
            Self {
                batch_id: Err("no value supplied for batch_id".to_string()),
                batch_index: Err("no value supplied for batch_index".to_string()),
                batch_size: Err("no value supplied for batch_size".to_string()),
                total_batches: Err("no value supplied for total_batches".to_string()),
            }
        }
    }
    impl BatchInfo {
        pub fn batch_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::uuid::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.batch_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for batch_id: {}", e));
            self
        }
        pub fn batch_index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u8>,
            T::Error: ::std::fmt::Display,
        {
            self.batch_index = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for batch_index: {}", e));
            self
        }
        pub fn batch_size<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.batch_size = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for batch_size: {}", e));
            self
        }
        pub fn total_batches<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.total_batches = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for total_batches: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<BatchInfo> for super::BatchInfo {
        type Error = super::error::ConversionError;
        fn try_from(
            value: BatchInfo,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                batch_id: value.batch_id?,
                batch_index: value.batch_index?,
                batch_size: value.batch_size?,
                total_batches: value.total_batches?,
            })
        }
    }
    impl ::std::convert::From<super::BatchInfo> for BatchInfo {
        fn from(value: super::BatchInfo) -> Self {
            Self {
                batch_id: Ok(value.batch_id),
                batch_index: Ok(value.batch_index),
                batch_size: Ok(value.batch_size),
                total_batches: Ok(value.total_batches),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Choice {
        id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
        metadata: ::std::result::Result<
            ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            ::std::string::String,
        >,
        next_node_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
        text: ::std::result::Result<super::ChoiceText, ::std::string::String>,
    }
    impl ::std::default::Default for Choice {
        fn default() -> Self {
            Self {
                id: Err("no value supplied for id".to_string()),
                metadata: Ok(Default::default()),
                next_node_id: Err("no value supplied for next_node_id".to_string()),
                text: Err("no value supplied for text".to_string()),
            }
        }
    }
    impl Choice {
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
        pub fn metadata<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.metadata = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for metadata: {}", e));
            self
        }
        pub fn next_node_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::uuid::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.next_node_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for next_node_id: {}", e));
            self
        }
        pub fn text<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ChoiceText>,
            T::Error: ::std::fmt::Display,
        {
            self.text = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for text: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<Choice> for super::Choice {
        type Error = super::error::ConversionError;
        fn try_from(value: Choice) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                id: value.id?,
                metadata: value.metadata?,
                next_node_id: value.next_node_id?,
                text: value.text?,
            })
        }
    }
    impl ::std::convert::From<super::Choice> for Choice {
        fn from(value: super::Choice) -> Self {
            Self {
                id: Ok(value.id),
                metadata: Ok(value.metadata),
                next_node_id: Ok(value.next_node_id),
                text: Ok(value.text),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ConstraintResult {
        correction_capability:
            ::std::result::Result<super::CorrectionCapability, ::std::string::String>,
        corrections: ::std::result::Result<
            ::std::vec::Vec<super::CorrectionSuggestion>,
            ::std::string::String,
        >,
        missing_elements:
            ::std::result::Result<::std::vec::Vec<::std::string::String>, ::std::string::String>,
        required_elements_present: ::std::result::Result<bool, ::std::string::String>,
        theme_consistency_score: ::std::result::Result<f64, ::std::string::String>,
        vocabulary_violations: ::std::result::Result<
            ::std::vec::Vec<super::VocabularyViolation>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for ConstraintResult {
        fn default() -> Self {
            Self {
                correction_capability: Err(
                    "no value supplied for correction_capability".to_string()
                ),
                corrections: Err("no value supplied for corrections".to_string()),
                missing_elements: Err("no value supplied for missing_elements".to_string()),
                required_elements_present: Err(
                    "no value supplied for required_elements_present".to_string()
                ),
                theme_consistency_score: Err(
                    "no value supplied for theme_consistency_score".to_string()
                ),
                vocabulary_violations: Err(
                    "no value supplied for vocabulary_violations".to_string()
                ),
            }
        }
    }
    impl ConstraintResult {
        pub fn correction_capability<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CorrectionCapability>,
            T::Error: ::std::fmt::Display,
        {
            self.correction_capability = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for correction_capability: {}",
                    e
                )
            });
            self
        }
        pub fn corrections<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::CorrectionSuggestion>>,
            T::Error: ::std::fmt::Display,
        {
            self.corrections = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for corrections: {}", e));
            self
        }
        pub fn missing_elements<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.missing_elements = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for missing_elements: {}",
                    e
                )
            });
            self
        }
        pub fn required_elements_present<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.required_elements_present = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for required_elements_present: {}",
                    e
                )
            });
            self
        }
        pub fn theme_consistency_score<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.theme_consistency_score = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for theme_consistency_score: {}",
                    e
                )
            });
            self
        }
        pub fn vocabulary_violations<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::VocabularyViolation>>,
            T::Error: ::std::fmt::Display,
        {
            self.vocabulary_violations = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for vocabulary_violations: {}",
                    e
                )
            });
            self
        }
    }
    impl ::std::convert::TryFrom<ConstraintResult> for super::ConstraintResult {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ConstraintResult,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                correction_capability: value.correction_capability?,
                corrections: value.corrections?,
                missing_elements: value.missing_elements?,
                required_elements_present: value.required_elements_present?,
                theme_consistency_score: value.theme_consistency_score?,
                vocabulary_violations: value.vocabulary_violations?,
            })
        }
    }
    impl ::std::convert::From<super::ConstraintResult> for ConstraintResult {
        fn from(value: super::ConstraintResult) -> Self {
            Self {
                correction_capability: Ok(value.correction_capability),
                corrections: Ok(value.corrections),
                missing_elements: Ok(value.missing_elements),
                required_elements_present: Ok(value.required_elements_present),
                theme_consistency_score: Ok(value.theme_consistency_score),
                vocabulary_violations: Ok(value.vocabulary_violations),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Content {
        choices: ::std::result::Result<::std::vec::Vec<super::Choice>, ::std::string::String>,
        convergence_point: ::std::result::Result<bool, ::std::string::String>,
        educational_content: ::std::result::Result<
            ::std::option::Option<super::EducationalContent>,
            ::std::string::String,
        >,
        next_nodes: ::std::result::Result<::std::vec::Vec<::uuid::Uuid>, ::std::string::String>,
        node_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
        text: ::std::result::Result<super::ContentText, ::std::string::String>,
        type_: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for Content {
        fn default() -> Self {
            Self {
                choices: Err("no value supplied for choices".to_string()),
                convergence_point: Err("no value supplied for convergence_point".to_string()),
                educational_content: Ok(Default::default()),
                next_nodes: Err("no value supplied for next_nodes".to_string()),
                node_id: Err("no value supplied for node_id".to_string()),
                text: Err("no value supplied for text".to_string()),
                type_: Err("no value supplied for type_".to_string()),
            }
        }
    }
    impl Content {
        pub fn choices<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Choice>>,
            T::Error: ::std::fmt::Display,
        {
            self.choices = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for choices: {}", e));
            self
        }
        pub fn convergence_point<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.convergence_point = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for convergence_point: {}",
                    e
                )
            });
            self
        }
        pub fn educational_content<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::EducationalContent>>,
            T::Error: ::std::fmt::Display,
        {
            self.educational_content = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for educational_content: {}",
                    e
                )
            });
            self
        }
        pub fn next_nodes<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::uuid::Uuid>>,
            T::Error: ::std::fmt::Display,
        {
            self.next_nodes = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for next_nodes: {}", e));
            self
        }
        pub fn node_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::uuid::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.node_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for node_id: {}", e));
            self
        }
        pub fn text<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ContentText>,
            T::Error: ::std::fmt::Display,
        {
            self.text = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for text: {}", e));
            self
        }
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
    }
    impl ::std::convert::TryFrom<Content> for super::Content {
        type Error = super::error::ConversionError;
        fn try_from(value: Content) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                choices: value.choices?,
                convergence_point: value.convergence_point?,
                educational_content: value.educational_content?,
                next_nodes: value.next_nodes?,
                node_id: value.node_id?,
                text: value.text?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::Content> for Content {
        fn from(value: super::Content) -> Self {
            Self {
                choices: Ok(value.choices),
                convergence_point: Ok(value.convergence_point),
                educational_content: Ok(value.educational_content),
                next_nodes: Ok(value.next_nodes),
                node_id: Ok(value.node_id),
                text: Ok(value.text),
                type_: Ok(value.type_),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ContentNode {
        content: ::std::result::Result<super::Content, ::std::string::String>,
        generation_metadata: ::std::result::Result<
            ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            ::std::string::String,
        >,
        id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
        incoming_edges: ::std::result::Result<u64, ::std::string::String>,
        outgoing_edges: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for ContentNode {
        fn default() -> Self {
            Self {
                content: Err("no value supplied for content".to_string()),
                generation_metadata: Ok(Default::default()),
                id: Err("no value supplied for id".to_string()),
                incoming_edges: Err("no value supplied for incoming_edges".to_string()),
                outgoing_edges: Err("no value supplied for outgoing_edges".to_string()),
            }
        }
    }
    impl ContentNode {
        pub fn content<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Content>,
            T::Error: ::std::fmt::Display,
        {
            self.content = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for content: {}", e));
            self
        }
        pub fn generation_metadata<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.generation_metadata = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for generation_metadata: {}",
                    e
                )
            });
            self
        }
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
        pub fn incoming_edges<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.incoming_edges = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for incoming_edges: {}", e));
            self
        }
        pub fn outgoing_edges<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.outgoing_edges = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for outgoing_edges: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<ContentNode> for super::ContentNode {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ContentNode,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                content: value.content?,
                generation_metadata: value.generation_metadata?,
                id: value.id?,
                incoming_edges: value.incoming_edges?,
                outgoing_edges: value.outgoing_edges?,
            })
        }
    }
    impl ::std::convert::From<super::ContentNode> for ContentNode {
        fn from(value: super::ContentNode) -> Self {
            Self {
                content: Ok(value.content),
                generation_metadata: Ok(value.generation_metadata),
                id: Ok(value.id),
                incoming_edges: Ok(value.incoming_edges),
                outgoing_edges: Ok(value.outgoing_edges),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ContentReference {
        content: ::std::result::Result<super::Content, ::std::string::String>,
        temp_node_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
    }
    impl ::std::default::Default for ContentReference {
        fn default() -> Self {
            Self {
                content: Err("no value supplied for content".to_string()),
                temp_node_id: Err("no value supplied for temp_node_id".to_string()),
            }
        }
    }
    impl ContentReference {
        pub fn content<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Content>,
            T::Error: ::std::fmt::Display,
        {
            self.content = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for content: {}", e));
            self
        }
        pub fn temp_node_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::uuid::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.temp_node_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for temp_node_id: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<ContentReference> for super::ContentReference {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ContentReference,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                content: value.content?,
                temp_node_id: value.temp_node_id?,
            })
        }
    }
    impl ::std::convert::From<super::ContentReference> for ContentReference {
        fn from(value: super::ContentReference) -> Self {
            Self {
                content: Ok(value.content),
                temp_node_id: Ok(value.temp_node_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct CorrectionSuggestion {
        field: ::std::result::Result<::std::string::String, ::std::string::String>,
        issue: ::std::result::Result<::std::string::String, ::std::string::String>,
        severity: ::std::result::Result<super::CorrectionSuggestionSeverity, ::std::string::String>,
        suggestion: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for CorrectionSuggestion {
        fn default() -> Self {
            Self {
                field: Err("no value supplied for field".to_string()),
                issue: Err("no value supplied for issue".to_string()),
                severity: Err("no value supplied for severity".to_string()),
                suggestion: Err("no value supplied for suggestion".to_string()),
            }
        }
    }
    impl CorrectionSuggestion {
        pub fn field<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.field = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for field: {}", e));
            self
        }
        pub fn issue<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.issue = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for issue: {}", e));
            self
        }
        pub fn severity<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CorrectionSuggestionSeverity>,
            T::Error: ::std::fmt::Display,
        {
            self.severity = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for severity: {}", e));
            self
        }
        pub fn suggestion<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.suggestion = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for suggestion: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<CorrectionSuggestion> for super::CorrectionSuggestion {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CorrectionSuggestion,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                field: value.field?,
                issue: value.issue?,
                severity: value.severity?,
                suggestion: value.suggestion?,
            })
        }
    }
    impl ::std::convert::From<super::CorrectionSuggestion> for CorrectionSuggestion {
        fn from(value: super::CorrectionSuggestion) -> Self {
            Self {
                field: Ok(value.field),
                issue: Ok(value.issue),
                severity: Ok(value.severity),
                suggestion: Ok(value.suggestion),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct CorrectionSummary {
        attempts: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        correction_type:
            ::std::result::Result<super::CorrectionSummaryCorrectionType, ::std::string::String>,
        node_id: ::std::result::Result<::std::string::String, ::std::string::String>,
        success: ::std::result::Result<bool, ::std::string::String>,
    }
    impl ::std::default::Default for CorrectionSummary {
        fn default() -> Self {
            Self {
                attempts: Err("no value supplied for attempts".to_string()),
                correction_type: Err("no value supplied for correction_type".to_string()),
                node_id: Err("no value supplied for node_id".to_string()),
                success: Err("no value supplied for success".to_string()),
            }
        }
    }
    impl CorrectionSummary {
        pub fn attempts<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.attempts = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for attempts: {}", e));
            self
        }
        pub fn correction_type<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CorrectionSummaryCorrectionType>,
            T::Error: ::std::fmt::Display,
        {
            self.correction_type = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for correction_type: {}", e));
            self
        }
        pub fn node_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.node_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for node_id: {}", e));
            self
        }
        pub fn success<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.success = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for success: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<CorrectionSummary> for super::CorrectionSummary {
        type Error = super::error::ConversionError;
        fn try_from(
            value: CorrectionSummary,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                attempts: value.attempts?,
                correction_type: value.correction_type?,
                node_id: value.node_id?,
                success: value.success?,
            })
        }
    }
    impl ::std::convert::From<super::CorrectionSummary> for CorrectionSummary {
        fn from(value: super::CorrectionSummary) -> Self {
            Self {
                attempts: Ok(value.attempts),
                correction_type: Ok(value.correction_type),
                node_id: Ok(value.node_id),
                success: Ok(value.success),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Dag {
        convergence_points:
            ::std::result::Result<::std::vec::Vec<::uuid::Uuid>, ::std::string::String>,
        edges: ::std::result::Result<::std::vec::Vec<super::Edge>, ::std::string::String>,
        nodes: ::std::result::Result<
            ::std::collections::HashMap<::std::string::String, super::ContentNode>,
            ::std::string::String,
        >,
        start_node_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
    }
    impl ::std::default::Default for Dag {
        fn default() -> Self {
            Self {
                convergence_points: Err("no value supplied for convergence_points".to_string()),
                edges: Err("no value supplied for edges".to_string()),
                nodes: Err("no value supplied for nodes".to_string()),
                start_node_id: Err("no value supplied for start_node_id".to_string()),
            }
        }
    }
    impl Dag {
        pub fn convergence_points<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::uuid::Uuid>>,
            T::Error: ::std::fmt::Display,
        {
            self.convergence_points = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for convergence_points: {}",
                    e
                )
            });
            self
        }
        pub fn edges<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::Edge>>,
            T::Error: ::std::fmt::Display,
        {
            self.edges = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for edges: {}", e));
            self
        }
        pub fn nodes<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::std::collections::HashMap<::std::string::String, super::ContentNode>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.nodes = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for nodes: {}", e));
            self
        }
        pub fn start_node_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::uuid::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.start_node_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for start_node_id: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<Dag> for super::Dag {
        type Error = super::error::ConversionError;
        fn try_from(value: Dag) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                convergence_points: value.convergence_points?,
                edges: value.edges?,
                nodes: value.nodes?,
                start_node_id: value.start_node_id?,
            })
        }
    }
    impl ::std::convert::From<super::Dag> for Dag {
        fn from(value: super::Dag) -> Self {
            Self {
                convergence_points: Ok(value.convergence_points),
                edges: Ok(value.edges),
                nodes: Ok(value.nodes),
                start_node_id: Ok(value.start_node_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct DagStructureConfig {
        branching_factor: ::std::result::Result<i64, ::std::string::String>,
        convergence_pattern:
            ::std::result::Result<super::ConvergencePattern, ::std::string::String>,
        convergence_point_ratio:
            ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
        max_depth: ::std::result::Result<i64, ::std::string::String>,
        node_count: ::std::result::Result<i64, ::std::string::String>,
    }
    impl ::std::default::Default for DagStructureConfig {
        fn default() -> Self {
            Self {
                branching_factor: Err("no value supplied for branching_factor".to_string()),
                convergence_pattern: Err("no value supplied for convergence_pattern".to_string()),
                convergence_point_ratio: Ok(Default::default()),
                max_depth: Err("no value supplied for max_depth".to_string()),
                node_count: Err("no value supplied for node_count".to_string()),
            }
        }
    }
    impl DagStructureConfig {
        pub fn branching_factor<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.branching_factor = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for branching_factor: {}",
                    e
                )
            });
            self
        }
        pub fn convergence_pattern<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ConvergencePattern>,
            T::Error: ::std::fmt::Display,
        {
            self.convergence_pattern = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for convergence_pattern: {}",
                    e
                )
            });
            self
        }
        pub fn convergence_point_ratio<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<f64>>,
            T::Error: ::std::fmt::Display,
        {
            self.convergence_point_ratio = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for convergence_point_ratio: {}",
                    e
                )
            });
            self
        }
        pub fn max_depth<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.max_depth = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for max_depth: {}", e));
            self
        }
        pub fn node_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.node_count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for node_count: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<DagStructureConfig> for super::DagStructureConfig {
        type Error = super::error::ConversionError;
        fn try_from(
            value: DagStructureConfig,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                branching_factor: value.branching_factor?,
                convergence_pattern: value.convergence_pattern?,
                convergence_point_ratio: value.convergence_point_ratio?,
                max_depth: value.max_depth?,
                node_count: value.node_count?,
            })
        }
    }
    impl ::std::convert::From<super::DagStructureConfig> for DagStructureConfig {
        fn from(value: super::DagStructureConfig) -> Self {
            Self {
                branching_factor: Ok(value.branching_factor),
                convergence_pattern: Ok(value.convergence_pattern),
                convergence_point_ratio: Ok(value.convergence_point_ratio),
                max_depth: Ok(value.max_depth),
                node_count: Ok(value.node_count),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Edge {
        choice_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
        from_node_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
        to_node_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
        weight: ::std::result::Result<::std::option::Option<f64>, ::std::string::String>,
    }
    impl ::std::default::Default for Edge {
        fn default() -> Self {
            Self {
                choice_id: Err("no value supplied for choice_id".to_string()),
                from_node_id: Err("no value supplied for from_node_id".to_string()),
                to_node_id: Err("no value supplied for to_node_id".to_string()),
                weight: Ok(Default::default()),
            }
        }
    }
    impl Edge {
        pub fn choice_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::uuid::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.choice_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for choice_id: {}", e));
            self
        }
        pub fn from_node_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::uuid::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.from_node_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for from_node_id: {}", e));
            self
        }
        pub fn to_node_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::uuid::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.to_node_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for to_node_id: {}", e));
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
    impl ::std::convert::TryFrom<Edge> for super::Edge {
        type Error = super::error::ConversionError;
        fn try_from(value: Edge) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                choice_id: value.choice_id?,
                from_node_id: value.from_node_id?,
                to_node_id: value.to_node_id?,
                weight: value.weight?,
            })
        }
    }
    impl ::std::convert::From<super::Edge> for Edge {
        fn from(value: super::Edge) -> Self {
            Self {
                choice_id: Ok(value.choice_id),
                from_node_id: Ok(value.from_node_id),
                to_node_id: Ok(value.to_node_id),
                weight: Ok(value.weight),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct EducationalContent {
        educational_facts:
            ::std::result::Result<::std::vec::Vec<::std::string::String>, ::std::string::String>,
        learning_objective: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        topic: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        vocabulary_words:
            ::std::result::Result<::std::vec::Vec<::std::string::String>, ::std::string::String>,
    }
    impl ::std::default::Default for EducationalContent {
        fn default() -> Self {
            Self {
                educational_facts: Ok(Default::default()),
                learning_objective: Ok(Default::default()),
                topic: Ok(Default::default()),
                vocabulary_words: Ok(Default::default()),
            }
        }
    }
    impl EducationalContent {
        pub fn educational_facts<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.educational_facts = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for educational_facts: {}",
                    e
                )
            });
            self
        }
        pub fn learning_objective<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.learning_objective = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for learning_objective: {}",
                    e
                )
            });
            self
        }
        pub fn topic<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.topic = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for topic: {}", e));
            self
        }
        pub fn vocabulary_words<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.vocabulary_words = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for vocabulary_words: {}",
                    e
                )
            });
            self
        }
    }
    impl ::std::convert::TryFrom<EducationalContent> for super::EducationalContent {
        type Error = super::error::ConversionError;
        fn try_from(
            value: EducationalContent,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                educational_facts: value.educational_facts?,
                learning_objective: value.learning_objective?,
                topic: value.topic?,
                vocabulary_words: value.vocabulary_words?,
            })
        }
    }
    impl ::std::convert::From<super::EducationalContent> for EducationalContent {
        fn from(value: super::EducationalContent) -> Self {
            Self {
                educational_facts: Ok(value.educational_facts),
                learning_objective: Ok(value.learning_objective),
                topic: Ok(value.topic),
                vocabulary_words: Ok(value.vocabulary_words),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ExternalError {
        error_code: ::std::result::Result<::std::string::String, ::std::string::String>,
        error_message: ::std::result::Result<::std::string::String, ::std::string::String>,
        retry_possible: ::std::result::Result<::std::option::Option<bool>, ::std::string::String>,
        timestamp: ::std::result::Result<
            ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for ExternalError {
        fn default() -> Self {
            Self {
                error_code: Err("no value supplied for error_code".to_string()),
                error_message: Err("no value supplied for error_message".to_string()),
                retry_possible: Ok(Default::default()),
                timestamp: Ok(Default::default()),
            }
        }
    }
    impl ExternalError {
        pub fn error_code<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.error_code = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for error_code: {}", e));
            self
        }
        pub fn error_message<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.error_message = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for error_message: {}", e));
            self
        }
        pub fn retry_possible<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<bool>>,
            T::Error: ::std::fmt::Display,
        {
            self.retry_possible = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for retry_possible: {}", e));
            self
        }
        pub fn timestamp<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.timestamp = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for timestamp: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<ExternalError> for super::ExternalError {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ExternalError,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                error_code: value.error_code?,
                error_message: value.error_message?,
                retry_possible: value.retry_possible?,
                timestamp: value.timestamp?,
            })
        }
    }
    impl ::std::convert::From<super::ExternalError> for ExternalError {
        fn from(value: super::ExternalError) -> Self {
            Self {
                error_code: Ok(value.error_code),
                error_message: Ok(value.error_message),
                retry_possible: Ok(value.retry_possible),
                timestamp: Ok(value.timestamp),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ExternalGenerationRequestV1 {
        age_group: ::std::result::Result<super::AgeGroup, ::std::string::String>,
        language: ::std::result::Result<super::Language, ::std::string::String>,
        theme:
            ::std::result::Result<super::ExternalGenerationRequestV1Theme, ::std::string::String>,
    }
    impl ::std::default::Default for ExternalGenerationRequestV1 {
        fn default() -> Self {
            Self {
                age_group: Err("no value supplied for age_group".to_string()),
                language: Err("no value supplied for language".to_string()),
                theme: Err("no value supplied for theme".to_string()),
            }
        }
    }
    impl ExternalGenerationRequestV1 {
        pub fn age_group<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AgeGroup>,
            T::Error: ::std::fmt::Display,
        {
            self.age_group = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for age_group: {}", e));
            self
        }
        pub fn language<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Language>,
            T::Error: ::std::fmt::Display,
        {
            self.language = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for language: {}", e));
            self
        }
        pub fn theme<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ExternalGenerationRequestV1Theme>,
            T::Error: ::std::fmt::Display,
        {
            self.theme = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for theme: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<ExternalGenerationRequestV1> for super::ExternalGenerationRequestV1 {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ExternalGenerationRequestV1,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                age_group: value.age_group?,
                language: value.language?,
                theme: value.theme?,
            })
        }
    }
    impl ::std::convert::From<super::ExternalGenerationRequestV1> for ExternalGenerationRequestV1 {
        fn from(value: super::ExternalGenerationRequestV1) -> Self {
            Self {
                age_group: Ok(value.age_group),
                language: Ok(value.language),
                theme: Ok(value.theme),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ExternalGenerationResponseV1 {
        error: ::std::result::Result<
            ::std::option::Option<super::ExternalError>,
            ::std::string::String,
        >,
        job_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
        metadata: ::std::result::Result<
            ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            ::std::string::String,
        >,
        status:
            ::std::result::Result<super::ExternalGenerationResponseV1Status, ::std::string::String>,
        trail_data: ::std::result::Result<
            ::std::option::Option<super::TrailInsertData>,
            ::std::string::String,
        >,
        trail_steps_data: ::std::result::Result<
            ::std::vec::Vec<super::TrailStepInsertData>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for ExternalGenerationResponseV1 {
        fn default() -> Self {
            Self {
                error: Ok(Default::default()),
                job_id: Err("no value supplied for job_id".to_string()),
                metadata: Err("no value supplied for metadata".to_string()),
                status: Err("no value supplied for status".to_string()),
                trail_data: Ok(Default::default()),
                trail_steps_data: Ok(Default::default()),
            }
        }
    }
    impl ExternalGenerationResponseV1 {
        pub fn error<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::ExternalError>>,
            T::Error: ::std::fmt::Display,
        {
            self.error = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for error: {}", e));
            self
        }
        pub fn job_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::uuid::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.job_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for job_id: {}", e));
            self
        }
        pub fn metadata<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.metadata = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for metadata: {}", e));
            self
        }
        pub fn status<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ExternalGenerationResponseV1Status>,
            T::Error: ::std::fmt::Display,
        {
            self.status = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for status: {}", e));
            self
        }
        pub fn trail_data<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::TrailInsertData>>,
            T::Error: ::std::fmt::Display,
        {
            self.trail_data = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for trail_data: {}", e));
            self
        }
        pub fn trail_steps_data<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::TrailStepInsertData>>,
            T::Error: ::std::fmt::Display,
        {
            self.trail_steps_data = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for trail_steps_data: {}",
                    e
                )
            });
            self
        }
    }
    impl ::std::convert::TryFrom<ExternalGenerationResponseV1> for super::ExternalGenerationResponseV1 {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ExternalGenerationResponseV1,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                error: value.error?,
                job_id: value.job_id?,
                metadata: value.metadata?,
                status: value.status?,
                trail_data: value.trail_data?,
                trail_steps_data: value.trail_steps_data?,
            })
        }
    }
    impl ::std::convert::From<super::ExternalGenerationResponseV1> for ExternalGenerationResponseV1 {
        fn from(value: super::ExternalGenerationResponseV1) -> Self {
            Self {
                error: Ok(value.error),
                job_id: Ok(value.job_id),
                metadata: Ok(value.metadata),
                status: Ok(value.status),
                trail_data: Ok(value.trail_data),
                trail_steps_data: Ok(value.trail_steps_data),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ExternalJobStatus {
        current_phase: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        estimated_completion_seconds:
            ::std::result::Result<::std::option::Option<u64>, ::std::string::String>,
        job_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
        progress_percentage: ::std::result::Result<i64, ::std::string::String>,
        status: ::std::result::Result<super::ExternalJobStatusStatus, ::std::string::String>,
    }
    impl ::std::default::Default for ExternalJobStatus {
        fn default() -> Self {
            Self {
                current_phase: Ok(Default::default()),
                estimated_completion_seconds: Ok(Default::default()),
                job_id: Err("no value supplied for job_id".to_string()),
                progress_percentage: Err("no value supplied for progress_percentage".to_string()),
                status: Err("no value supplied for status".to_string()),
            }
        }
    }
    impl ExternalJobStatus {
        pub fn current_phase<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.current_phase = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for current_phase: {}", e));
            self
        }
        pub fn estimated_completion_seconds<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<u64>>,
            T::Error: ::std::fmt::Display,
        {
            self.estimated_completion_seconds = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for estimated_completion_seconds: {}",
                    e
                )
            });
            self
        }
        pub fn job_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::uuid::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.job_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for job_id: {}", e));
            self
        }
        pub fn progress_percentage<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.progress_percentage = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for progress_percentage: {}",
                    e
                )
            });
            self
        }
        pub fn status<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ExternalJobStatusStatus>,
            T::Error: ::std::fmt::Display,
        {
            self.status = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for status: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<ExternalJobStatus> for super::ExternalJobStatus {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ExternalJobStatus,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                current_phase: value.current_phase?,
                estimated_completion_seconds: value.estimated_completion_seconds?,
                job_id: value.job_id?,
                progress_percentage: value.progress_percentage?,
                status: value.status?,
            })
        }
    }
    impl ::std::convert::From<super::ExternalJobStatus> for ExternalJobStatus {
        fn from(value: super::ExternalJobStatus) -> Self {
            Self {
                current_phase: Ok(value.current_phase),
                estimated_completion_seconds: Ok(value.estimated_completion_seconds),
                job_id: Ok(value.job_id),
                progress_percentage: Ok(value.progress_percentage),
                status: Ok(value.status),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct GatewayMappingConfig {
        defaults: ::std::result::Result<super::MappingDefaults, ::std::string::String>,
        validation: ::std::result::Result<super::MappingValidation, ::std::string::String>,
    }
    impl ::std::default::Default for GatewayMappingConfig {
        fn default() -> Self {
            Self {
                defaults: Err("no value supplied for defaults".to_string()),
                validation: Err("no value supplied for validation".to_string()),
            }
        }
    }
    impl GatewayMappingConfig {
        pub fn defaults<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MappingDefaults>,
            T::Error: ::std::fmt::Display,
        {
            self.defaults = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for defaults: {}", e));
            self
        }
        pub fn validation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MappingValidation>,
            T::Error: ::std::fmt::Display,
        {
            self.validation = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for validation: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<GatewayMappingConfig> for super::GatewayMappingConfig {
        type Error = super::error::ConversionError;
        fn try_from(
            value: GatewayMappingConfig,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                defaults: value.defaults?,
                validation: value.validation?,
            })
        }
    }
    impl ::std::convert::From<super::GatewayMappingConfig> for GatewayMappingConfig {
        fn from(value: super::GatewayMappingConfig) -> Self {
            Self {
                defaults: Ok(value.defaults),
                validation: Ok(value.validation),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct GenerationError {
        error_code: ::std::result::Result<::std::string::String, ::std::string::String>,
        error_message: ::std::result::Result<::std::string::String, ::std::string::String>,
        retry_possible: ::std::result::Result<bool, ::std::string::String>,
        timestamp:
            ::std::result::Result<::chrono::DateTime<::chrono::offset::Utc>, ::std::string::String>,
    }
    impl ::std::default::Default for GenerationError {
        fn default() -> Self {
            Self {
                error_code: Err("no value supplied for error_code".to_string()),
                error_message: Err("no value supplied for error_message".to_string()),
                retry_possible: Err("no value supplied for retry_possible".to_string()),
                timestamp: Err("no value supplied for timestamp".to_string()),
            }
        }
    }
    impl GenerationError {
        pub fn error_code<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.error_code = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for error_code: {}", e));
            self
        }
        pub fn error_message<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.error_message = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for error_message: {}", e));
            self
        }
        pub fn retry_possible<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.retry_possible = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for retry_possible: {}", e));
            self
        }
        pub fn timestamp<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
            T::Error: ::std::fmt::Display,
        {
            self.timestamp = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for timestamp: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<GenerationError> for super::GenerationError {
        type Error = super::error::ConversionError;
        fn try_from(
            value: GenerationError,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                error_code: value.error_code?,
                error_message: value.error_message?,
                retry_possible: value.retry_possible?,
                timestamp: value.timestamp?,
            })
        }
    }
    impl ::std::convert::From<super::GenerationError> for GenerationError {
        fn from(value: super::GenerationError) -> Self {
            Self {
                error_code: Ok(value.error_code),
                error_message: Ok(value.error_message),
                retry_possible: Ok(value.retry_possible),
                timestamp: Ok(value.timestamp),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct GenerationMetadata {
        ai_model_version: ::std::result::Result<::std::string::String, ::std::string::String>,
        corrections_applied:
            ::std::result::Result<::std::vec::Vec<super::CorrectionSummary>, ::std::string::String>,
        generated_at:
            ::std::result::Result<::chrono::DateTime<::chrono::offset::Utc>, ::std::string::String>,
        generation_duration_seconds: ::std::result::Result<u64, ::std::string::String>,
        negotiation_rounds_executed: ::std::result::Result<i64, ::std::string::String>,
        orchestrator_version: ::std::result::Result<::std::string::String, ::std::string::String>,
        resolved_node_count: ::std::result::Result<i64, ::std::string::String>,
        total_word_count: ::std::result::Result<i64, ::std::string::String>,
        unresolved_validation_issues: ::std::result::Result<
            ::std::vec::Vec<super::ValidationIssueSummary>,
            ::std::string::String,
        >,
        validation_pass_rate: ::std::result::Result<f64, ::std::string::String>,
        validation_rounds: ::std::result::Result<i64, ::std::string::String>,
    }
    impl ::std::default::Default for GenerationMetadata {
        fn default() -> Self {
            Self {
                ai_model_version: Err("no value supplied for ai_model_version".to_string()),
                corrections_applied: Ok(Default::default()),
                generated_at: Err("no value supplied for generated_at".to_string()),
                generation_duration_seconds: Err(
                    "no value supplied for generation_duration_seconds".to_string(),
                ),
                negotiation_rounds_executed: Err(
                    "no value supplied for negotiation_rounds_executed".to_string(),
                ),
                orchestrator_version: Err("no value supplied for orchestrator_version".to_string()),
                resolved_node_count: Err("no value supplied for resolved_node_count".to_string()),
                total_word_count: Err("no value supplied for total_word_count".to_string()),
                unresolved_validation_issues: Ok(Default::default()),
                validation_pass_rate: Err("no value supplied for validation_pass_rate".to_string()),
                validation_rounds: Err("no value supplied for validation_rounds".to_string()),
            }
        }
    }
    impl GenerationMetadata {
        pub fn ai_model_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.ai_model_version = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for ai_model_version: {}",
                    e
                )
            });
            self
        }
        pub fn corrections_applied<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::CorrectionSummary>>,
            T::Error: ::std::fmt::Display,
        {
            self.corrections_applied = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for corrections_applied: {}",
                    e
                )
            });
            self
        }
        pub fn generated_at<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
            T::Error: ::std::fmt::Display,
        {
            self.generated_at = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for generated_at: {}", e));
            self
        }
        pub fn generation_duration_seconds<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.generation_duration_seconds = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for generation_duration_seconds: {}",
                    e
                )
            });
            self
        }
        pub fn negotiation_rounds_executed<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.negotiation_rounds_executed = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for negotiation_rounds_executed: {}",
                    e
                )
            });
            self
        }
        pub fn orchestrator_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.orchestrator_version = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for orchestrator_version: {}",
                    e
                )
            });
            self
        }
        pub fn resolved_node_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.resolved_node_count = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for resolved_node_count: {}",
                    e
                )
            });
            self
        }
        pub fn total_word_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.total_word_count = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for total_word_count: {}",
                    e
                )
            });
            self
        }
        pub fn unresolved_validation_issues<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::ValidationIssueSummary>>,
            T::Error: ::std::fmt::Display,
        {
            self.unresolved_validation_issues = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for unresolved_validation_issues: {}",
                    e
                )
            });
            self
        }
        pub fn validation_pass_rate<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.validation_pass_rate = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for validation_pass_rate: {}",
                    e
                )
            });
            self
        }
        pub fn validation_rounds<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.validation_rounds = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for validation_rounds: {}",
                    e
                )
            });
            self
        }
    }
    impl ::std::convert::TryFrom<GenerationMetadata> for super::GenerationMetadata {
        type Error = super::error::ConversionError;
        fn try_from(
            value: GenerationMetadata,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                ai_model_version: value.ai_model_version?,
                corrections_applied: value.corrections_applied?,
                generated_at: value.generated_at?,
                generation_duration_seconds: value.generation_duration_seconds?,
                negotiation_rounds_executed: value.negotiation_rounds_executed?,
                orchestrator_version: value.orchestrator_version?,
                resolved_node_count: value.resolved_node_count?,
                total_word_count: value.total_word_count?,
                unresolved_validation_issues: value.unresolved_validation_issues?,
                validation_pass_rate: value.validation_pass_rate?,
                validation_rounds: value.validation_rounds?,
            })
        }
    }
    impl ::std::convert::From<super::GenerationMetadata> for GenerationMetadata {
        fn from(value: super::GenerationMetadata) -> Self {
            Self {
                ai_model_version: Ok(value.ai_model_version),
                corrections_applied: Ok(value.corrections_applied),
                generated_at: Ok(value.generated_at),
                generation_duration_seconds: Ok(value.generation_duration_seconds),
                negotiation_rounds_executed: Ok(value.negotiation_rounds_executed),
                orchestrator_version: Ok(value.orchestrator_version),
                resolved_node_count: Ok(value.resolved_node_count),
                total_word_count: Ok(value.total_word_count),
                unresolved_validation_issues: Ok(value.unresolved_validation_issues),
                validation_pass_rate: Ok(value.validation_pass_rate),
                validation_rounds: Ok(value.validation_rounds),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct GenerationRequest {
        age_group: ::std::result::Result<super::AgeGroup, ::std::string::String>,
        author_id: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        dag_config: ::std::result::Result<
            ::std::option::Option<super::DagStructureConfig>,
            ::std::string::String,
        >,
        educational_goals:
            ::std::result::Result<::std::vec::Vec<::std::string::String>, ::std::string::String>,
        language: ::std::result::Result<super::Language, ::std::string::String>,
        node_count: ::std::result::Result<u8, ::std::string::String>,
        prompt_packages: ::std::result::Result<
            ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
            ::std::string::String,
        >,
        required_elements:
            ::std::result::Result<::std::vec::Vec<::std::string::String>, ::std::string::String>,
        story_structure: ::std::result::Result<
            ::std::option::Option<super::GenerationRequestStoryStructure>,
            ::std::string::String,
        >,
        tags: ::std::result::Result<
            ::std::vec::Vec<super::GenerationRequestTagsItem>,
            ::std::string::String,
        >,
        tenant_id: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        theme: ::std::result::Result<super::GenerationRequestTheme, ::std::string::String>,
        validation_policy: ::std::result::Result<
            ::std::option::Option<super::ValidationPolicy>,
            ::std::string::String,
        >,
        vocabulary_level: ::std::result::Result<super::VocabularyLevel, ::std::string::String>,
    }
    impl ::std::default::Default for GenerationRequest {
        fn default() -> Self {
            Self {
                age_group: Err("no value supplied for age_group".to_string()),
                author_id: Ok(Default::default()),
                dag_config: Ok(Default::default()),
                educational_goals: Ok(Default::default()),
                language: Err("no value supplied for language".to_string()),
                node_count: Ok(super::defaults::default_u64::<u8, 30>()),
                prompt_packages: Ok(Default::default()),
                required_elements: Ok(Default::default()),
                story_structure: Ok(Default::default()),
                tags: Ok(Default::default()),
                tenant_id: Err("no value supplied for tenant_id".to_string()),
                theme: Err("no value supplied for theme".to_string()),
                validation_policy: Ok(Default::default()),
                vocabulary_level: Ok(super::defaults::generation_request_vocabulary_level()),
            }
        }
    }
    impl GenerationRequest {
        pub fn age_group<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AgeGroup>,
            T::Error: ::std::fmt::Display,
        {
            self.age_group = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for age_group: {}", e));
            self
        }
        pub fn author_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.author_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for author_id: {}", e));
            self
        }
        pub fn dag_config<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::DagStructureConfig>>,
            T::Error: ::std::fmt::Display,
        {
            self.dag_config = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for dag_config: {}", e));
            self
        }
        pub fn educational_goals<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.educational_goals = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for educational_goals: {}",
                    e
                )
            });
            self
        }
        pub fn language<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Language>,
            T::Error: ::std::fmt::Display,
        {
            self.language = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for language: {}", e));
            self
        }
        pub fn node_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u8>,
            T::Error: ::std::fmt::Display,
        {
            self.node_count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for node_count: {}", e));
            self
        }
        pub fn prompt_packages<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::std::option::Option<
                    ::serde_json::Map<::std::string::String, ::serde_json::Value>,
                >,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.prompt_packages = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for prompt_packages: {}", e));
            self
        }
        pub fn required_elements<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.required_elements = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for required_elements: {}",
                    e
                )
            });
            self
        }
        pub fn story_structure<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::std::option::Option<super::GenerationRequestStoryStructure>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.story_structure = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for story_structure: {}", e));
            self
        }
        pub fn tags<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::GenerationRequestTagsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.tags = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for tags: {}", e));
            self
        }
        pub fn tenant_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.tenant_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for tenant_id: {}", e));
            self
        }
        pub fn theme<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::GenerationRequestTheme>,
            T::Error: ::std::fmt::Display,
        {
            self.theme = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for theme: {}", e));
            self
        }
        pub fn validation_policy<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::ValidationPolicy>>,
            T::Error: ::std::fmt::Display,
        {
            self.validation_policy = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for validation_policy: {}",
                    e
                )
            });
            self
        }
        pub fn vocabulary_level<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::VocabularyLevel>,
            T::Error: ::std::fmt::Display,
        {
            self.vocabulary_level = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for vocabulary_level: {}",
                    e
                )
            });
            self
        }
    }
    impl ::std::convert::TryFrom<GenerationRequest> for super::GenerationRequest {
        type Error = super::error::ConversionError;
        fn try_from(
            value: GenerationRequest,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                age_group: value.age_group?,
                author_id: value.author_id?,
                dag_config: value.dag_config?,
                educational_goals: value.educational_goals?,
                language: value.language?,
                node_count: value.node_count?,
                prompt_packages: value.prompt_packages?,
                required_elements: value.required_elements?,
                story_structure: value.story_structure?,
                tags: value.tags?,
                tenant_id: value.tenant_id?,
                theme: value.theme?,
                validation_policy: value.validation_policy?,
                vocabulary_level: value.vocabulary_level?,
            })
        }
    }
    impl ::std::convert::From<super::GenerationRequest> for GenerationRequest {
        fn from(value: super::GenerationRequest) -> Self {
            Self {
                age_group: Ok(value.age_group),
                author_id: Ok(value.author_id),
                dag_config: Ok(value.dag_config),
                educational_goals: Ok(value.educational_goals),
                language: Ok(value.language),
                node_count: Ok(value.node_count),
                prompt_packages: Ok(value.prompt_packages),
                required_elements: Ok(value.required_elements),
                story_structure: Ok(value.story_structure),
                tags: Ok(value.tags),
                tenant_id: Ok(value.tenant_id),
                theme: Ok(value.theme),
                validation_policy: Ok(value.validation_policy),
                vocabulary_level: Ok(value.vocabulary_level),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct GenerationResponse {
        errors:
            ::std::result::Result<::std::vec::Vec<super::GenerationError>, ::std::string::String>,
        execution_trace: ::std::result::Result<
            ::std::option::Option<super::PipelineExecutionTrace>,
            ::std::string::String,
        >,
        generation_metadata: ::std::result::Result<
            ::std::option::Option<super::GenerationMetadata>,
            ::std::string::String,
        >,
        progress_percentage: ::std::result::Result<i64, ::std::string::String>,
        prompt_generation_metadata: ::std::result::Result<
            ::std::option::Option<super::PromptGenerationSummary>,
            ::std::string::String,
        >,
        request_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
        status: ::std::result::Result<super::GenerationStatus, ::std::string::String>,
        trail: ::std::result::Result<::std::option::Option<super::Trail>, ::std::string::String>,
        trail_steps:
            ::std::result::Result<::std::vec::Vec<super::TrailStep>, ::std::string::String>,
    }
    impl ::std::default::Default for GenerationResponse {
        fn default() -> Self {
            Self {
                errors: Ok(Default::default()),
                execution_trace: Ok(Default::default()),
                generation_metadata: Ok(Default::default()),
                progress_percentage: Err("no value supplied for progress_percentage".to_string()),
                prompt_generation_metadata: Ok(Default::default()),
                request_id: Err("no value supplied for request_id".to_string()),
                status: Err("no value supplied for status".to_string()),
                trail: Ok(Default::default()),
                trail_steps: Ok(Default::default()),
            }
        }
    }
    impl GenerationResponse {
        pub fn errors<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::GenerationError>>,
            T::Error: ::std::fmt::Display,
        {
            self.errors = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for errors: {}", e));
            self
        }
        pub fn execution_trace<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::PipelineExecutionTrace>>,
            T::Error: ::std::fmt::Display,
        {
            self.execution_trace = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for execution_trace: {}", e));
            self
        }
        pub fn generation_metadata<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::GenerationMetadata>>,
            T::Error: ::std::fmt::Display,
        {
            self.generation_metadata = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for generation_metadata: {}",
                    e
                )
            });
            self
        }
        pub fn progress_percentage<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.progress_percentage = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for progress_percentage: {}",
                    e
                )
            });
            self
        }
        pub fn prompt_generation_metadata<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::PromptGenerationSummary>>,
            T::Error: ::std::fmt::Display,
        {
            self.prompt_generation_metadata = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for prompt_generation_metadata: {}",
                    e
                )
            });
            self
        }
        pub fn request_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::uuid::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.request_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for request_id: {}", e));
            self
        }
        pub fn status<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::GenerationStatus>,
            T::Error: ::std::fmt::Display,
        {
            self.status = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for status: {}", e));
            self
        }
        pub fn trail<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::Trail>>,
            T::Error: ::std::fmt::Display,
        {
            self.trail = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for trail: {}", e));
            self
        }
        pub fn trail_steps<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::TrailStep>>,
            T::Error: ::std::fmt::Display,
        {
            self.trail_steps = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for trail_steps: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<GenerationResponse> for super::GenerationResponse {
        type Error = super::error::ConversionError;
        fn try_from(
            value: GenerationResponse,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                errors: value.errors?,
                execution_trace: value.execution_trace?,
                generation_metadata: value.generation_metadata?,
                progress_percentage: value.progress_percentage?,
                prompt_generation_metadata: value.prompt_generation_metadata?,
                request_id: value.request_id?,
                status: value.status?,
                trail: value.trail?,
                trail_steps: value.trail_steps?,
            })
        }
    }
    impl ::std::convert::From<super::GenerationResponse> for GenerationResponse {
        fn from(value: super::GenerationResponse) -> Self {
            Self {
                errors: Ok(value.errors),
                execution_trace: Ok(value.execution_trace),
                generation_metadata: Ok(value.generation_metadata),
                progress_percentage: Ok(value.progress_percentage),
                prompt_generation_metadata: Ok(value.prompt_generation_metadata),
                request_id: Ok(value.request_id),
                status: Ok(value.status),
                trail: Ok(value.trail),
                trail_steps: Ok(value.trail_steps),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct LlmConfig {
        frequency_penalty: ::std::result::Result<f64, ::std::string::String>,
        max_tokens: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        presence_penalty: ::std::result::Result<f64, ::std::string::String>,
        stop_sequences:
            ::std::result::Result<::std::vec::Vec<::std::string::String>, ::std::string::String>,
        temperature: ::std::result::Result<f64, ::std::string::String>,
        top_p: ::std::result::Result<f64, ::std::string::String>,
    }
    impl ::std::default::Default for LlmConfig {
        fn default() -> Self {
            Self {
                frequency_penalty: Err("no value supplied for frequency_penalty".to_string()),
                max_tokens: Err("no value supplied for max_tokens".to_string()),
                presence_penalty: Err("no value supplied for presence_penalty".to_string()),
                stop_sequences: Err("no value supplied for stop_sequences".to_string()),
                temperature: Err("no value supplied for temperature".to_string()),
                top_p: Err("no value supplied for top_p".to_string()),
            }
        }
    }
    impl LlmConfig {
        pub fn frequency_penalty<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.frequency_penalty = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for frequency_penalty: {}",
                    e
                )
            });
            self
        }
        pub fn max_tokens<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.max_tokens = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for max_tokens: {}", e));
            self
        }
        pub fn presence_penalty<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.presence_penalty = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for presence_penalty: {}",
                    e
                )
            });
            self
        }
        pub fn stop_sequences<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.stop_sequences = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for stop_sequences: {}", e));
            self
        }
        pub fn temperature<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.temperature = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for temperature: {}", e));
            self
        }
        pub fn top_p<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.top_p = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for top_p: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<LlmConfig> for super::LlmConfig {
        type Error = super::error::ConversionError;
        fn try_from(
            value: LlmConfig,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                frequency_penalty: value.frequency_penalty?,
                max_tokens: value.max_tokens?,
                presence_penalty: value.presence_penalty?,
                stop_sequences: value.stop_sequences?,
                temperature: value.temperature?,
                top_p: value.top_p?,
            })
        }
    }
    impl ::std::convert::From<super::LlmConfig> for LlmConfig {
        fn from(value: super::LlmConfig) -> Self {
            Self {
                frequency_penalty: Ok(value.frequency_penalty),
                max_tokens: Ok(value.max_tokens),
                presence_penalty: Ok(value.presence_penalty),
                stop_sequences: Ok(value.stop_sequences),
                temperature: Ok(value.temperature),
                top_p: Ok(value.top_p),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MappingDefaults {
        educational_goals: ::std::result::Result<
            ::std::collections::HashMap<
                ::std::string::String,
                ::std::vec::Vec<::std::string::String>,
            >,
            ::std::string::String,
        >,
        node_count_12_14: ::std::result::Result<i64, ::std::string::String>,
        node_count_15_17: ::std::result::Result<i64, ::std::string::String>,
        node_count_18_plus: ::std::result::Result<i64, ::std::string::String>,
        node_count_6_8: ::std::result::Result<i64, ::std::string::String>,
        node_count_9_11: ::std::result::Result<i64, ::std::string::String>,
        required_elements: ::std::result::Result<
            ::std::collections::HashMap<
                ::std::string::String,
                ::std::vec::Vec<::std::string::String>,
            >,
            ::std::string::String,
        >,
        vocabulary_level_12_14: ::std::result::Result<::std::string::String, ::std::string::String>,
        vocabulary_level_15_17: ::std::result::Result<::std::string::String, ::std::string::String>,
        vocabulary_level_18_plus:
            ::std::result::Result<::std::string::String, ::std::string::String>,
        vocabulary_level_6_8: ::std::result::Result<::std::string::String, ::std::string::String>,
        vocabulary_level_9_11: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for MappingDefaults {
        fn default() -> Self {
            Self {
                educational_goals: Ok(Default::default()),
                node_count_12_14: Ok(super::defaults::default_u64::<i64, 16>()),
                node_count_15_17: Ok(super::defaults::default_u64::<i64, 24>()),
                node_count_18_plus: Ok(super::defaults::default_u64::<i64, 30>()),
                node_count_6_8: Ok(super::defaults::default_u64::<i64, 12>()),
                node_count_9_11: Ok(super::defaults::default_u64::<i64, 12>()),
                required_elements: Ok(Default::default()),
                vocabulary_level_12_14: Ok(
                    super::defaults::mapping_defaults_vocabulary_level_12_14(),
                ),
                vocabulary_level_15_17: Ok(
                    super::defaults::mapping_defaults_vocabulary_level_15_17(),
                ),
                vocabulary_level_18_plus: Ok(
                    super::defaults::mapping_defaults_vocabulary_level_18_plus(),
                ),
                vocabulary_level_6_8: Ok(super::defaults::mapping_defaults_vocabulary_level_6_8()),
                vocabulary_level_9_11: Ok(super::defaults::mapping_defaults_vocabulary_level_9_11()),
            }
        }
    }
    impl MappingDefaults {
        pub fn educational_goals<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::std::collections::HashMap<
                    ::std::string::String,
                    ::std::vec::Vec<::std::string::String>,
                >,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.educational_goals = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for educational_goals: {}",
                    e
                )
            });
            self
        }
        pub fn node_count_12_14<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.node_count_12_14 = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for node_count_12_14: {}",
                    e
                )
            });
            self
        }
        pub fn node_count_15_17<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.node_count_15_17 = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for node_count_15_17: {}",
                    e
                )
            });
            self
        }
        pub fn node_count_18_plus<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.node_count_18_plus = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for node_count_18_plus: {}",
                    e
                )
            });
            self
        }
        pub fn node_count_6_8<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.node_count_6_8 = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for node_count_6_8: {}", e));
            self
        }
        pub fn node_count_9_11<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.node_count_9_11 = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for node_count_9_11: {}", e));
            self
        }
        pub fn required_elements<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::std::collections::HashMap<
                    ::std::string::String,
                    ::std::vec::Vec<::std::string::String>,
                >,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.required_elements = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for required_elements: {}",
                    e
                )
            });
            self
        }
        pub fn vocabulary_level_12_14<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.vocabulary_level_12_14 = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for vocabulary_level_12_14: {}",
                    e
                )
            });
            self
        }
        pub fn vocabulary_level_15_17<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.vocabulary_level_15_17 = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for vocabulary_level_15_17: {}",
                    e
                )
            });
            self
        }
        pub fn vocabulary_level_18_plus<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.vocabulary_level_18_plus = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for vocabulary_level_18_plus: {}",
                    e
                )
            });
            self
        }
        pub fn vocabulary_level_6_8<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.vocabulary_level_6_8 = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for vocabulary_level_6_8: {}",
                    e
                )
            });
            self
        }
        pub fn vocabulary_level_9_11<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.vocabulary_level_9_11 = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for vocabulary_level_9_11: {}",
                    e
                )
            });
            self
        }
    }
    impl ::std::convert::TryFrom<MappingDefaults> for super::MappingDefaults {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MappingDefaults,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                educational_goals: value.educational_goals?,
                node_count_12_14: value.node_count_12_14?,
                node_count_15_17: value.node_count_15_17?,
                node_count_18_plus: value.node_count_18_plus?,
                node_count_6_8: value.node_count_6_8?,
                node_count_9_11: value.node_count_9_11?,
                required_elements: value.required_elements?,
                vocabulary_level_12_14: value.vocabulary_level_12_14?,
                vocabulary_level_15_17: value.vocabulary_level_15_17?,
                vocabulary_level_18_plus: value.vocabulary_level_18_plus?,
                vocabulary_level_6_8: value.vocabulary_level_6_8?,
                vocabulary_level_9_11: value.vocabulary_level_9_11?,
            })
        }
    }
    impl ::std::convert::From<super::MappingDefaults> for MappingDefaults {
        fn from(value: super::MappingDefaults) -> Self {
            Self {
                educational_goals: Ok(value.educational_goals),
                node_count_12_14: Ok(value.node_count_12_14),
                node_count_15_17: Ok(value.node_count_15_17),
                node_count_18_plus: Ok(value.node_count_18_plus),
                node_count_6_8: Ok(value.node_count_6_8),
                node_count_9_11: Ok(value.node_count_9_11),
                required_elements: Ok(value.required_elements),
                vocabulary_level_12_14: Ok(value.vocabulary_level_12_14),
                vocabulary_level_15_17: Ok(value.vocabulary_level_15_17),
                vocabulary_level_18_plus: Ok(value.vocabulary_level_18_plus),
                vocabulary_level_6_8: Ok(value.vocabulary_level_6_8),
                vocabulary_level_9_11: Ok(value.vocabulary_level_9_11),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MappingError {
        error_type: ::std::result::Result<super::MappingErrorErrorType, ::std::string::String>,
        message: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for MappingError {
        fn default() -> Self {
            Self {
                error_type: Err("no value supplied for error_type".to_string()),
                message: Err("no value supplied for message".to_string()),
            }
        }
    }
    impl MappingError {
        pub fn error_type<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::MappingErrorErrorType>,
            T::Error: ::std::fmt::Display,
        {
            self.error_type = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for error_type: {}", e));
            self
        }
        pub fn message<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.message = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for message: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<MappingError> for super::MappingError {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MappingError,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                error_type: value.error_type?,
                message: value.message?,
            })
        }
    }
    impl ::std::convert::From<super::MappingError> for MappingError {
        fn from(value: super::MappingError) -> Self {
            Self {
                error_type: Ok(value.error_type),
                message: Ok(value.message),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct MappingValidation {
        allowed_age_groups:
            ::std::result::Result<::std::vec::Vec<::std::string::String>, ::std::string::String>,
        allowed_languages:
            ::std::result::Result<::std::vec::Vec<::std::string::String>, ::std::string::String>,
        max_theme_length: ::std::result::Result<i64, ::std::string::String>,
        min_theme_length: ::std::result::Result<i64, ::std::string::String>,
    }
    impl ::std::default::Default for MappingValidation {
        fn default() -> Self {
            Self {
                allowed_age_groups: Ok(super::defaults::mapping_validation_allowed_age_groups()),
                allowed_languages: Ok(super::defaults::mapping_validation_allowed_languages()),
                max_theme_length: Ok(super::defaults::default_u64::<i64, 200>()),
                min_theme_length: Ok(super::defaults::default_u64::<i64, 5>()),
            }
        }
    }
    impl MappingValidation {
        pub fn allowed_age_groups<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.allowed_age_groups = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for allowed_age_groups: {}",
                    e
                )
            });
            self
        }
        pub fn allowed_languages<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.allowed_languages = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for allowed_languages: {}",
                    e
                )
            });
            self
        }
        pub fn max_theme_length<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.max_theme_length = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for max_theme_length: {}",
                    e
                )
            });
            self
        }
        pub fn min_theme_length<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.min_theme_length = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for min_theme_length: {}",
                    e
                )
            });
            self
        }
    }
    impl ::std::convert::TryFrom<MappingValidation> for super::MappingValidation {
        type Error = super::error::ConversionError;
        fn try_from(
            value: MappingValidation,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                allowed_age_groups: value.allowed_age_groups?,
                allowed_languages: value.allowed_languages?,
                max_theme_length: value.max_theme_length?,
                min_theme_length: value.min_theme_length?,
            })
        }
    }
    impl ::std::convert::From<super::MappingValidation> for MappingValidation {
        fn from(value: super::MappingValidation) -> Self {
            Self {
                allowed_age_groups: Ok(value.allowed_age_groups),
                allowed_languages: Ok(value.allowed_languages),
                max_theme_length: Ok(value.max_theme_length),
                min_theme_length: Ok(value.min_theme_length),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct NodeContext {
        incoming_edges: ::std::result::Result<u64, ::std::string::String>,
        is_convergence_point: ::std::result::Result<bool, ::std::string::String>,
        node_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
        node_position: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        previous_content: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for NodeContext {
        fn default() -> Self {
            Self {
                incoming_edges: Err("no value supplied for incoming_edges".to_string()),
                is_convergence_point: Err("no value supplied for is_convergence_point".to_string()),
                node_id: Err("no value supplied for node_id".to_string()),
                node_position: Err("no value supplied for node_position".to_string()),
                previous_content: Ok(Default::default()),
            }
        }
    }
    impl NodeContext {
        pub fn incoming_edges<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.incoming_edges = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for incoming_edges: {}", e));
            self
        }
        pub fn is_convergence_point<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.is_convergence_point = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for is_convergence_point: {}",
                    e
                )
            });
            self
        }
        pub fn node_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::uuid::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.node_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for node_id: {}", e));
            self
        }
        pub fn node_position<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.node_position = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for node_position: {}", e));
            self
        }
        pub fn previous_content<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.previous_content = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for previous_content: {}",
                    e
                )
            });
            self
        }
    }
    impl ::std::convert::TryFrom<NodeContext> for super::NodeContext {
        type Error = super::error::ConversionError;
        fn try_from(
            value: NodeContext,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                incoming_edges: value.incoming_edges?,
                is_convergence_point: value.is_convergence_point?,
                node_id: value.node_id?,
                node_position: value.node_position?,
                previous_content: value.previous_content?,
            })
        }
    }
    impl ::std::convert::From<super::NodeContext> for NodeContext {
        fn from(value: super::NodeContext) -> Self {
            Self {
                incoming_edges: Ok(value.incoming_edges),
                is_convergence_point: Ok(value.is_convergence_point),
                node_id: Ok(value.node_id),
                node_position: Ok(value.node_position),
                previous_content: Ok(value.previous_content),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct PipelineExecutionTrace {
        events_published: ::std::result::Result<
            ::std::vec::Vec<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
            ::std::string::String,
        >,
        phases_completed:
            ::std::result::Result<::std::vec::Vec<super::GenerationPhase>, ::std::string::String>,
        request_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
        service_invocations:
            ::std::result::Result<::std::vec::Vec<super::ServiceInvocation>, ::std::string::String>,
        total_duration_ms: ::std::result::Result<u64, ::std::string::String>,
    }
    impl ::std::default::Default for PipelineExecutionTrace {
        fn default() -> Self {
            Self {
                events_published: Ok(Default::default()),
                phases_completed: Err("no value supplied for phases_completed".to_string()),
                request_id: Err("no value supplied for request_id".to_string()),
                service_invocations: Err("no value supplied for service_invocations".to_string()),
                total_duration_ms: Err("no value supplied for total_duration_ms".to_string()),
            }
        }
    }
    impl PipelineExecutionTrace {
        pub fn events_published<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::std::vec::Vec<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.events_published = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for events_published: {}",
                    e
                )
            });
            self
        }
        pub fn phases_completed<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::GenerationPhase>>,
            T::Error: ::std::fmt::Display,
        {
            self.phases_completed = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for phases_completed: {}",
                    e
                )
            });
            self
        }
        pub fn request_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::uuid::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.request_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for request_id: {}", e));
            self
        }
        pub fn service_invocations<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::ServiceInvocation>>,
            T::Error: ::std::fmt::Display,
        {
            self.service_invocations = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for service_invocations: {}",
                    e
                )
            });
            self
        }
        pub fn total_duration_ms<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.total_duration_ms = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for total_duration_ms: {}",
                    e
                )
            });
            self
        }
    }
    impl ::std::convert::TryFrom<PipelineExecutionTrace> for super::PipelineExecutionTrace {
        type Error = super::error::ConversionError;
        fn try_from(
            value: PipelineExecutionTrace,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                events_published: value.events_published?,
                phases_completed: value.phases_completed?,
                request_id: value.request_id?,
                service_invocations: value.service_invocations?,
                total_duration_ms: value.total_duration_ms?,
            })
        }
    }
    impl ::std::convert::From<super::PipelineExecutionTrace> for PipelineExecutionTrace {
        fn from(value: super::PipelineExecutionTrace) -> Self {
            Self {
                events_published: Ok(value.events_published),
                phases_completed: Ok(value.phases_completed),
                request_id: Ok(value.request_id),
                service_invocations: Ok(value.service_invocations),
                total_duration_ms: Ok(value.total_duration_ms),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct PromptGenerationRequest {
        batch_info:
            ::std::result::Result<::std::option::Option<super::BatchInfo>, ::std::string::String>,
        generation_request: ::std::result::Result<super::GenerationRequest, ::std::string::String>,
        node_context:
            ::std::result::Result<::std::option::Option<super::NodeContext>, ::std::string::String>,
        service_target: ::std::result::Result<super::McpServiceType, ::std::string::String>,
    }
    impl ::std::default::Default for PromptGenerationRequest {
        fn default() -> Self {
            Self {
                batch_info: Ok(Default::default()),
                generation_request: Err("no value supplied for generation_request".to_string()),
                node_context: Ok(Default::default()),
                service_target: Err("no value supplied for service_target".to_string()),
            }
        }
    }
    impl PromptGenerationRequest {
        pub fn batch_info<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::BatchInfo>>,
            T::Error: ::std::fmt::Display,
        {
            self.batch_info = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for batch_info: {}", e));
            self
        }
        pub fn generation_request<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::GenerationRequest>,
            T::Error: ::std::fmt::Display,
        {
            self.generation_request = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for generation_request: {}",
                    e
                )
            });
            self
        }
        pub fn node_context<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::NodeContext>>,
            T::Error: ::std::fmt::Display,
        {
            self.node_context = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for node_context: {}", e));
            self
        }
        pub fn service_target<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::McpServiceType>,
            T::Error: ::std::fmt::Display,
        {
            self.service_target = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for service_target: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<PromptGenerationRequest> for super::PromptGenerationRequest {
        type Error = super::error::ConversionError;
        fn try_from(
            value: PromptGenerationRequest,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                batch_info: value.batch_info?,
                generation_request: value.generation_request?,
                node_context: value.node_context?,
                service_target: value.service_target?,
            })
        }
    }
    impl ::std::convert::From<super::PromptGenerationRequest> for PromptGenerationRequest {
        fn from(value: super::PromptGenerationRequest) -> Self {
            Self {
                batch_info: Ok(value.batch_info),
                generation_request: Ok(value.generation_request),
                node_context: Ok(value.node_context),
                service_target: Ok(value.service_target),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct PromptGenerationSummary {
        fallback_count: ::std::result::Result<u64, ::std::string::String>,
        llm_generated_count: ::std::result::Result<u64, ::std::string::String>,
        prompt_generation_duration_ms: ::std::result::Result<u64, ::std::string::String>,
        prompts_generated_at:
            ::std::result::Result<::chrono::DateTime<::chrono::offset::Utc>, ::std::string::String>,
        prompts_used: ::std::result::Result<
            ::std::collections::HashMap<::std::string::String, super::PromptPackage>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for PromptGenerationSummary {
        fn default() -> Self {
            Self {
                fallback_count: Err("no value supplied for fallback_count".to_string()),
                llm_generated_count: Err("no value supplied for llm_generated_count".to_string()),
                prompt_generation_duration_ms: Err(
                    "no value supplied for prompt_generation_duration_ms".to_string(),
                ),
                prompts_generated_at: Err("no value supplied for prompts_generated_at".to_string()),
                prompts_used: Err("no value supplied for prompts_used".to_string()),
            }
        }
    }
    impl PromptGenerationSummary {
        pub fn fallback_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.fallback_count = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for fallback_count: {}", e));
            self
        }
        pub fn llm_generated_count<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.llm_generated_count = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for llm_generated_count: {}",
                    e
                )
            });
            self
        }
        pub fn prompt_generation_duration_ms<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.prompt_generation_duration_ms = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for prompt_generation_duration_ms: {}",
                    e
                )
            });
            self
        }
        pub fn prompts_generated_at<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
            T::Error: ::std::fmt::Display,
        {
            self.prompts_generated_at = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for prompts_generated_at: {}",
                    e
                )
            });
            self
        }
        pub fn prompts_used<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::std::collections::HashMap<::std::string::String, super::PromptPackage>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.prompts_used = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for prompts_used: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<PromptGenerationSummary> for super::PromptGenerationSummary {
        type Error = super::error::ConversionError;
        fn try_from(
            value: PromptGenerationSummary,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                fallback_count: value.fallback_count?,
                llm_generated_count: value.llm_generated_count?,
                prompt_generation_duration_ms: value.prompt_generation_duration_ms?,
                prompts_generated_at: value.prompts_generated_at?,
                prompts_used: value.prompts_used?,
            })
        }
    }
    impl ::std::convert::From<super::PromptGenerationSummary> for PromptGenerationSummary {
        fn from(value: super::PromptGenerationSummary) -> Self {
            Self {
                fallback_count: Ok(value.fallback_count),
                llm_generated_count: Ok(value.llm_generated_count),
                prompt_generation_duration_ms: Ok(value.prompt_generation_duration_ms),
                prompts_generated_at: Ok(value.prompts_generated_at),
                prompts_used: Ok(value.prompts_used),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct PromptMetadata {
        age_group_context: ::std::result::Result<super::AgeGroup, ::std::string::String>,
        generated_at:
            ::std::result::Result<::chrono::DateTime<::chrono::offset::Utc>, ::std::string::String>,
        generation_method:
            ::std::result::Result<super::PromptGenerationMethod, ::std::string::String>,
        language_context: ::std::result::Result<super::Language, ::std::string::String>,
        service_target: ::std::result::Result<super::McpServiceType, ::std::string::String>,
        template_version: ::std::result::Result<::std::string::String, ::std::string::String>,
        theme_context: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for PromptMetadata {
        fn default() -> Self {
            Self {
                age_group_context: Err("no value supplied for age_group_context".to_string()),
                generated_at: Err("no value supplied for generated_at".to_string()),
                generation_method: Err("no value supplied for generation_method".to_string()),
                language_context: Err("no value supplied for language_context".to_string()),
                service_target: Err("no value supplied for service_target".to_string()),
                template_version: Err("no value supplied for template_version".to_string()),
                theme_context: Err("no value supplied for theme_context".to_string()),
            }
        }
    }
    impl PromptMetadata {
        pub fn age_group_context<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AgeGroup>,
            T::Error: ::std::fmt::Display,
        {
            self.age_group_context = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for age_group_context: {}",
                    e
                )
            });
            self
        }
        pub fn generated_at<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
            T::Error: ::std::fmt::Display,
        {
            self.generated_at = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for generated_at: {}", e));
            self
        }
        pub fn generation_method<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::PromptGenerationMethod>,
            T::Error: ::std::fmt::Display,
        {
            self.generation_method = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for generation_method: {}",
                    e
                )
            });
            self
        }
        pub fn language_context<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Language>,
            T::Error: ::std::fmt::Display,
        {
            self.language_context = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for language_context: {}",
                    e
                )
            });
            self
        }
        pub fn service_target<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::McpServiceType>,
            T::Error: ::std::fmt::Display,
        {
            self.service_target = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for service_target: {}", e));
            self
        }
        pub fn template_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.template_version = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for template_version: {}",
                    e
                )
            });
            self
        }
        pub fn theme_context<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.theme_context = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for theme_context: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<PromptMetadata> for super::PromptMetadata {
        type Error = super::error::ConversionError;
        fn try_from(
            value: PromptMetadata,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                age_group_context: value.age_group_context?,
                generated_at: value.generated_at?,
                generation_method: value.generation_method?,
                language_context: value.language_context?,
                service_target: value.service_target?,
                template_version: value.template_version?,
                theme_context: value.theme_context?,
            })
        }
    }
    impl ::std::convert::From<super::PromptMetadata> for PromptMetadata {
        fn from(value: super::PromptMetadata) -> Self {
            Self {
                age_group_context: Ok(value.age_group_context),
                generated_at: Ok(value.generated_at),
                generation_method: Ok(value.generation_method),
                language_context: Ok(value.language_context),
                service_target: Ok(value.service_target),
                template_version: Ok(value.template_version),
                theme_context: Ok(value.theme_context),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct PromptPackage {
        fallback_used: ::std::result::Result<bool, ::std::string::String>,
        language: ::std::result::Result<super::Language, ::std::string::String>,
        llm_config: ::std::result::Result<super::LlmConfig, ::std::string::String>,
        llm_model: ::std::result::Result<::std::string::String, ::std::string::String>,
        prompt_metadata: ::std::result::Result<super::PromptMetadata, ::std::string::String>,
        system_prompt: ::std::result::Result<::std::string::String, ::std::string::String>,
        user_prompt: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for PromptPackage {
        fn default() -> Self {
            Self {
                fallback_used: Err("no value supplied for fallback_used".to_string()),
                language: Err("no value supplied for language".to_string()),
                llm_config: Err("no value supplied for llm_config".to_string()),
                llm_model: Err("no value supplied for llm_model".to_string()),
                prompt_metadata: Err("no value supplied for prompt_metadata".to_string()),
                system_prompt: Err("no value supplied for system_prompt".to_string()),
                user_prompt: Err("no value supplied for user_prompt".to_string()),
            }
        }
    }
    impl PromptPackage {
        pub fn fallback_used<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.fallback_used = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for fallback_used: {}", e));
            self
        }
        pub fn language<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Language>,
            T::Error: ::std::fmt::Display,
        {
            self.language = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for language: {}", e));
            self
        }
        pub fn llm_config<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::LlmConfig>,
            T::Error: ::std::fmt::Display,
        {
            self.llm_config = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for llm_config: {}", e));
            self
        }
        pub fn llm_model<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.llm_model = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for llm_model: {}", e));
            self
        }
        pub fn prompt_metadata<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::PromptMetadata>,
            T::Error: ::std::fmt::Display,
        {
            self.prompt_metadata = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for prompt_metadata: {}", e));
            self
        }
        pub fn system_prompt<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.system_prompt = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for system_prompt: {}", e));
            self
        }
        pub fn user_prompt<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.user_prompt = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for user_prompt: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<PromptPackage> for super::PromptPackage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: PromptPackage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                fallback_used: value.fallback_used?,
                language: value.language?,
                llm_config: value.llm_config?,
                llm_model: value.llm_model?,
                prompt_metadata: value.prompt_metadata?,
                system_prompt: value.system_prompt?,
                user_prompt: value.user_prompt?,
            })
        }
    }
    impl ::std::convert::From<super::PromptPackage> for PromptPackage {
        fn from(value: super::PromptPackage) -> Self {
            Self {
                fallback_used: Ok(value.fallback_used),
                language: Ok(value.language),
                llm_config: Ok(value.llm_config),
                llm_model: Ok(value.llm_model),
                prompt_metadata: Ok(value.prompt_metadata),
                system_prompt: Ok(value.system_prompt),
                user_prompt: Ok(value.user_prompt),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ServiceInvocation {
        batch_id: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
        duration_ms: ::std::result::Result<u64, ::std::string::String>,
        error_message: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        node_id: ::std::result::Result<::std::option::Option<::uuid::Uuid>, ::std::string::String>,
        phase: ::std::result::Result<super::GenerationPhase, ::std::string::String>,
        service_name: ::std::result::Result<::std::string::String, ::std::string::String>,
        started_at:
            ::std::result::Result<::chrono::DateTime<::chrono::offset::Utc>, ::std::string::String>,
        success: ::std::result::Result<bool, ::std::string::String>,
        tool_name: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for ServiceInvocation {
        fn default() -> Self {
            Self {
                batch_id: Ok(Default::default()),
                duration_ms: Err("no value supplied for duration_ms".to_string()),
                error_message: Ok(Default::default()),
                node_id: Ok(Default::default()),
                phase: Err("no value supplied for phase".to_string()),
                service_name: Err("no value supplied for service_name".to_string()),
                started_at: Err("no value supplied for started_at".to_string()),
                success: Err("no value supplied for success".to_string()),
                tool_name: Err("no value supplied for tool_name".to_string()),
            }
        }
    }
    impl ServiceInvocation {
        pub fn batch_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<i64>>,
            T::Error: ::std::fmt::Display,
        {
            self.batch_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for batch_id: {}", e));
            self
        }
        pub fn duration_ms<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<u64>,
            T::Error: ::std::fmt::Display,
        {
            self.duration_ms = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for duration_ms: {}", e));
            self
        }
        pub fn error_message<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.error_message = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for error_message: {}", e));
            self
        }
        pub fn node_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::uuid::Uuid>>,
            T::Error: ::std::fmt::Display,
        {
            self.node_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for node_id: {}", e));
            self
        }
        pub fn phase<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::GenerationPhase>,
            T::Error: ::std::fmt::Display,
        {
            self.phase = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for phase: {}", e));
            self
        }
        pub fn service_name<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.service_name = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for service_name: {}", e));
            self
        }
        pub fn started_at<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::chrono::DateTime<::chrono::offset::Utc>>,
            T::Error: ::std::fmt::Display,
        {
            self.started_at = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for started_at: {}", e));
            self
        }
        pub fn success<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.success = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for success: {}", e));
            self
        }
        pub fn tool_name<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.tool_name = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for tool_name: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<ServiceInvocation> for super::ServiceInvocation {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ServiceInvocation,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                batch_id: value.batch_id?,
                duration_ms: value.duration_ms?,
                error_message: value.error_message?,
                node_id: value.node_id?,
                phase: value.phase?,
                service_name: value.service_name?,
                started_at: value.started_at?,
                success: value.success?,
                tool_name: value.tool_name?,
            })
        }
    }
    impl ::std::convert::From<super::ServiceInvocation> for ServiceInvocation {
        fn from(value: super::ServiceInvocation) -> Self {
            Self {
                batch_id: Ok(value.batch_id),
                duration_ms: Ok(value.duration_ms),
                error_message: Ok(value.error_message),
                node_id: Ok(value.node_id),
                phase: Ok(value.phase),
                service_name: Ok(value.service_name),
                started_at: Ok(value.started_at),
                success: Ok(value.success),
                tool_name: Ok(value.tool_name),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TaleTrailCustomMetadata {
        batch_id: ::std::result::Result<::std::option::Option<::uuid::Uuid>, ::std::string::String>,
        correlation_id:
            ::std::result::Result<::std::option::Option<::uuid::Uuid>, ::std::string::String>,
        generation_phase: ::std::result::Result<
            ::std::option::Option<super::GenerationPhase>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for TaleTrailCustomMetadata {
        fn default() -> Self {
            Self {
                batch_id: Ok(Default::default()),
                correlation_id: Ok(Default::default()),
                generation_phase: Ok(Default::default()),
            }
        }
    }
    impl TaleTrailCustomMetadata {
        pub fn batch_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::uuid::Uuid>>,
            T::Error: ::std::fmt::Display,
        {
            self.batch_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for batch_id: {}", e));
            self
        }
        pub fn correlation_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::uuid::Uuid>>,
            T::Error: ::std::fmt::Display,
        {
            self.correlation_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for correlation_id: {}", e));
            self
        }
        pub fn generation_phase<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::GenerationPhase>>,
            T::Error: ::std::fmt::Display,
        {
            self.generation_phase = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for generation_phase: {}",
                    e
                )
            });
            self
        }
    }
    impl ::std::convert::TryFrom<TaleTrailCustomMetadata> for super::TaleTrailCustomMetadata {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TaleTrailCustomMetadata,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                batch_id: value.batch_id?,
                correlation_id: value.correlation_id?,
                generation_phase: value.generation_phase?,
            })
        }
    }
    impl ::std::convert::From<super::TaleTrailCustomMetadata> for TaleTrailCustomMetadata {
        fn from(value: super::TaleTrailCustomMetadata) -> Self {
            Self {
                batch_id: Ok(value.batch_id),
                correlation_id: Ok(value.correlation_id),
                generation_phase: Ok(value.generation_phase),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct Trail {
        category: ::std::result::Result<
            ::std::option::Option<::std::string::String>,
            ::std::string::String,
        >,
        description: ::std::result::Result<
            ::std::option::Option<super::TrailDescription>,
            ::std::string::String,
        >,
        is_public: ::std::result::Result<bool, ::std::string::String>,
        metadata: ::std::result::Result<
            ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            ::std::string::String,
        >,
        price_coins: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
        status: ::std::result::Result<super::TrailStatus, ::std::string::String>,
        tags: ::std::result::Result<::std::vec::Vec<::std::string::String>, ::std::string::String>,
        title: ::std::result::Result<super::TrailTitle, ::std::string::String>,
    }
    impl ::std::default::Default for Trail {
        fn default() -> Self {
            Self {
                category: Ok(Default::default()),
                description: Ok(Default::default()),
                is_public: Err("no value supplied for is_public".to_string()),
                metadata: Err("no value supplied for metadata".to_string()),
                price_coins: Ok(Default::default()),
                status: Err("no value supplied for status".to_string()),
                tags: Ok(Default::default()),
                title: Err("no value supplied for title".to_string()),
            }
        }
    }
    impl Trail {
        pub fn category<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.category = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for category: {}", e));
            self
        }
        pub fn description<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::TrailDescription>>,
            T::Error: ::std::fmt::Display,
        {
            self.description = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for description: {}", e));
            self
        }
        pub fn is_public<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.is_public = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for is_public: {}", e));
            self
        }
        pub fn metadata<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.metadata = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for metadata: {}", e));
            self
        }
        pub fn price_coins<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<i64>>,
            T::Error: ::std::fmt::Display,
        {
            self.price_coins = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for price_coins: {}", e));
            self
        }
        pub fn status<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TrailStatus>,
            T::Error: ::std::fmt::Display,
        {
            self.status = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for status: {}", e));
            self
        }
        pub fn tags<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.tags = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for tags: {}", e));
            self
        }
        pub fn title<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TrailTitle>,
            T::Error: ::std::fmt::Display,
        {
            self.title = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for title: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<Trail> for super::Trail {
        type Error = super::error::ConversionError;
        fn try_from(value: Trail) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                category: value.category?,
                description: value.description?,
                is_public: value.is_public?,
                metadata: value.metadata?,
                price_coins: value.price_coins?,
                status: value.status?,
                tags: value.tags?,
                title: value.title?,
            })
        }
    }
    impl ::std::convert::From<super::Trail> for Trail {
        fn from(value: super::Trail) -> Self {
            Self {
                category: Ok(value.category),
                description: Ok(value.description),
                is_public: Ok(value.is_public),
                metadata: Ok(value.metadata),
                price_coins: Ok(value.price_coins),
                status: Ok(value.status),
                tags: Ok(value.tags),
                title: Ok(value.title),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TrailInsertData {
        description: ::std::result::Result<
            ::std::option::Option<super::TrailInsertDataDescription>,
            ::std::string::String,
        >,
        is_public: ::std::result::Result<bool, ::std::string::String>,
        metadata: ::std::result::Result<
            ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            ::std::string::String,
        >,
        price_coins: ::std::result::Result<::std::option::Option<i64>, ::std::string::String>,
        status: ::std::result::Result<super::TrailStatus, ::std::string::String>,
        tags: ::std::result::Result<
            ::std::vec::Vec<super::TrailInsertDataTagsItem>,
            ::std::string::String,
        >,
        title: ::std::result::Result<super::TrailInsertDataTitle, ::std::string::String>,
    }
    impl ::std::default::Default for TrailInsertData {
        fn default() -> Self {
            Self {
                description: Ok(Default::default()),
                is_public: Err("no value supplied for is_public".to_string()),
                metadata: Err("no value supplied for metadata".to_string()),
                price_coins: Ok(Default::default()),
                status: Err("no value supplied for status".to_string()),
                tags: Ok(Default::default()),
                title: Err("no value supplied for title".to_string()),
            }
        }
    }
    impl TrailInsertData {
        pub fn description<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::TrailInsertDataDescription>>,
            T::Error: ::std::fmt::Display,
        {
            self.description = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for description: {}", e));
            self
        }
        pub fn is_public<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.is_public = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for is_public: {}", e));
            self
        }
        pub fn metadata<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.metadata = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for metadata: {}", e));
            self
        }
        pub fn price_coins<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<i64>>,
            T::Error: ::std::fmt::Display,
        {
            self.price_coins = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for price_coins: {}", e));
            self
        }
        pub fn status<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TrailStatus>,
            T::Error: ::std::fmt::Display,
        {
            self.status = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for status: {}", e));
            self
        }
        pub fn tags<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::TrailInsertDataTagsItem>>,
            T::Error: ::std::fmt::Display,
        {
            self.tags = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for tags: {}", e));
            self
        }
        pub fn title<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TrailInsertDataTitle>,
            T::Error: ::std::fmt::Display,
        {
            self.title = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for title: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<TrailInsertData> for super::TrailInsertData {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TrailInsertData,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                description: value.description?,
                is_public: value.is_public?,
                metadata: value.metadata?,
                price_coins: value.price_coins?,
                status: value.status?,
                tags: value.tags?,
                title: value.title?,
            })
        }
    }
    impl ::std::convert::From<super::TrailInsertData> for TrailInsertData {
        fn from(value: super::TrailInsertData) -> Self {
            Self {
                description: Ok(value.description),
                is_public: Ok(value.is_public),
                metadata: Ok(value.metadata),
                price_coins: Ok(value.price_coins),
                status: Ok(value.status),
                tags: Ok(value.tags),
                title: Ok(value.title),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TrailStep {
        content_reference: ::std::result::Result<super::ContentReference, ::std::string::String>,
        description: ::std::result::Result<
            ::std::option::Option<super::TrailStepDescription>,
            ::std::string::String,
        >,
        is_required: ::std::result::Result<bool, ::std::string::String>,
        metadata: ::std::result::Result<
            ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            ::std::string::String,
        >,
        step_order: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        title: ::std::result::Result<
            ::std::option::Option<super::TrailStepTitle>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for TrailStep {
        fn default() -> Self {
            Self {
                content_reference: Err("no value supplied for content_reference".to_string()),
                description: Ok(Default::default()),
                is_required: Err("no value supplied for is_required".to_string()),
                metadata: Err("no value supplied for metadata".to_string()),
                step_order: Err("no value supplied for step_order".to_string()),
                title: Ok(Default::default()),
            }
        }
    }
    impl TrailStep {
        pub fn content_reference<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ContentReference>,
            T::Error: ::std::fmt::Display,
        {
            self.content_reference = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for content_reference: {}",
                    e
                )
            });
            self
        }
        pub fn description<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::TrailStepDescription>>,
            T::Error: ::std::fmt::Display,
        {
            self.description = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for description: {}", e));
            self
        }
        pub fn is_required<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.is_required = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for is_required: {}", e));
            self
        }
        pub fn metadata<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.metadata = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for metadata: {}", e));
            self
        }
        pub fn step_order<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.step_order = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for step_order: {}", e));
            self
        }
        pub fn title<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::TrailStepTitle>>,
            T::Error: ::std::fmt::Display,
        {
            self.title = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for title: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<TrailStep> for super::TrailStep {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TrailStep,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                content_reference: value.content_reference?,
                description: value.description?,
                is_required: value.is_required?,
                metadata: value.metadata?,
                step_order: value.step_order?,
                title: value.title?,
            })
        }
    }
    impl ::std::convert::From<super::TrailStep> for TrailStep {
        fn from(value: super::TrailStep) -> Self {
            Self {
                content_reference: Ok(value.content_reference),
                description: Ok(value.description),
                is_required: Ok(value.is_required),
                metadata: Ok(value.metadata),
                step_order: Ok(value.step_order),
                title: Ok(value.title),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TrailStepInsertData {
        content_data: ::std::result::Result<
            ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            ::std::string::String,
        >,
        description: ::std::result::Result<
            ::std::option::Option<super::TrailStepInsertDataDescription>,
            ::std::string::String,
        >,
        is_required: ::std::result::Result<bool, ::std::string::String>,
        metadata: ::std::result::Result<
            ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            ::std::string::String,
        >,
        step_order: ::std::result::Result<::std::num::NonZeroU64, ::std::string::String>,
        title: ::std::result::Result<
            ::std::option::Option<super::TrailStepInsertDataTitle>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for TrailStepInsertData {
        fn default() -> Self {
            Self {
                content_data: Err("no value supplied for content_data".to_string()),
                description: Ok(Default::default()),
                is_required: Err("no value supplied for is_required".to_string()),
                metadata: Err("no value supplied for metadata".to_string()),
                step_order: Err("no value supplied for step_order".to_string()),
                title: Ok(Default::default()),
            }
        }
    }
    impl TrailStepInsertData {
        pub fn content_data<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.content_data = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for content_data: {}", e));
            self
        }
        pub fn description<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::std::option::Option<super::TrailStepInsertDataDescription>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.description = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for description: {}", e));
            self
        }
        pub fn is_required<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.is_required = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for is_required: {}", e));
            self
        }
        pub fn metadata<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::serde_json::Map<::std::string::String, ::serde_json::Value>,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.metadata = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for metadata: {}", e));
            self
        }
        pub fn step_order<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::num::NonZeroU64>,
            T::Error: ::std::fmt::Display,
        {
            self.step_order = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for step_order: {}", e));
            self
        }
        pub fn title<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::TrailStepInsertDataTitle>>,
            T::Error: ::std::fmt::Display,
        {
            self.title = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for title: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<TrailStepInsertData> for super::TrailStepInsertData {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TrailStepInsertData,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                content_data: value.content_data?,
                description: value.description?,
                is_required: value.is_required?,
                metadata: value.metadata?,
                step_order: value.step_order?,
                title: value.title?,
            })
        }
    }
    impl ::std::convert::From<super::TrailStepInsertData> for TrailStepInsertData {
        fn from(value: super::TrailStepInsertData) -> Self {
            Self {
                content_data: Ok(value.content_data),
                description: Ok(value.description),
                is_required: Ok(value.is_required),
                metadata: Ok(value.metadata),
                step_order: Ok(value.step_order),
                title: Ok(value.title),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ValidationIssueSummary {
        description: ::std::result::Result<::std::string::String, ::std::string::String>,
        issue_type: ::std::result::Result<::std::string::String, ::std::string::String>,
        node_id: ::std::result::Result<::std::string::String, ::std::string::String>,
        severity:
            ::std::result::Result<super::ValidationIssueSummarySeverity, ::std::string::String>,
    }
    impl ::std::default::Default for ValidationIssueSummary {
        fn default() -> Self {
            Self {
                description: Err("no value supplied for description".to_string()),
                issue_type: Err("no value supplied for issue_type".to_string()),
                node_id: Err("no value supplied for node_id".to_string()),
                severity: Err("no value supplied for severity".to_string()),
            }
        }
    }
    impl ValidationIssueSummary {
        pub fn description<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.description = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for description: {}", e));
            self
        }
        pub fn issue_type<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.issue_type = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for issue_type: {}", e));
            self
        }
        pub fn node_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.node_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for node_id: {}", e));
            self
        }
        pub fn severity<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ValidationIssueSummarySeverity>,
            T::Error: ::std::fmt::Display,
        {
            self.severity = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for severity: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<ValidationIssueSummary> for super::ValidationIssueSummary {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ValidationIssueSummary,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                description: value.description?,
                issue_type: value.issue_type?,
                node_id: value.node_id?,
                severity: value.severity?,
            })
        }
    }
    impl ::std::convert::From<super::ValidationIssueSummary> for ValidationIssueSummary {
        fn from(value: super::ValidationIssueSummary) -> Self {
            Self {
                description: Ok(value.description),
                issue_type: Ok(value.issue_type),
                node_id: Ok(value.node_id),
                severity: Ok(value.severity),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ValidationPolicy {
        custom_restricted_words: ::std::result::Result<
            ::std::collections::HashMap<
                ::std::string::String,
                ::std::vec::Vec<::std::string::String>,
            >,
            ::std::string::String,
        >,
        enable_validation: ::std::result::Result<bool, ::std::string::String>,
        merge_mode: ::std::result::Result<
            ::std::option::Option<super::RestrictedWordsMergeMode>,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for ValidationPolicy {
        fn default() -> Self {
            Self {
                custom_restricted_words: Ok(Default::default()),
                enable_validation: Ok(super::defaults::default_bool::<true>()),
                merge_mode: Ok(Default::default()),
            }
        }
    }
    impl ValidationPolicy {
        pub fn custom_restricted_words<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<
                ::std::collections::HashMap<
                    ::std::string::String,
                    ::std::vec::Vec<::std::string::String>,
                >,
            >,
            T::Error: ::std::fmt::Display,
        {
            self.custom_restricted_words = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for custom_restricted_words: {}",
                    e
                )
            });
            self
        }
        pub fn enable_validation<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.enable_validation = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for enable_validation: {}",
                    e
                )
            });
            self
        }
        pub fn merge_mode<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::RestrictedWordsMergeMode>>,
            T::Error: ::std::fmt::Display,
        {
            self.merge_mode = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for merge_mode: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<ValidationPolicy> for super::ValidationPolicy {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ValidationPolicy,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                custom_restricted_words: value.custom_restricted_words?,
                enable_validation: value.enable_validation?,
                merge_mode: value.merge_mode?,
            })
        }
    }
    impl ::std::convert::From<super::ValidationPolicy> for ValidationPolicy {
        fn from(value: super::ValidationPolicy) -> Self {
            Self {
                custom_restricted_words: Ok(value.custom_restricted_words),
                enable_validation: Ok(value.enable_validation),
                merge_mode: Ok(value.merge_mode),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct ValidationResult {
        age_appropriate_score: ::std::result::Result<f64, ::std::string::String>,
        correction_capability:
            ::std::result::Result<super::CorrectionCapability, ::std::string::String>,
        corrections: ::std::result::Result<
            ::std::vec::Vec<super::CorrectionSuggestion>,
            ::std::string::String,
        >,
        educational_value_score: ::std::result::Result<f64, ::std::string::String>,
        is_valid: ::std::result::Result<bool, ::std::string::String>,
        safety_issues:
            ::std::result::Result<::std::vec::Vec<::std::string::String>, ::std::string::String>,
    }
    impl ::std::default::Default for ValidationResult {
        fn default() -> Self {
            Self {
                age_appropriate_score: Err(
                    "no value supplied for age_appropriate_score".to_string()
                ),
                correction_capability: Err(
                    "no value supplied for correction_capability".to_string()
                ),
                corrections: Err("no value supplied for corrections".to_string()),
                educational_value_score: Err(
                    "no value supplied for educational_value_score".to_string()
                ),
                is_valid: Err("no value supplied for is_valid".to_string()),
                safety_issues: Err("no value supplied for safety_issues".to_string()),
            }
        }
    }
    impl ValidationResult {
        pub fn age_appropriate_score<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.age_appropriate_score = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for age_appropriate_score: {}",
                    e
                )
            });
            self
        }
        pub fn correction_capability<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::CorrectionCapability>,
            T::Error: ::std::fmt::Display,
        {
            self.correction_capability = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for correction_capability: {}",
                    e
                )
            });
            self
        }
        pub fn corrections<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::CorrectionSuggestion>>,
            T::Error: ::std::fmt::Display,
        {
            self.corrections = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for corrections: {}", e));
            self
        }
        pub fn educational_value_score<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<f64>,
            T::Error: ::std::fmt::Display,
        {
            self.educational_value_score = value.try_into().map_err(|e| {
                format!(
                    "error converting supplied value for educational_value_score: {}",
                    e
                )
            });
            self
        }
        pub fn is_valid<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.is_valid = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for is_valid: {}", e));
            self
        }
        pub fn safety_issues<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.safety_issues = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for safety_issues: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<ValidationResult> for super::ValidationResult {
        type Error = super::error::ConversionError;
        fn try_from(
            value: ValidationResult,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                age_appropriate_score: value.age_appropriate_score?,
                correction_capability: value.correction_capability?,
                corrections: value.corrections?,
                educational_value_score: value.educational_value_score?,
                is_valid: value.is_valid?,
                safety_issues: value.safety_issues?,
            })
        }
    }
    impl ::std::convert::From<super::ValidationResult> for ValidationResult {
        fn from(value: super::ValidationResult) -> Self {
            Self {
                age_appropriate_score: Ok(value.age_appropriate_score),
                correction_capability: Ok(value.correction_capability),
                corrections: Ok(value.corrections),
                educational_value_score: Ok(value.educational_value_score),
                is_valid: Ok(value.is_valid),
                safety_issues: Ok(value.safety_issues),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct VocabularyViolation {
        current_level: ::std::result::Result<super::VocabularyLevel, ::std::string::String>,
        node_id: ::std::result::Result<::uuid::Uuid, ::std::string::String>,
        suggestions:
            ::std::result::Result<::std::vec::Vec<::std::string::String>, ::std::string::String>,
        target_level: ::std::result::Result<super::VocabularyLevel, ::std::string::String>,
        word: ::std::result::Result<::std::string::String, ::std::string::String>,
    }
    impl ::std::default::Default for VocabularyViolation {
        fn default() -> Self {
            Self {
                current_level: Err("no value supplied for current_level".to_string()),
                node_id: Err("no value supplied for node_id".to_string()),
                suggestions: Err("no value supplied for suggestions".to_string()),
                target_level: Err("no value supplied for target_level".to_string()),
                word: Err("no value supplied for word".to_string()),
            }
        }
    }
    impl VocabularyViolation {
        pub fn current_level<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::VocabularyLevel>,
            T::Error: ::std::fmt::Display,
        {
            self.current_level = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for current_level: {}", e));
            self
        }
        pub fn node_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::uuid::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.node_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for node_id: {}", e));
            self
        }
        pub fn suggestions<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
            T::Error: ::std::fmt::Display,
        {
            self.suggestions = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for suggestions: {}", e));
            self
        }
        pub fn target_level<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::VocabularyLevel>,
            T::Error: ::std::fmt::Display,
        {
            self.target_level = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for target_level: {}", e));
            self
        }
        pub fn word<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::string::String>,
            T::Error: ::std::fmt::Display,
        {
            self.word = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for word: {}", e));
            self
        }
    }
    impl ::std::convert::TryFrom<VocabularyViolation> for super::VocabularyViolation {
        type Error = super::error::ConversionError;
        fn try_from(
            value: VocabularyViolation,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                current_level: value.current_level?,
                node_id: value.node_id?,
                suggestions: value.suggestions?,
                target_level: value.target_level?,
                word: value.word?,
            })
        }
    }
    impl ::std::convert::From<super::VocabularyViolation> for VocabularyViolation {
        fn from(value: super::VocabularyViolation) -> Self {
            Self {
                current_level: Ok(value.current_level),
                node_id: Ok(value.node_id),
                suggestions: Ok(value.suggestions),
                target_level: Ok(value.target_level),
                word: Ok(value.word),
            }
        }
    }
}
#[doc = r" Generation of default values for serde."]
pub mod defaults {
    pub(super) fn default_bool<const V: bool>() -> bool {
        V
    }
    pub(super) fn default_u64<T, const V: u64>() -> T
    where
        T: ::std::convert::TryFrom<u64>,
        <T as ::std::convert::TryFrom<u64>>::Error: ::std::fmt::Debug,
    {
        T::try_from(V).unwrap()
    }
    pub(super) fn generation_request_vocabulary_level() -> super::VocabularyLevel {
        super::VocabularyLevel::Basic
    }
    pub(super) fn mapping_defaults_vocabulary_level_12_14() -> ::std::string::String {
        "intermediate".to_string()
    }
    pub(super) fn mapping_defaults_vocabulary_level_15_17() -> ::std::string::String {
        "intermediate".to_string()
    }
    pub(super) fn mapping_defaults_vocabulary_level_18_plus() -> ::std::string::String {
        "advanced".to_string()
    }
    pub(super) fn mapping_defaults_vocabulary_level_6_8() -> ::std::string::String {
        "basic".to_string()
    }
    pub(super) fn mapping_defaults_vocabulary_level_9_11() -> ::std::string::String {
        "basic".to_string()
    }
    pub(super) fn mapping_validation_allowed_age_groups() -> ::std::vec::Vec<::std::string::String>
    {
        vec![
            "6-8".to_string(),
            "9-11".to_string(),
            "12-14".to_string(),
            "15-17".to_string(),
            "+18".to_string(),
        ]
    }
    pub(super) fn mapping_validation_allowed_languages() -> ::std::vec::Vec<::std::string::String> {
        vec!["de".to_string(), "en".to_string()]
    }
}
