//! Language-specific prompt templates for story generation
//!
//! This module provides fallback prompt templates for different languages and age groups.
//! These templates are used when dynamic prompt generation via prompt-helper fails or
//! when operating in offline/fallback mode.
//!
//! # Template Structure
//!
//! Each template includes:
//! - **System Prompt**: Instructions for the LLM on how to generate content
//! - **User Prompt**: Template with placeholders for story context
//!
//! # Template Variables
//!
//! Templates support the following placeholders:
//! - `{theme}`: Story theme (e.g., "Space Adventure", "Mystery Island")
//! - `{age_group}`: Target age group (e.g., "6-8", "9-11")
//! - `{educational_goals}`: Optional learning objectives
//! - `{previous_content}`: Story so far (for continuity)
//! - `{choices_made}`: Path taken through the story
//! - `{node_position}`: Current position in story (e.g., "3/16")
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use story_generator::prompts::PromptTemplates;
//! use shared_types::{AgeGroup, Language};
//!
//! let templates = PromptTemplates::new();
//! let (system, user) = templates.get_template(Language::En, AgeGroup::_9To11);
//!
//! let filled = user
//!     .replace("{theme}", "Space Exploration")
//!     .replace("{age_group}", "9-11")
//!     .replace("{node_position}", "5/16");
//! ```

use shared_types::{AgeGroup, Language};
use std::collections::HashMap;

/// Container for language and age-specific prompt templates
#[allow(dead_code)]
pub struct PromptTemplates {
    /// English templates by age group
    pub english: HashMap<AgeGroup, (String, String)>,
    /// German templates by age group
    pub german: HashMap<AgeGroup, (String, String)>,
}

impl PromptTemplates {
    /// Create new template collection with all language variants
    #[allow(dead_code)]
    pub fn new() -> Self {
        let mut templates = Self {
            english: HashMap::new(),
            german: HashMap::new(),
        };

        templates.initialize_english_templates();
        templates.initialize_german_templates();

        templates
    }

    /// Get template for specified language and age group
    ///
    /// Returns tuple of (system_prompt, user_prompt)
    ///
    /// Falls back to English 9-11 if specific template not found
    #[allow(dead_code)]
    pub fn get_template(&self, language: Language, age_group: AgeGroup) -> (String, String) {
        let templates = match language {
            Language::En => &self.english,
            Language::De => &self.german,
        };

        templates
            .get(&age_group)
            .cloned()
            .unwrap_or_else(|| self.english.get(&AgeGroup::_9To11).unwrap().clone())
    }

    /// Initialize English templates for all age groups
    fn initialize_english_templates(&mut self) {
        // 6-8 years: Simple language, short sentences, clear choices
        self.english.insert(
            AgeGroup::_6To8,
            (
                self.system_prompt_english_6_8(),
                self.user_prompt_english_6_8(),
            ),
        );

        // 9-11 years: Moderate complexity, adventure focus
        self.english.insert(
            AgeGroup::_9To11,
            (
                self.system_prompt_english_9_11(),
                self.user_prompt_english_9_11(),
            ),
        );

        // 12-14 years: More complex themes, moral choices
        self.english.insert(
            AgeGroup::_12To14,
            (
                self.system_prompt_english_12_14(),
                self.user_prompt_english_12_14(),
            ),
        );

        // 15-17 years: Advanced themes, nuanced choices
        self.english.insert(
            AgeGroup::_15To17,
            (
                self.system_prompt_english_15_17(),
                self.user_prompt_english_15_17(),
            ),
        );

        // 18+ years: Sophisticated narratives, complex decisions
        self.english.insert(
            AgeGroup::Plus18,
            (
                self.system_prompt_english_18_plus(),
                self.user_prompt_english_18_plus(),
            ),
        );
    }

    /// Initialize German templates for all age groups
    fn initialize_german_templates(&mut self) {
        // 6-8 Jahre: Einfache Sprache, kurze Sätze
        self.german.insert(
            AgeGroup::_6To8,
            (
                self.system_prompt_german_6_8(),
                self.user_prompt_german_6_8(),
            ),
        );

        // 9-11 Jahre: Moderate Komplexität
        self.german.insert(
            AgeGroup::_9To11,
            (
                self.system_prompt_german_9_11(),
                self.user_prompt_german_9_11(),
            ),
        );

        // 12-14 Jahre: Komplexere Themen
        self.german.insert(
            AgeGroup::_12To14,
            (
                self.system_prompt_german_12_14(),
                self.user_prompt_german_12_14(),
            ),
        );

        // 15-17 Jahre: Fortgeschrittene Themen
        self.german.insert(
            AgeGroup::_15To17,
            (
                self.system_prompt_german_15_17(),
                self.user_prompt_german_15_17(),
            ),
        );

        // 18+ Jahre: Anspruchsvolle Erzählungen
        self.german.insert(
            AgeGroup::Plus18,
            (
                self.system_prompt_german_18_plus(),
                self.user_prompt_german_18_plus(),
            ),
        );
    }

    // ========================================================================
    // English System Prompts
    // ========================================================================

    fn system_prompt_english_6_8(&self) -> String {
        r#"You are a creative storyteller writing interactive stories for children ages 6-8.

Guidelines:
- Use simple, clear language with short sentences
- Keep vocabulary appropriate for early readers
- Create fun, engaging narratives with positive themes
- Avoid scary or complex concepts
- Include educational elements when requested
- Generate exactly 3 simple choices at the end

Format your response as JSON:
{
  "narrative": "Story text here (~200 words)",
  "choices": ["Choice 1", "Choice 2", "Choice 3"],
  "educational_content": "Optional educational info"
}"#.to_string()
    }

    fn system_prompt_english_9_11(&self) -> String {
        r#"You are a creative storyteller writing interactive adventure stories for children ages 9-11.

Guidelines:
- Use age-appropriate language with moderate complexity
- Include adventure, mystery, or exploration themes
- Create engaging characters and situations
- Maintain continuity with previous story content
- Include educational elements when requested
- Generate exactly 3 meaningful choices that affect the story

Format your response as JSON:
{
  "narrative": "Story text here (~300-400 words)",
  "choices": ["Choice 1 (~20 words)", "Choice 2 (~20 words)", "Choice 3 (~20 words)"],
  "educational_content": "Optional educational info"
}"#.to_string()
    }

    fn system_prompt_english_12_14(&self) -> String {
        r#"You are a creative storyteller writing interactive stories for young teens ages 12-14.

Guidelines:
- Use sophisticated vocabulary appropriate for middle school
- Explore themes of friendship, identity, and moral choices
- Create complex characters with realistic motivations
- Build tension and maintain story continuity
- Include educational elements when requested
- Generate exactly 3 choices with meaningful consequences

Format your response as JSON:
{
  "narrative": "Story text here (~400-500 words)",
  "choices": ["Choice 1 with consequences", "Choice 2 with consequences", "Choice 3 with consequences"],
  "educational_content": "Optional educational info"
}"#.to_string()
    }

    fn system_prompt_english_15_17(&self) -> String {
        r#"You are a creative storyteller writing interactive stories for teenagers ages 15-17.

Guidelines:
- Use advanced vocabulary and complex sentence structures
- Explore sophisticated themes (ethics, philosophy, society)
- Create nuanced characters with conflicting motivations
- Build intricate plots with multiple layers
- Include educational elements when requested
- Generate exactly 3 choices with significant ethical dimensions

Format your response as JSON:
{
  "narrative": "Story text here (~500 words)",
  "choices": ["Complex choice 1", "Complex choice 2", "Complex choice 3"],
  "educational_content": "Optional educational info"
}"#.to_string()
    }

    fn system_prompt_english_18_plus(&self) -> String {
        r#"You are a creative storyteller writing sophisticated interactive narratives for adults.

Guidelines:
- Use literary-quality prose with rich vocabulary
- Explore complex themes and moral ambiguity
- Create deep, multi-dimensional characters
- Build intricate narratives with subtext and symbolism
- Include educational elements when requested
- Generate exactly 3 choices with far-reaching implications

Format your response as JSON:
{
  "narrative": "Story text here (~500-600 words)",
  "choices": ["Sophisticated choice 1", "Sophisticated choice 2", "Sophisticated choice 3"],
  "educational_content": "Optional educational info"
}"#.to_string()
    }

    // ========================================================================
    // English User Prompts
    // ========================================================================

    fn user_prompt_english_6_8(&self) -> String {
        r#"Theme: {theme}
Age Group: {age_group}
Node Position: {node_position}

Previous Story:
{previous_content}

Choices Made:
{choices_made}

Educational Goals:
{educational_goals}

Generate the next part of the story with 3 simple choices."#.to_string()
    }

    fn user_prompt_english_9_11(&self) -> String {
        r#"Theme: {theme}
Age Group: {age_group}
Node Position: {node_position}

Story So Far:
{previous_content}

Choices Made:
{choices_made}

Educational Goals:
{educational_goals}

Generate the next chapter of this adventure with 3 meaningful choices."#.to_string()
    }

    fn user_prompt_english_12_14(&self) -> String {
        r#"Theme: {theme}
Age Group: {age_group}
Node Position: {node_position}

Story Context:
{previous_content}

Path Taken:
{choices_made}

Educational Objectives:
{educational_goals}

Continue the story with 3 choices that have meaningful consequences."#.to_string()
    }

    fn user_prompt_english_15_17(&self) -> String {
        r#"Theme: {theme}
Age Group: {age_group}
Node Position: {node_position}

Narrative Context:
{previous_content}

Decisions Made:
{choices_made}

Learning Objectives:
{educational_goals}

Advance the narrative with 3 ethically complex choices."#.to_string()
    }

    fn user_prompt_english_18_plus(&self) -> String {
        r#"Theme: {theme}
Age Group: {age_group}
Node Position: {node_position}

Narrative Context:
{previous_content}

Narrative Path:
{choices_made}

Educational Context:
{educational_goals}

Continue the narrative with 3 sophisticated choices with significant implications."#.to_string()
    }

    // ========================================================================
    // German System Prompts
    // ========================================================================

    fn system_prompt_german_6_8(&self) -> String {
        r#"Du bist ein kreativer Geschichtenerzähler, der interaktive Geschichten für Kinder im Alter von 6-8 Jahren schreibt.

Richtlinien:
- Verwende einfache, klare Sprache mit kurzen Sätzen
- Halte das Vokabular für Erstleser angemessen
- Erstelle lustige, fesselnde Erzählungen mit positiven Themen
- Vermeide gruselige oder komplexe Konzepte
- Füge bei Bedarf Bildungselemente hinzu
- Generiere genau 3 einfache Wahlmöglichkeiten am Ende

Formatiere deine Antwort als JSON:
{
  "narrative": "Geschichtentext hier (~200 Wörter)",
  "choices": ["Wahl 1", "Wahl 2", "Wahl 3"],
  "educational_content": "Optionale Bildungsinformationen"
}"#.to_string()
    }

    fn system_prompt_german_9_11(&self) -> String {
        r#"Du bist ein kreativer Geschichtenerzähler, der interaktive Abenteuergeschichten für Kinder im Alter von 9-11 Jahren schreibt.

Richtlinien:
- Verwende altersgerechte Sprache mit mäßiger Komplexität
- Integriere Abenteuer-, Rätsel- oder Entdeckungsthemen
- Erstelle ansprechende Charaktere und Situationen
- Halte die Kontinuität mit vorherigen Geschichteninhalten
- Füge bei Bedarf Bildungselemente hinzu
- Generiere genau 3 bedeutungsvolle Wahlmöglichkeiten, die die Geschichte beeinflussen

Formatiere deine Antwort als JSON:
{
  "narrative": "Geschichtentext hier (~300-400 Wörter)",
  "choices": ["Wahl 1 (~20 Wörter)", "Wahl 2 (~20 Wörter)", "Wahl 3 (~20 Wörter)"],
  "educational_content": "Optionale Bildungsinformationen"
}"#.to_string()
    }

    fn system_prompt_german_12_14(&self) -> String {
        r#"Du bist ein kreativer Geschichtenerzähler, der interaktive Geschichten für Jugendliche im Alter von 12-14 Jahren schreibt.

Richtlinien:
- Verwende anspruchsvolles Vokabular für Mittelschüler
- Erkunde Themen wie Freundschaft, Identität und moralische Entscheidungen
- Erstelle komplexe Charaktere mit realistischen Motivationen
- Baue Spannung auf und halte die Geschichtenkontinuität
- Füge bei Bedarf Bildungselemente hinzu
- Generiere genau 3 Wahlmöglichkeiten mit bedeutungsvollen Konsequenzen

Formatiere deine Antwort als JSON:
{
  "narrative": "Geschichtentext hier (~400-500 Wörter)",
  "choices": ["Wahl 1 mit Konsequenzen", "Wahl 2 mit Konsequenzen", "Wahl 3 mit Konsequenzen"],
  "educational_content": "Optionale Bildungsinformationen"
}"#.to_string()
    }

    fn system_prompt_german_15_17(&self) -> String {
        r#"Du bist ein kreativer Geschichtenerzähler, der interaktive Geschichten für Jugendliche im Alter von 15-17 Jahren schreibt.

Richtlinien:
- Verwende fortgeschrittenes Vokabular und komplexe Satzstrukturen
- Erkunde anspruchsvolle Themen (Ethik, Philosophie, Gesellschaft)
- Erstelle nuancierte Charaktere mit widersprüchlichen Motivationen
- Baue komplizierte Handlungen mit mehreren Ebenen
- Füge bei Bedarf Bildungselemente hinzu
- Generiere genau 3 Wahlmöglichkeiten mit erheblichen ethischen Dimensionen

Formatiere deine Antwort als JSON:
{
  "narrative": "Geschichtentext hier (~500 Wörter)",
  "choices": ["Komplexe Wahl 1", "Komplexe Wahl 2", "Komplexe Wahl 3"],
  "educational_content": "Optionale Bildungsinformationen"
}"#.to_string()
    }

    fn system_prompt_german_18_plus(&self) -> String {
        r#"Du bist ein kreativer Geschichtenerzähler, der anspruchsvolle interaktive Erzählungen für Erwachsene schreibt.

Richtlinien:
- Verwende literarisch hochwertige Prosa mit reichem Vokabular
- Erkunde komplexe Themen und moralische Mehrdeutigkeit
- Erstelle tiefe, mehrdimensionale Charaktere
- Baue komplizierte Erzählungen mit Subtext und Symbolik
- Füge bei Bedarf Bildungselemente hinzu
- Generiere genau 3 Wahlmöglichkeiten mit weitreichenden Implikationen

Formatiere deine Antwort als JSON:
{
  "narrative": "Geschichtentext hier (~500-600 Wörter)",
  "choices": ["Anspruchsvolle Wahl 1", "Anspruchsvolle Wahl 2", "Anspruchsvolle Wahl 3"],
  "educational_content": "Optionale Bildungsinformationen"
}"#.to_string()
    }

    // ========================================================================
    // German User Prompts
    // ========================================================================

    fn user_prompt_german_6_8(&self) -> String {
        r#"Thema: {theme}
Altersgruppe: {age_group}
Knotenposition: {node_position}

Bisherige Geschichte:
{previous_content}

Getroffene Wahlen:
{choices_made}

Bildungsziele:
{educational_goals}

Generiere den nächsten Teil der Geschichte mit 3 einfachen Wahlmöglichkeiten."#.to_string()
    }

    fn user_prompt_german_9_11(&self) -> String {
        r#"Thema: {theme}
Altersgruppe: {age_group}
Knotenposition: {node_position}

Geschichte bisher:
{previous_content}

Getroffene Wahlen:
{choices_made}

Bildungsziele:
{educational_goals}

Generiere das nächste Kapitel dieses Abenteuers mit 3 bedeutungsvollen Wahlmöglichkeiten."#.to_string()
    }

    fn user_prompt_german_12_14(&self) -> String {
        r#"Thema: {theme}
Altersgruppe: {age_group}
Knotenposition: {node_position}

Geschichtenkontext:
{previous_content}

Gewählter Weg:
{choices_made}

Bildungsziele:
{educational_goals}

Setze die Geschichte mit 3 Wahlmöglichkeiten fort, die bedeutungsvolle Konsequenzen haben."#.to_string()
    }

    fn user_prompt_german_15_17(&self) -> String {
        r#"Thema: {theme}
Altersgruppe: {age_group}
Knotenposition: {node_position}

Erzählkontext:
{previous_content}

Getroffene Entscheidungen:
{choices_made}

Lernziele:
{educational_goals}

Führe die Erzählung mit 3 ethisch komplexen Wahlmöglichkeiten fort."#.to_string()
    }

    fn user_prompt_german_18_plus(&self) -> String {
        r#"Thema: {theme}
Altersgruppe: {age_group}
Knotenposition: {node_position}

Erzählkontext:
{previous_content}

Erzählweg:
{choices_made}

Bildungskontext:
{educational_goals}

Setze die Erzählung mit 3 anspruchsvollen Wahlmöglichkeiten mit erheblichen Implikationen fort."#.to_string()
    }
}

impl Default for PromptTemplates {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_initialization() {
        let templates = PromptTemplates::new();

        // Verify all age groups have English templates
        assert!(templates.english.contains_key(&AgeGroup::_6To8));
        assert!(templates.english.contains_key(&AgeGroup::_9To11));
        assert!(templates.english.contains_key(&AgeGroup::_12To14));
        assert!(templates.english.contains_key(&AgeGroup::_15To17));
        assert!(templates.english.contains_key(&AgeGroup::Plus18));

        // Verify all age groups have German templates
        assert!(templates.german.contains_key(&AgeGroup::_6To8));
        assert!(templates.german.contains_key(&AgeGroup::_9To11));
        assert!(templates.german.contains_key(&AgeGroup::_12To14));
        assert!(templates.german.contains_key(&AgeGroup::_15To17));
        assert!(templates.german.contains_key(&AgeGroup::Plus18));
    }

    #[test]
    fn test_get_template_english() {
        let templates = PromptTemplates::new();
        let (system, user) = templates.get_template(Language::En, AgeGroup::_9To11);

        assert!(!system.is_empty());
        assert!(!user.is_empty());
        assert!(system.contains("storyteller"));
        assert!(user.contains("{theme}"));
        assert!(user.contains("{age_group}"));
    }

    #[test]
    fn test_get_template_german() {
        let templates = PromptTemplates::new();
        let (system, user) = templates.get_template(Language::De, AgeGroup::_9To11);

        assert!(!system.is_empty());
        assert!(!user.is_empty());
        assert!(system.contains("Geschichtenerzähler"));
        assert!(user.contains("{theme}"));
    }

    #[test]
    fn test_template_placeholders() {
        let templates = PromptTemplates::new();
        let (_, user) = templates.get_template(Language::En, AgeGroup::_9To11);

        // Verify all expected placeholders are present
        assert!(user.contains("{theme}"));
        assert!(user.contains("{age_group}"));
        assert!(user.contains("{node_position}"));
        assert!(user.contains("{previous_content}"));
        assert!(user.contains("{choices_made}"));
        assert!(user.contains("{educational_goals}"));
    }

    #[test]
    fn test_template_replacement() {
        let templates = PromptTemplates::new();
        let (_, user) = templates.get_template(Language::En, AgeGroup::_9To11);

        let filled = user
            .replace("{theme}", "Space Adventure")
            .replace("{age_group}", "9-11")
            .replace("{node_position}", "5/16")
            .replace("{previous_content}", "Story beginning")
            .replace("{choices_made}", "Choice 1 → Choice 2")
            .replace("{educational_goals}", "Learn about planets");

        assert!(!filled.contains("{theme}"));
        assert!(!filled.contains("{age_group}"));
        assert!(filled.contains("Space Adventure"));
        assert!(filled.contains("9-11"));
    }
}
