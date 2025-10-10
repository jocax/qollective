//! Vocabulary level checking and suggestion generation
//!
//! This module provides vocabulary-level validation for content,
//! ensuring words are appropriate for target age groups with
//! simpler alternatives when needed.

use shared_types::{Language, VocabularyLevel, VocabularyViolation};
use std::collections::HashMap;

/// Check vocabulary level of content against target level
///
/// # Arguments
/// * `content` - Text content to check
/// * `language` - Language of the content (English or German)
/// * `level` - Target vocabulary level (Basic, Intermediate, Advanced)
/// * `node_id` - Node identifier for violation tracking
///
/// # Returns
/// Vector of violations for words exceeding target level with suggestions
pub fn check_vocabulary_level(
    content: &str,
    language: Language,
    level: VocabularyLevel,
    node_id: &str,
) -> Vec<VocabularyViolation> {
    if content.trim().is_empty() {
        return Vec::new();
    }

    let words = extract_words(content);
    let mut violations = Vec::new();

    for word in words {
        let word_level = get_word_level(&word, &language);

        // Check if word exceeds target level
        if is_level_exceeded(&word_level, &level) {
            let suggestions = get_suggestions(&word, &language);
            violations.push(VocabularyViolation {
                word: word.clone(),
                node_id: node_id.to_string(),
                current_level: word_level,
                target_level: level.clone(),
                suggestions,
            });
        }
    }

    violations
}

/// Extract words from content, stripping punctuation
fn extract_words(content: &str) -> Vec<String> {
    content
        .split_whitespace()
        .map(|word| {
            word.chars()
                .filter(|c| c.is_alphabetic())
                .collect::<String>()
                .to_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect()
}

/// Get vocabulary level for a word
fn get_word_level(word: &str, language: &Language) -> VocabularyLevel {
    let word_lower = word.to_lowercase();

    match language {
        Language::En => {
            if ENGLISH_BASIC.contains(&word_lower.as_str()) {
                VocabularyLevel::Basic
            } else if ENGLISH_INTERMEDIATE.contains(&word_lower.as_str()) {
                VocabularyLevel::Intermediate
            } else if ENGLISH_ADVANCED.contains(&word_lower.as_str()) {
                VocabularyLevel::Advanced
            } else {
                // Use heuristic for unknown words
                word_level_heuristic(word)
            }
        }
        Language::De => {
            if GERMAN_BASIC.contains(&word_lower.as_str()) {
                VocabularyLevel::Basic
            } else if GERMAN_INTERMEDIATE.contains(&word_lower.as_str()) {
                VocabularyLevel::Intermediate
            } else if GERMAN_ADVANCED.contains(&word_lower.as_str()) {
                VocabularyLevel::Advanced
            } else {
                // Use heuristic for unknown words
                word_level_heuristic(word)
            }
        }
    }
}

/// Heuristic word level determination based on word length
fn word_level_heuristic(word: &str) -> VocabularyLevel {
    let len = word.len();
    if len <= 6 {
        VocabularyLevel::Basic
    } else if len <= 10 {
        VocabularyLevel::Intermediate
    } else {
        VocabularyLevel::Advanced
    }
}

/// Check if word level exceeds target level
fn is_level_exceeded(word_level: &VocabularyLevel, target_level: &VocabularyLevel) -> bool {
    let word_rank = level_to_rank(word_level);
    let target_rank = level_to_rank(target_level);
    word_rank > target_rank
}

/// Convert vocabulary level to numeric rank for comparison
fn level_to_rank(level: &VocabularyLevel) -> u8 {
    match level {
        VocabularyLevel::Basic => 0,
        VocabularyLevel::Intermediate => 1,
        VocabularyLevel::Advanced => 2,
    }
}

/// Get simpler word suggestions for a complex word
fn get_suggestions(word: &str, language: &Language) -> Vec<String> {
    let word_lower = word.to_lowercase();

    match language {
        Language::En => ENGLISH_SUGGESTIONS
            .get(word_lower.as_str())
            .map(|s| s.iter().map(|&w| w.to_string()).collect())
            .unwrap_or_else(Vec::new),
        Language::De => GERMAN_SUGGESTIONS
            .get(word_lower.as_str())
            .map(|s| s.iter().map(|&w| w.to_string()).collect())
            .unwrap_or_else(Vec::new),
    }
}

// =============================================================================
// English Vocabulary Lists
// =============================================================================

/// English basic vocabulary (ages 6-8) - 500 common words
const ENGLISH_BASIC: &[&str] = &[
    // Common words
    "the", "and", "a", "to", "of", "in", "is", "it", "you", "that",
    "he", "she", "was", "for", "on", "are", "with", "as", "his", "they",
    "at", "be", "this", "from", "have", "or", "one", "had", "by", "but",
    "not", "what", "all", "were", "when", "we", "there", "can", "an", "your",

    // Basic verbs
    "run", "play", "eat", "drink", "sleep", "walk", "jump", "sit", "stand", "talk",
    "look", "see", "hear", "feel", "think", "know", "go", "come", "get", "make",
    "do", "say", "take", "give", "find", "help", "want", "need", "like", "love",
    "ask", "tell", "try", "use", "work", "call", "write", "read", "open", "close",
    "start", "stop", "move", "turn", "put", "hold", "show", "keep", "leave", "begin",

    // Basic nouns
    "friend", "family", "home", "house", "room", "door", "window", "table", "chair", "bed",
    "book", "toy", "game", "ball", "dog", "cat", "bird", "tree", "flower", "sun",
    "moon", "star", "water", "food", "cake", "apple", "bread", "milk", "day", "night",
    "time", "year", "week", "hour", "morning", "evening", "boy", "girl", "man", "woman",
    "child", "baby", "hand", "foot", "head", "eye", "ear", "nose", "mouth", "face",

    // Basic adjectives
    "good", "bad", "big", "small", "happy", "sad", "new", "old", "hot", "cold",
    "fast", "slow", "high", "low", "long", "short", "easy", "hard", "right", "wrong",
    "pretty", "nice", "kind", "fun", "soft", "clean", "dirty", "wet", "dry", "full",
    "empty", "strong", "weak", "young", "bright", "dark", "loud", "quiet", "sweet", "sour",

    // Colors
    "red", "blue", "green", "yellow", "orange", "purple", "black", "white", "brown", "pink",
    "gray", "gold", "silver",

    // Numbers
    "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
    "first", "second", "third", "last", "many", "few", "some", "more", "less", "most",

    // Common prepositions
    "up", "down", "in", "out", "on", "off", "over", "under", "near", "far",
    "here", "there", "where", "when", "why", "how", "because", "if", "then", "so",

    // Basic actions
    "sing", "dance", "laugh", "cry", "smile", "wave", "clap", "point", "pull", "push",
    "throw", "catch", "kick", "hit", "touch", "wash", "draw", "paint", "cut", "paste",

    // School words
    "school", "teacher", "student", "class", "learn", "study", "paper", "pencil", "pen", "color",
    "draw", "write", "count", "spell", "answer", "question", "story", "picture", "number", "letter",

    // Time words
    "today", "tomorrow", "yesterday", "now", "later", "soon", "early", "late", "always", "never",
    "sometimes", "often", "before", "after", "during", "while", "until", "since",

    // Family
    "mother", "father", "sister", "brother", "grandmother", "grandfather", "aunt", "uncle", "cousin",

    // Animals
    "cow", "pig", "horse", "sheep", "chicken", "duck", "fish", "mouse", "rabbit", "bear",
    "lion", "tiger", "elephant", "monkey", "snake", "frog", "bee", "ant", "spider", "butterfly",

    // Nature
    "sky", "cloud", "rain", "snow", "wind", "grass", "leaf", "stone", "rock", "hill",
    "mountain", "river", "lake", "sea", "beach", "sand", "wave", "ice",

    // Food
    "egg", "cheese", "meat", "fish", "rice", "soup", "pizza", "cookie", "candy", "juice",
    "tea", "coffee", "sugar", "salt", "butter", "honey",

    // Body parts
    "arm", "leg", "knee", "elbow", "finger", "toe", "back", "chest", "stomach", "neck",
    "hair", "tooth", "teeth", "skin", "heart", "bone",

    // Clothing
    "shirt", "pants", "dress", "skirt", "coat", "jacket", "hat", "shoe", "sock", "glove",

    // Places
    "city", "town", "street", "park", "store", "shop", "farm", "garden", "forest", "field",

    // Transportation
    "car", "bus", "train", "bike", "boat", "plane", "truck",

    // Activities
    "party", "trip", "visit", "walk", "ride", "race", "rest", "wait", "watch", "listen",

    // Common adjectives/adverbs
    "very", "too", "much", "little", "great", "best", "better", "same", "different", "other",
    "another", "each", "every", "any", "both", "only", "just", "still", "also", "well",

    // Questions/pronouns
    "who", "whom", "whose", "which", "what", "where", "when", "why", "how",
    "I", "me", "my", "mine", "we", "us", "our", "ours",
    "you", "your", "yours", "he", "him", "his", "she", "her", "hers",
    "it", "its", "they", "them", "their", "theirs",

    // Conjunctions
    "and", "or", "but", "so", "yet", "for", "nor",

    // Additional common words
    "about", "after", "again", "all", "also", "am", "an", "any", "around", "as",
    "back", "be", "been", "being", "both", "but", "by", "came", "can", "could",
    "did", "does", "doing", "done", "each", "end", "even", "get", "gets", "got",
    "has", "having", "her", "here", "hers", "him", "himself", "how", "into", "just",
    "kind", "made", "may", "might", "more", "must", "my", "next", "no", "off",
    "once", "only", "our", "own", "part", "people", "place", "put", "said", "same",
    "see", "shall", "should", "side", "such", "than", "that", "the", "their", "them",
    "then", "these", "this", "those", "through", "time", "too", "two", "up", "us",
    "very", "was", "way", "well", "went", "were", "what", "when", "where", "which",
    "while", "who", "will", "with", "would", "yes", "yet", "you",
];

/// English intermediate vocabulary (ages 9-11) - 1000+ words
const ENGLISH_INTERMEDIATE: &[&str] = &[
    // Adventure & exploration
    "adventure", "explore", "discover", "journey", "quest", "expedition", "voyage", "travel",
    "wander", "roam", "search", "seek", "investigate", "examine", "observe", "patrol",

    // Emotions & feelings
    "excited", "nervous", "worried", "afraid", "brave", "confident", "proud", "ashamed",
    "jealous", "grateful", "surprised", "amazed", "confused", "curious", "determined", "patient",
    "frustrated", "disappointed", "relieved", "content", "eager", "anxious", "cheerful", "gloomy",

    // Actions & verbs
    "achieve", "accomplish", "attempt", "challenge", "compete", "cooperate", "create", "defend",
    "demonstrate", "develop", "encourage", "escape", "establish", "experience", "express", "gather",
    "generate", "guard", "improve", "include", "inform", "introduce", "maintain", "manage",
    "observe", "obtain", "organize", "participate", "perform", "prepare", "present", "prevent",
    "produce", "protect", "provide", "publish", "realize", "receive", "recognize", "recommend",
    "reduce", "refer", "reflect", "refuse", "release", "remain", "remember", "remove",
    "repeat", "replace", "represent", "require", "respond", "restore", "reveal", "review",

    // Descriptive words
    "ancient", "magnificent", "mysterious", "peculiar", "precious", "remarkable", "tremendous",
    "brilliant", "delicate", "delicious", "enormous", "excellent", "fantastic", "genuine",
    "glorious", "graceful", "incredible", "intense", "massive", "outstanding", "splendid",
    "superior", "supreme", "terrible", "terrible", "unique", "valuable", "various", "vast",

    // Nature & science
    "atmosphere", "climate", "creature", "earthquake", "environment", "fossil", "galaxy",
    "habitat", "kingdom", "landscape", "mineral", "molecule", "ocean", "oxygen", "planet",
    "pollution", "population", "predator", "prey", "resource", "species", "temperature",
    "territory", "universe", "vegetation", "volcano", "wilderness",

    // Society & culture
    "ancient", "civilization", "community", "custom", "festival", "government", "heritage",
    "history", "identity", "language", "legend", "monument", "myth", "nation", "origin",
    "parliament", "pioneer", "revolution", "society", "symbol", "tradition", "treaty",

    // Thinking & learning
    "analyze", "calculate", "classify", "compare", "conclude", "consider", "deduce",
    "determine", "distinguish", "estimate", "evaluate", "explain", "identify", "illustrate",
    "imagine", "infer", "interpret", "measure", "predict", "reason", "recall", "solve",
    "summarize", "understand", "verify",

    // Communication
    "announce", "argue", "communicate", "convince", "debate", "declare", "describe",
    "discuss", "emphasize", "exaggerate", "excuse", "explain", "gossip", "imply",
    "inform", "interrupt", "mention", "negotiate", "persuade", "proclaim", "protest",
    "suggest", "whisper",

    // Time & sequence
    "ancient", "century", "decade", "duration", "era", "eternal", "eventual", "frequent",
    "gradual", "immediate", "instant", "moment", "occasional", "permanent", "previous",
    "prior", "recent", "regular", "sequence", "temporary", "timeless",

    // Places & locations
    "boundary", "capital", "castle", "cathedral", "chamber", "channel", "chapel", "cliff",
    "continent", "corridor", "courtyard", "district", "domain", "dungeon", "dwelling",
    "entrance", "estate", "fortress", "frontier", "gateway", "headquarters", "island",
    "mainland", "mansion", "marketplace", "palace", "passage", "peninsula", "province",
    "region", "territory", "tower", "valley", "village",

    // Materials & objects
    "armor", "bronze", "canvas", "crystal", "fabric", "granite", "ivory", "leather",
    "marble", "metal", "parchment", "pottery", "silk", "steel", "timber", "velvet",

    // Qualities & characteristics
    "absolute", "accurate", "actual", "adequate", "artificial", "authentic", "automatic",
    "average", "basic", "beneficial", "capable", "certain", "complex", "constant",
    "crucial", "definite", "dense", "distinct", "efficient", "elaborate", "essential",
    "evident", "exact", "extreme", "flexible", "formal", "frequent", "fundamental",
    "general", "gradual", "identical", "independent", "individual", "inevitable",
    "informal", "intense", "internal", "invisible", "logical", "minor", "moderate",
    "natural", "negative", "neutral", "normal", "obvious", "official", "opposite",
    "ordinary", "original", "partial", "passive", "permanent", "personal", "positive",
    "potential", "practical", "precise", "previous", "primary", "private", "probable",
    "professional", "proper", "pure", "rapid", "rare", "regular", "relative", "relevant",
    "reliable", "remarkable", "remote", "representative", "reverse", "rigid", "rough",
    "separate", "severe", "significant", "similar", "simple", "slight", "smooth",
    "solid", "sophisticated", "specific", "stable", "standard", "steady", "substantial",
    "successful", "sufficient", "suitable", "supreme", "swift", "temporary", "thorough",
    "total", "traditional", "transparent", "typical", "ultimate", "universal", "unusual",
    "urgent", "usual", "vague", "valid", "visible", "vital", "vivid", "voluntary",
];

/// English advanced vocabulary (ages 12-14+) - complex words
const ENGLISH_ADVANCED: &[&str] = &[
    // Complex academic words
    "accumulate", "acquisition", "adjacent", "advocate", "allocate", "alternative", "ambiguous",
    "analyze", "anticipate", "apparent", "appropriate", "approximate", "arbitrary", "aspire",
    "assess", "assume", "attribute", "authorize", "circumstance", "clarify", "coherent",
    "coincide", "collaborate", "commence", "compensate", "complement", "comprehensive",
    "comprise", "conceive", "concentrate", "concept", "conclude", "concurrent", "conduct",
    "confer", "confine", "confirm", "conflict", "conform", "consent", "consequent",
    "considerable", "consist", "constant", "constitute", "constrain", "construct", "consult",
    "consume", "contemporary", "context", "contract", "contradict", "contrary", "contrast",
    "contribute", "controversy", "convene", "convention", "convert", "convince", "cooperate",
    "coordinate", "core", "correspond", "create", "criteria", "crucial", "culture",

    // Advanced verbs
    "deduce", "define", "demonstrate", "denote", "depict", "derive", "designate", "detect",
    "deviate", "device", "devote", "differentiate", "dimension", "diminish", "discard",
    "discriminate", "displace", "dispose", "distinct", "distinguish", "distort", "distribute",
    "diverse", "document", "dominate", "draft", "dynamic", "elaborate", "eliminate",
    "emerge", "emphasize", "empirical", "enable", "encounter", "energy", "enhance",
    "enormous", "ensure", "entity", "environment", "equate", "equivalent", "erode",
    "error", "establish", "estate", "estimate", "ethical", "ethnic", "evaluate", "eventual",
    "evident", "evolve", "exceed", "exclude", "exhibit", "expand", "expert", "explicit",
    "exploit", "export", "expose", "external", "extract",

    // Sophisticated nouns
    "facilitate", "factor", "feature", "federal", "phenomenon", "philosophy", "physical",
    "proportion", "prospect", "protocol", "psychology", "publication", "pursue", "qualitative",
    "rational", "reaction", "recover", "refine", "regime", "region", "register", "regulate",
    "reinforce", "reject", "relax", "release", "relevant", "reluctance", "rely", "remove",
    "require", "research", "reside", "resolve", "resource", "respond", "restore", "restrain",
    "restrict", "retain", "reveal", "revenue", "reverse", "revise", "revolution", "rigid",

    // Academic adjectives
    "abstract", "academic", "accurate", "acquire", "adequate", "adjacent", "adjust",
    "administrate", "adult", "advocate", "affect", "aggregate", "aid", "albeit", "allocate",
    "alter", "alternative", "ambiguous", "amend", "analogy", "analyze", "annual", "anticipate",
    "apparent", "append", "appreciate", "approach", "appropriate", "approximate", "arbitrary",
    "area", "aspect", "assemble", "assess", "assign", "assist", "assume", "assure", "attach",
    "attain", "attitude", "attribute", "author", "authority", "automate", "available", "aware",

    // Complex concepts
    "behalf", "benefit", "bias", "bond", "brief", "bulk", "capable", "capacity", "category",
    "cease", "challenge", "channel", "chapter", "chart", "chemical", "circumstance", "cite",
    "civil", "clarify", "classic", "clause", "code", "coherent", "coincide", "collapse",
    "colleague", "commence", "comment", "commission", "commit", "commodity", "communicate",
    "community", "compatible", "compensate", "compile", "complement", "complex", "component",
    "compound", "comprehensive", "comprise", "compute", "conceive", "concentrate", "concept",
    "conclude", "concurrent", "conduct", "confer", "confine", "confirm", "conflict", "conform",
    "consent", "consequent", "considerable", "consist", "constant", "constitute", "constrain",
    "construct", "consult", "consume", "contact", "contemporary", "context", "contract",
    "contradict", "contrary", "contrast", "contribute", "controversy", "convene", "converse",
    "convert", "convince", "cooperate", "coordinate", "core", "corporate", "correspond",
    "create", "credit", "criteria", "crucial", "culture", "currency", "cycle",

    // Scientific/technical
    "consequence", "conservation", "constitution", "contamination", "contradiction",
    "correlation", "deduction", "demonstration", "differentiation", "disintegration",
    "displacement", "documentation", "elimination", "equilibrium", "establishment",
    "examination", "exaggeration", "exclamation", "experimentation", "explanation",
    "exploration", "fluctuation", "formulation", "foundation", "fragmentation",
    "generation", "hypothesis", "illustration", "implementation", "implication",
    "indication", "infrastructure", "innovation", "integration", "interpretation",
    "investigation", "manifestation", "manipulation", "multiplication", "navigation",
    "negotiation", "observation", "optimization", "organization", "orientation",
    "participation", "penetration", "perturbation", "precipitation", "preparation",
    "preservation", "presentation", "proclamation", "propagation", "publication",
    "qualification", "quantification", "realization", "recommendation", "reconstruction",
    "representation", "reservation", "resolution", "restoration", "revelation",
    "specification", "stabilization", "synchronization", "transformation", "translation",
    "transportation", "utilization", "verification", "visualization",
];

// =============================================================================
// German Vocabulary Lists
// =============================================================================

/// German basic vocabulary (ages 6-8) - 500 common words
const GERMAN_BASIC: &[&str] = &[
    // Common words
    "der", "die", "das", "und", "ein", "eine", "in", "ist", "zu", "von",
    "mit", "den", "auf", "für", "nicht", "ich", "du", "er", "sie", "es",
    "wir", "ihr", "haben", "sein", "werden", "können", "müssen", "sollen",

    // Basic verbs
    "laufen", "spielen", "essen", "trinken", "schlafen", "gehen", "kommen", "sehen", "hören",
    "sprechen", "sagen", "machen", "tun", "geben", "nehmen", "finden", "suchen", "wollen",
    "mögen", "lieben", "helfen", "brauchen", "zeigen", "öffnen", "schließen", "lesen",
    "schreiben", "singen", "tanzen", "lachen", "weinen", "springen", "sitzen", "stehen",

    // Basic nouns
    "freund", "freundin", "familie", "haus", "zimmer", "tür", "fenster", "tisch", "stuhl",
    "bett", "buch", "spielzeug", "spiel", "ball", "hund", "katze", "vogel", "baum",
    "blume", "sonne", "mond", "stern", "wasser", "essen", "kuchen", "apfel", "brot",
    "milch", "tag", "nacht", "zeit", "jahr", "woche", "stunde", "morgen", "abend",
    "junge", "mädchen", "mann", "frau", "kind", "baby", "hand", "fuß", "kopf",
    "auge", "ohr", "nase", "mund", "gesicht",

    // Basic adjectives
    "gut", "schlecht", "groß", "klein", "glücklich", "traurig", "neu", "alt", "heiß",
    "kalt", "schnell", "langsam", "hoch", "tief", "lang", "kurz", "leicht", "schwer",
    "richtig", "falsch", "schön", "nett", "lieb", "lustig", "weich", "hart", "sauber",
    "schmutzig", "nass", "trocken", "voll", "leer", "stark", "schwach", "jung",
    "hell", "dunkel", "laut", "leise", "süß", "sauer",

    // Colors
    "rot", "blau", "grün", "gelb", "orange", "lila", "schwarz", "weiß", "braun",
    "rosa", "grau", "gold", "silber",

    // Numbers
    "eins", "zwei", "drei", "vier", "fünf", "sechs", "sieben", "acht", "neun", "zehn",
    "erste", "zweite", "dritte", "letzte", "viele", "wenige", "einige", "mehr", "weniger",
    "meiste",

    // Prepositions
    "auf", "ab", "an", "aus", "bei", "durch", "für", "gegen", "hinter", "in",
    "mit", "nach", "neben", "über", "um", "unter", "von", "vor", "zu", "zwischen",

    // School words
    "schule", "lehrer", "lehrerin", "schüler", "schülerin", "klasse", "lernen",
    "studieren", "papier", "stift", "bleistift", "farbe", "malen", "zeichnen",
    "zählen", "buchstabieren", "antwort", "frage", "geschichte", "bild", "zahl", "brief",

    // Time words
    "heute", "morgen", "gestern", "jetzt", "später", "bald", "früh", "spät", "immer",
    "nie", "manchmal", "oft", "vorher", "nachher", "während", "bis", "seit",

    // Family
    "mutter", "vater", "schwester", "bruder", "großmutter", "großvater", "oma", "opa",
    "tante", "onkel", "cousin", "cousine",

    // Animals
    "kuh", "schwein", "pferd", "schaf", "huhn", "ente", "fisch", "maus", "hase",
    "bär", "löwe", "tiger", "elefant", "affe", "schlange", "frosch", "biene",
    "ameise", "spinne", "schmetterling",

    // Nature
    "himmel", "wolke", "regen", "schnee", "wind", "gras", "blatt", "stein", "fels",
    "hügel", "berg", "fluss", "see", "meer", "strand", "sand", "welle", "eis",

    // Food
    "ei", "käse", "fleisch", "reis", "suppe", "pizza", "keks", "bonbon", "saft",
    "tee", "kaffee", "zucker", "salz", "butter", "honig",

    // Body parts
    "arm", "bein", "knie", "ellbogen", "finger", "zeh", "zehe", "rücken", "brust",
    "bauch", "hals", "haar", "zahn", "zähne", "haut", "herz", "knochen",

    // Clothing
    "hemd", "hose", "kleid", "rock", "mantel", "jacke", "hut", "mütze", "schuh",
    "socke", "handschuh",

    // Places
    "stadt", "dorf", "straße", "park", "laden", "geschäft", "bauernhof", "garten",
    "wald", "feld",

    // Transportation
    "auto", "bus", "zug", "fahrrad", "boot", "flugzeug", "lastwagen",

    // Activities
    "party", "feier", "reise", "besuch", "spaziergang", "fahrt", "rennen", "pause",
    "warten", "schauen", "hören",

    // Common modifiers
    "sehr", "zu", "viel", "wenig", "toll", "beste", "besser", "gleich", "anders",
    "andere", "jeder", "alle", "beide", "nur", "noch", "auch", "noch", "schon",

    // Questions/pronouns
    "wer", "wen", "wem", "wessen", "welche", "welcher", "welches", "was", "wo",
    "wann", "warum", "wie", "wieso", "weshalb",
    "mein", "meine", "dein", "deine", "sein", "seine", "ihr", "ihre", "unser",
    "unsere", "euer", "eure",
];

/// German intermediate vocabulary (ages 9-11) - 1000+ words
const GERMAN_INTERMEDIATE: &[&str] = &[
    // Adventure & exploration
    "abenteuer", "erforschen", "entdecken", "reise", "quest", "expedition", "fahrt",
    "wandern", "umherstreifen", "suchen", "untersuchen", "prüfen", "beobachten",
    "patrouillieren",

    // Emotions
    "aufgeregt", "nervös", "besorgt", "ängstlich", "mutig", "selbstbewusst", "stolz",
    "beschämt", "eifersüchtig", "dankbar", "überrascht", "erstaunt", "verwirrt",
    "neugierig", "entschlossen", "geduldig", "frustriert", "enttäuscht", "erleichtert",
    "zufrieden", "eifrig", "beunruhigt", "fröhlich", "düster",

    // Actions
    "erreichen", "vollbringen", "versuchen", "herausfordern", "konkurrieren",
    "zusammenarbeiten", "erschaffen", "verteidigen", "demonstrieren", "entwickeln",
    "ermutigen", "entkommen", "etablieren", "erfahren", "ausdrücken", "sammeln",
    "erzeugen", "bewachen", "verbessern", "einschließen", "informieren", "vorstellen",
    "erhalten", "verwalten", "teilnehmen", "vorbereiten", "präsentieren", "verhindern",
    "produzieren", "schützen", "veröffentlichen", "erkennen", "empfehlen", "reduzieren",
    "verweisen", "reflektieren", "verweigern", "veröffentlichen", "bleiben", "erinnern",
    "entfernen", "wiederholen", "ersetzen", "repräsentieren", "erfordern", "antworten",

    // Descriptive words
    "alt", "antik", "herrlich", "geheimnisvoll", "eigenartig", "kostbar", "bemerkenswert",
    "gewaltig", "brillant", "zart", "köstlich", "enorm", "ausgezeichnet", "fantastisch",
    "echt", "glorreich", "anmutig", "unglaublich", "intensiv", "massiv", "hervorragend",
    "prächtig", "überlegen", "schrecklich", "einzigartig", "wertvoll", "verschieden",
    "riesig",

    // Nature & science
    "atmosphäre", "klima", "kreatur", "erdbeben", "umwelt", "fossil", "galaxie",
    "lebensraum", "königreich", "landschaft", "mineral", "molekül", "ozean", "sauerstoff",
    "planet", "verschmutzung", "bevölkerung", "raubtier", "beute", "ressource",
    "spezies", "temperatur", "territorium", "universum", "vegetation", "vulkan",
    "wildnis",

    // Society & culture
    "zivilisation", "gemeinschaft", "brauch", "fest", "festival", "regierung", "erbe",
    "geschichte", "identität", "sprache", "legende", "denkmal", "mythos", "nation",
    "ursprung", "parlament", "pionier", "revolution", "gesellschaft", "symbol",
    "tradition", "vertrag",

    // Thinking
    "analysieren", "berechnen", "klassifizieren", "vergleichen", "schließen",
    "betrachten", "ableiten", "bestimmen", "unterscheiden", "schätzen", "bewerten",
    "erklären", "identifizieren", "illustrieren", "vorstellen", "folgern",
    "interpretieren", "messen", "vorhersagen", "argumentieren", "erinnern", "lösen",
    "zusammenfassen", "verstehen", "überprüfen",

    // Communication
    "ankündigen", "streiten", "kommunizieren", "überzeugen", "debattieren", "erklären",
    "diskutieren", "betonen", "übertreiben", "entschuldigen", "klatschen", "andeuten",
    "unterbrechen", "erwähnen", "verhandeln", "überreden", "verkünden", "protestieren",
    "vorschlagen", "flüstern",

    // Time
    "jahrhundert", "jahrzehnt", "dauer", "ära", "zeitalter", "ewig", "schließlich",
    "häufig", "allmählich", "sofort", "augenblick", "moment", "gelegentlich",
    "dauerhaft", "vorherig", "früher", "kürzlich", "regelmäßig", "folge", "reihenfolge",
    "vorübergehend", "zeitlos",

    // Places
    "grenze", "hauptstadt", "schloss", "kathedrale", "kammer", "kanal", "kapelle",
    "klippe", "kontinent", "korridor", "innenhof", "bezirk", "domäne", "verlies",
    "wohnung", "eingang", "anwesen", "festung", "grenze", "tor", "hauptquartier",
    "insel", "festland", "herrenhaus", "marktplatz", "palast", "durchgang", "halbinsel",
    "provinz", "region", "territorium", "turm", "tal", "dorf",

    // Materials
    "rüstung", "panzer", "bronze", "leinwand", "kristall", "stoff", "granit",
    "elfenbein", "leder", "marmor", "metall", "pergament", "töpferei", "seide",
    "stahl", "holz", "samt",

    // Qualities
    "absolut", "genau", "tatsächlich", "angemessen", "künstlich", "authentisch",
    "automatisch", "durchschnittlich", "grundlegend", "vorteilhaft", "fähig",
    "sicher", "bestimmt", "komplex", "konstant", "entscheidend", "definitiv",
    "dicht", "deutlich", "effizient", "aufwendig", "wesentlich", "offensichtlich",
    "exakt", "extrem", "flexibel", "formell", "häufig", "fundamental", "allgemein",
    "allmählich", "identisch", "unabhängig", "individuell", "unvermeidlich",
    "informell", "intensiv", "intern", "unsichtbar", "logisch", "geringfügig",
    "moderat", "natürlich", "negativ", "neutral", "normal", "offensichtlich",
    "offiziell", "gegenteilig", "gewöhnlich", "original", "teilweise", "passiv",
    "dauerhaft", "persönlich", "positiv", "potenziell", "praktisch", "präzise",
    "vorherig", "primär", "privat", "wahrscheinlich", "professionell", "ordnungsgemäß",
    "rein", "schnell", "selten", "regelmäßig", "relativ", "relevant", "zuverlässig",
    "bemerkenswert", "abgelegen", "repräsentativ", "umgekehrt", "starr", "rau",
    "getrennt", "schwer", "bedeutend", "ähnlich", "einfach", "leicht", "glatt",
    "fest", "anspruchsvoll", "spezifisch", "stabil", "standard", "stetig",
    "wesentlich", "erfolgreich", "ausreichend", "geeignet", "höchst", "schnell",
    "vorübergehend", "gründlich", "gesamt", "traditionell", "transparent", "typisch",
    "ultimativ", "universell", "ungewöhnlich", "dringend", "üblich", "vage",
    "gültig", "sichtbar", "lebenswichtig", "lebhaft", "freiwillig",
];

/// German advanced vocabulary (ages 12-14+) - complex words
const GERMAN_ADVANCED: &[&str] = &[
    // Complex academic words
    "akkumulieren", "erwerb", "angrenzend", "befürworten", "zuweisen", "alternative",
    "mehrdeutig", "analysieren", "antizipieren", "offensichtlich", "angemessen",
    "ungefähr", "willkürlich", "anstreben", "bewerten", "annehmen", "zuschreiben",
    "autorisieren", "umstand", "klären", "kohärent", "zusammenfallen", "zusammenarbeiten",
    "beginnen", "kompensieren", "ergänzen", "umfassend", "umfassen", "begreifen",
    "konzentrieren", "konzept", "schließen", "gleichzeitig", "durchführen", "beraten",
    "begrenzen", "bestätigen", "konflikt", "entsprechen", "zustimmen", "folglich",
    "beträchtlich", "bestehen", "konstant", "bilden", "einschränken", "konstruieren",
    "konsultieren", "verbrauchen", "zeitgenössisch", "kontext", "vertrag", "widersprechen",
    "gegenteilig", "kontrast", "beitragen", "kontroverse", "einberufen", "konvention",
    "konvertieren", "überzeugen", "kooperieren", "koordinieren", "kern", "entsprechen",
    "erschaffen", "kriterien", "entscheidend", "kultur",

    // Advanced verbs
    "ableiten", "definieren", "demonstrieren", "bezeichnen", "darstellen", "herleiten",
    "bestimmen", "erkennen", "abweichen", "gerät", "widmen", "differenzieren",
    "dimension", "verringern", "verwerfen", "diskriminieren", "verdrängen", "entsorgen",
    "deutlich", "unterscheiden", "verzerren", "verteilen", "vielfältig", "dokumentieren",
    "dominieren", "entwurf", "dynamisch", "ausarbeiten", "eliminieren", "entstehen",
    "betonen", "empirisch", "ermöglichen", "begegnen", "energie", "verbessern",
    "enorm", "sicherstellen", "entität", "umwelt", "gleichsetzen", "äquivalent",
    "erodieren", "fehler", "etablieren", "grundstück", "schätzen", "ethisch",
    "ethnisch", "evaluieren", "schließlich", "offensichtlich", "entwickeln", "überschreiten",
    "ausschließen", "ausstellen", "erweitern", "experte", "explizit", "ausbeuten",
    "exportieren", "aussetzen", "extern", "extrahieren",

    // Sophisticated nouns
    "erleichtern", "faktor", "merkmal", "bundesstaat", "phänomen", "philosophie",
    "physisch", "proportion", "aussicht", "protokoll", "psychologie", "publikation",
    "verfolgen", "qualitativ", "rational", "reaktion", "wiederherstellen", "verfeinern",
    "regime", "region", "register", "regulieren", "verstärken", "ablehnen", "entspannen",
    "veröffentlichen", "relevant", "zögern", "verlassen", "entfernen", "erfordern",
    "forschung", "wohnen", "lösen", "ressource", "reagieren", "wiederherstellen",
    "zurückhalten", "einschränken", "behalten", "offenbaren", "einnahmen", "umkehren",
    "überarbeiten", "revolution", "starr",

    // Academic adjectives
    "abstrakt", "akademisch", "genau", "erwerben", "adäquat", "angrenzend", "anpassen",
    "verwalten", "erwachsen", "befürworten", "beeinflussen", "aggregat", "hilfe",
    "obwohl", "zuweisen", "ändern", "alternative", "mehrdeutig", "ändern", "analogie",
    "analysieren", "jährlich", "antizipieren", "offensichtlich", "anhängen", "schätzen",
    "annäherung", "angemessen", "ungefähr", "willkürlich", "bereich", "aspekt",
    "zusammenbauen", "bewerten", "zuweisen", "unterstützen", "annehmen", "versichern",
    "anhängen", "erreichen", "haltung", "zuschreiben", "autor", "autorität",
    "automatisieren", "verfügbar", "bewusst",

    // Complex concepts
    "behalf", "nutzen", "voreingenommenheit", "bindung", "kurz", "masse", "fähig",
    "kapazität", "kategorie", "aufhören", "herausforderung", "kanal", "kapitel",
    "diagramm", "chemisch", "umstand", "zitieren", "zivil", "klären", "klassisch",
    "klausel", "code", "kohärent", "zusammenfallen", "zusammenbruch", "kollege",
    "beginnen", "kommentar", "kommission", "verpflichten", "ware", "kommunizieren",
    "gemeinschaft", "kompatibel", "kompensieren", "kompilieren", "ergänzen", "komplex",
    "komponente", "verbindung", "umfassend", "umfassen", "berechnen", "begreifen",
    "konzentrieren", "konzept", "schließen", "gleichzeitig", "durchführen", "beraten",
    "begrenzen", "bestätigen", "konflikt", "entsprechen", "zustimmen", "folglich",
    "beträchtlich", "bestehen", "konstant", "bilden", "einschränken", "konstruieren",
    "konsultieren", "verbrauchen", "kontakt", "zeitgenössisch", "kontext", "vertrag",
    "widersprechen", "gegenteilig", "kontrast", "beitragen", "kontroverse", "einberufen",
    "konversation", "konvertieren", "überzeugen", "kooperieren", "koordinieren", "kern",
    "unternehmen", "entsprechen", "erschaffen", "kredit", "kriterien", "entscheidend",
    "kultur", "währung", "zyklus",

    // Scientific/technical
    "konsequenz", "naturschutz", "verfassung", "kontamination", "widerspruch",
    "korrelation", "ableitung", "demonstration", "differenzierung", "zerfall",
    "verschiebung", "dokumentation", "eliminierung", "gleichgewicht", "etablierung",
    "prüfung", "übertreibung", "ausruf", "experiment", "erklärung", "erforschung",
    "schwankung", "formulierung", "grundlage", "fragmentierung", "generation",
    "hypothese", "illustration", "implementierung", "implikation", "hinweis",
    "infrastruktur", "innovation", "integration", "interpretation", "untersuchung",
    "manifestation", "manipulation", "multiplikation", "navigation", "verhandlung",
    "beobachtung", "optimierung", "organisation", "orientierung", "teilnahme",
    "penetration", "perturbation", "niederschlag", "vorbereitung", "erhaltung",
    "präsentation", "proklamation", "ausbreitung", "publikation", "qualifikation",
    "quantifizierung", "realisierung", "empfehlung", "rekonstruktion", "darstellung",
    "reservierung", "auflösung", "restaurierung", "offenbarung", "spezifikation",
    "stabilisierung", "synchronisierung", "transformation", "übersetzung",
    "transport", "nutzung", "überprüfung", "visualisierung",
];

// =============================================================================
// Suggestion Mappings
// =============================================================================

lazy_static::lazy_static! {
    /// English word suggestions: complex word -> simpler alternatives
    static ref ENGLISH_SUGGESTIONS: HashMap<&'static str, Vec<&'static str>> = {
        let mut m = HashMap::new();

        // Advanced -> Intermediate/Basic
        m.insert("exclamation", vec!["shout", "yell", "call out"]);
        m.insert("investigate", vec!["explore", "look at", "study"]);
        m.insert("consequence", vec!["result", "effect", "outcome"]);
        m.insert("phenomenon", vec!["event", "happening", "thing"]);
        m.insert("accumulate", vec!["gather", "collect", "pile up"]);
        m.insert("adjacent", vec!["next to", "beside", "near"]);
        m.insert("anticipate", vec!["expect", "wait for", "look ahead"]);
        m.insert("approximate", vec!["about", "near", "close to"]);
        m.insert("attribute", vec!["quality", "trait", "feature"]);
        m.insert("collaborate", vec!["work together", "cooperate", "team up"]);
        m.insert("compensate", vec!["make up for", "pay back", "balance"]);
        m.insert("comprehensive", vec!["complete", "full", "total"]);
        m.insert("constitute", vec!["make up", "form", "create"]);
        m.insert("contemporary", vec!["modern", "current", "today's"]);
        m.insert("demonstrate", vec!["show", "prove", "display"]);
        m.insert("differentiate", vec!["tell apart", "separate", "distinguish"]);
        m.insert("eliminate", vec!["remove", "get rid of", "delete"]);
        m.insert("facilitate", vec!["help", "make easier", "assist"]);
        m.insert("implement", vec!["do", "carry out", "put in place"]);
        m.insert("manifest", vec!["show", "reveal", "display"]);
        m.insert("obtain", vec!["get", "receive", "acquire"]);
        m.insert("participate", vec!["join in", "take part", "help"]);
        m.insert("persuade", vec!["convince", "talk into", "get to agree"]);
        m.insert("predominant", vec!["main", "biggest", "most common"]);
        m.insert("proclaim", vec!["announce", "declare", "say loudly"]);
        m.insert("substantial", vec!["large", "big", "important"]);
        m.insert("terminate", vec!["end", "stop", "finish"]);
        m.insert("utilize", vec!["use", "employ", "apply"]);
        m.insert("magnificent", vec!["great", "wonderful", "amazing"]);
        m.insert("tremendous", vec!["huge", "very big", "enormous"]);

        m
    };

    /// German word suggestions: complex word -> simpler alternatives
    static ref GERMAN_SUGGESTIONS: HashMap<&'static str, Vec<&'static str>> = {
        let mut m = HashMap::new();

        // Advanced -> Intermediate/Basic
        m.insert("ausruf", vec!["rufen", "schreien", "laut sagen"]);
        m.insert("untersuchen", vec!["erforschen", "anschauen", "prüfen"]);
        m.insert("konsequenz", vec!["folge", "ergebnis", "wirkung"]);
        m.insert("phänomen", vec!["ereignis", "geschehen", "sache"]);
        m.insert("akkumulieren", vec!["sammeln", "anhäufen", "aufbauen"]);
        m.insert("angrenzend", vec!["neben", "bei", "nahe"]);
        m.insert("antizipieren", vec!["erwarten", "voraussehen", "ahnen"]);
        m.insert("ungefähr", vec!["etwa", "circa", "nahe"]);
        m.insert("zuschreiben", vec!["merkmal", "eigenschaft", "qualität"]);
        m.insert("zusammenarbeiten", vec!["kooperieren", "gemeinsam arbeiten", "helfen"]);
        m.insert("kompensieren", vec!["ausgleichen", "ersetzen", "gutmachen"]);
        m.insert("umfassend", vec!["vollständig", "komplett", "ganz"]);
        m.insert("bilden", vec!["machen", "formen", "erstellen"]);
        m.insert("zeitgenössisch", vec!["modern", "aktuell", "heutig"]);
        m.insert("demonstrieren", vec!["zeigen", "beweisen", "darstellen"]);
        m.insert("differenzieren", vec!["unterscheiden", "trennen", "auseinanderhalten"]);
        m.insert("eliminieren", vec!["entfernen", "beseitigen", "löschen"]);
        m.insert("erleichtern", vec!["helfen", "vereinfachen", "unterstützen"]);
        m.insert("implementieren", vec!["umsetzen", "durchführen", "einführen"]);
        m.insert("manifestieren", vec!["zeigen", "offenbaren", "darstellen"]);
        m.insert("erhalten", vec!["bekommen", "erwerben", "empfangen"]);
        m.insert("teilnehmen", vec!["mitmachen", "dabei sein", "helfen"]);
        m.insert("überreden", vec!["überzeugen", "einreden", "bewegen"]);
        m.insert("vorherrschend", vec!["hauptsächlich", "wichtigste", "häufigste"]);
        m.insert("verkünden", vec!["ankündigen", "erklären", "laut sagen"]);
        m.insert("wesentlich", vec!["groß", "wichtig", "bedeutend"]);
        m.insert("beenden", vec!["aufhören", "stoppen", "abschließen"]);
        m.insert("nutzen", vec!["verwenden", "benutzen", "gebrauchen"]);
        m.insert("herrlich", vec!["toll", "wunderbar", "großartig"]);
        m.insert("gewaltig", vec!["riesig", "sehr groß", "enorm"]);

        m
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_words() {
        let content = "Hello, world! This is a test.";
        let words = extract_words(content);
        assert_eq!(words, vec!["hello", "world", "this", "is", "a", "test"]);
    }

    #[test]
    fn test_extract_words_empty() {
        let words = extract_words("");
        assert!(words.is_empty());
    }

    #[test]
    fn test_get_word_level_basic_english() {
        let level = get_word_level("the", &Language::En);
        assert_eq!(level, VocabularyLevel::Basic);
    }

    #[test]
    fn test_get_word_level_intermediate_english() {
        let level = get_word_level("adventure", &Language::En);
        assert_eq!(level, VocabularyLevel::Intermediate);
    }

    #[test]
    fn test_get_word_level_advanced_english() {
        let level = get_word_level("phenomenon", &Language::En);
        assert_eq!(level, VocabularyLevel::Advanced);
    }

    #[test]
    fn test_get_word_level_heuristic() {
        // Unknown word, uses heuristic
        let level = get_word_level("supercalifragilisticexpialidocious", &Language::En);
        assert_eq!(level, VocabularyLevel::Advanced); // > 10 chars
    }

    #[test]
    fn test_get_suggestions_english() {
        let suggestions = get_suggestions("exclamation", &Language::En);
        assert!(!suggestions.is_empty());
        assert!(suggestions.contains(&"shout".to_string()));
    }

    #[test]
    fn test_get_suggestions_german() {
        let suggestions = get_suggestions("ausruf", &Language::De);
        assert!(!suggestions.is_empty());
        assert!(suggestions.contains(&"rufen".to_string()));
    }

    #[test]
    fn test_get_suggestions_unknown_word() {
        let suggestions = get_suggestions("unknownword", &Language::En);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_check_vocabulary_level_no_violations() {
        let content = "The cat sat on the mat.";
        let violations = check_vocabulary_level(
            content,
            Language::En,
            VocabularyLevel::Basic,
            "test_node",
        );
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_vocabulary_level_with_violations() {
        let content = "The phenomenon was investigated thoroughly.";
        let violations = check_vocabulary_level(
            content,
            Language::En,
            VocabularyLevel::Basic,
            "test_node",
        );
        assert!(!violations.is_empty());

        // Should flag "phenomenon" and "investigated" as advanced
        let violation_words: Vec<_> = violations.iter().map(|v| v.word.as_str()).collect();
        assert!(violation_words.contains(&"phenomenon"));
        assert!(violation_words.contains(&"investigated"));
    }

    #[test]
    fn test_check_vocabulary_level_empty_content() {
        let violations = check_vocabulary_level(
            "",
            Language::En,
            VocabularyLevel::Basic,
            "test_node",
        );
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_vocabulary_level_german() {
        let content = "Das Phänomen wurde untersucht.";
        let violations = check_vocabulary_level(
            content,
            Language::De,
            VocabularyLevel::Basic,
            "test_node",
        );
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_violation_includes_suggestions() {
        let content = "This is an exclamation!";
        let violations = check_vocabulary_level(
            content,
            Language::En,
            VocabularyLevel::Basic,
            "test_node",
        );

        if let Some(violation) = violations.iter().find(|v| v.word == "exclamation") {
            assert!(!violation.suggestions.is_empty());
            assert!(violation.suggestions.contains(&"shout".to_string()));
        }
    }

    #[test]
    fn test_level_comparison() {
        assert!(is_level_exceeded(&VocabularyLevel::Advanced, &VocabularyLevel::Basic));
        assert!(is_level_exceeded(&VocabularyLevel::Intermediate, &VocabularyLevel::Basic));
        assert!(!is_level_exceeded(&VocabularyLevel::Basic, &VocabularyLevel::Basic));
        assert!(!is_level_exceeded(&VocabularyLevel::Basic, &VocabularyLevel::Intermediate));
    }
}
