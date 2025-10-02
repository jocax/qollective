// ABOUTME: TypeScript type definitions matching Rust shared-types business models
// ABOUTME: Provides type safety for React components interacting with holodeck data

export enum HolodeckStoryType {
  Adventure = 'Adventure',
  Mystery = 'Mystery',
  Drama = 'Drama',
  Comedy = 'Comedy',
  Historical = 'Historical',
  SciFi = 'SciFi',
  Fantasy = 'Fantasy',
  Educational = 'Educational',
}

export enum SafetyLevel {
  Training = 'Training',
  Standard = 'Standard',
  Reduced = 'Reduced',
  Disabled = 'Disabled',
}

export enum SessionStatus {
  Active = 'Active',
  Paused = 'Paused',
  Completed = 'Completed',
  Aborted = 'Aborted',
  SafetyHalt = 'SafetyHalt',
}

export interface Position3D {
  x: number;
  y: number;
  z: number;
}

export interface EnvironmentalControls {
  temperatureCelsius: number;
  humidityPercent: number;
  atmosphericPressure: number;
  oxygenLevel: number;
  windSimulation: boolean;
  weatherEffects: boolean;
}

export interface HolodeckConfig {
  safetyLevel: SafetyLevel | string;
  maxParticipants: number;
  durationMinutes?: number;
  autoSaveEnabled: boolean;
  voiceRecognition: boolean;
  hapticFeedback: boolean;
  replicatorAccess: boolean;
  transporterIntegration: boolean;
  environmentalControls: EnvironmentalControls;
}

export interface Scene {
  id: string;
  name: string;
  description: string;
  environmentId: string;
  charactersPresent: string[];
  props: SceneProp[];
  backgroundAudio?: string;
  lightingConfig: LightingConfig;
  physicsSettings: PhysicsSettings;
}

export interface SceneProp {
  id: string;
  name: string;
  description: string;
  position: Position3D;
  interactive: boolean;
  physicsEnabled: boolean;
}

export interface LightingConfig {
  ambientLight: number;
  directionalLights: DirectionalLight[];
  mood: LightingMood;
}

export interface DirectionalLight {
  direction: Position3D;
  intensity: number;
  color: Color;
}

export interface Color {
  r: number;
  g: number;
  b: number;
  a: number;
}

export enum LightingMood {
  Bright = 'Bright',
  Dim = 'Dim',
  Dramatic = 'Dramatic',
  Cozy = 'Cozy',
  Mysterious = 'Mysterious',
  Tense = 'Tense',
}

export interface PhysicsSettings {
  gravityEnabled: boolean;
  gravityStrength: number;
  collisionDetection: boolean;
  realTimePhysics: boolean;
}

export interface Holodeck {
  id: string;
  name: string;
  topic: string;
  storyType: HolodeckStoryType;
  participants: Character[];
  currentScene?: Scene;
  configuration: HolodeckConfig;
  createdAt: Date;
  updatedAt: Date;
}

export interface Character {
  id: string;
  name: string;
  characterType: CharacterType;
  personality: PersonalityTraits;
  voiceConfig: VoiceConfig;
  appearance: CharacterAppearance;
  knowledgeDomains: KnowledgeDomain[];
  relationships: CharacterRelationship[];
  currentMood: Mood;
  position?: Position3D;
  status: CharacterStatus;
}

export enum CharacterType {
  Captain = 'Captain',
  FirstOfficer = 'FirstOfficer',
  Android = 'Android',
  ChiefEngineer = 'ChiefEngineer',
  ChiefMedicalOfficer = 'ChiefMedicalOfficer',
  Counselor = 'Counselor',
  ChiefOfSecurity = 'ChiefOfSecurity',
  NavigationOfficer = 'NavigationOfficer',
  BarTender = 'BarTender',
  Civilian = 'Civilian',
  HistoricalFigure = 'HistoricalFigure',
  Custom = 'Custom',
}

export interface PersonalityTraits {
  logical: number; // 0.0 - 1.0
  emotional: number; // 0.0 - 1.0
  authoritative: number; // 0.0 - 1.0
  diplomatic: number; // 0.0 - 1.0
  curious: number; // 0.0 - 1.0
  cautious: number; // 0.0 - 1.0
  humorTendency: number; // 0.0 - 1.0
  leadershipStyle: LeadershipStyle;
  communicationStyle: CommunicationStyle;
}

export enum LeadershipStyle {
  Commanding = 'Commanding',
  Collaborative = 'Collaborative',
  Technical = 'Technical',
  Supportive = 'Supportive',
  Analytical = 'Analytical',
}

export enum CommunicationStyle {
  Direct = 'Direct',
  Diplomatic = 'Diplomatic',
  Technical = 'Technical',
  Philosophical = 'Philosophical',
  Empathetic = 'Empathetic',
  Formal = 'Formal',
}

export interface VoiceConfig {
  voiceActor: string;
  speechPatterns: SpeechPattern[];
  commonPhrases: string[];
  accent?: string;
  toneModulation: ToneModulation;
}

export interface SpeechPattern {
  patternType: SpeechPatternType;
  frequency: number; // 0.0 - 1.0
  examples: string[];
}

export enum SpeechPatternType {
  CatchPhrase = 'CatchPhrase',
  TechnicalJargon = 'TechnicalJargon',
  PhilosophicalReference = 'PhilosophicalReference',
  MilitaryProtocol = 'MilitaryProtocol',
  ScientificExplanation = 'ScientificExplanation',
  EmotionalExpression = 'EmotionalExpression',
  LogicalAnalysis = 'LogicalAnalysis',
}

export interface ToneModulation {
  basePitch: number;
  emotionalRange: number;
  formalityLevel: number;
  speechSpeed: number;
}

export interface CharacterAppearance {
  species: Species;
  heightCm: number;
  uniformColor: UniformColor;
  distinctiveFeatures: string[];
  holographicFidelity: FidelityLevel;
}

export enum Species {
  Human = 'Human',
  Android = 'Android',
  Klingon = 'Klingon',
  Vulcan = 'Vulcan',
  Betazoid = 'Betazoid',
  ElAurian = 'ElAurian',
  Bolian = 'Bolian',
}

export enum UniformColor {
  Command = 'Command',
  Sciences = 'Sciences',
  Operations = 'Operations',
  Civilian = 'Civilian',
}

export enum FidelityLevel {
  Basic = 'Basic',
  Standard = 'Standard',
  HighDefinition = 'HighDefinition',
  Photorealistic = 'Photorealistic',
}

export enum KnowledgeDomain {
  Starfleet = 'Starfleet',
  Engineering = 'Engineering',
  Medical = 'Medical',
  Psychology = 'Psychology',
  Physics = 'Physics',
  Diplomacy = 'Diplomacy',
  MilitaryTactics = 'MilitaryTactics',
  Philosophy = 'Philosophy',
  History = 'History',
  AlienCultures = 'AlienCultures',
  Technology = 'Technology',
  Navigation = 'Navigation',
}

export interface CharacterRelationship {
  otherCharacterId: string;
  relationshipType: RelationshipType;
  strength: number; // 0.0 - 1.0
  trustLevel: number; // 0.0 - 1.0
}

export enum RelationshipType {
  Superior = 'Superior',
  Subordinate = 'Subordinate',
  Colleague = 'Colleague',
  Friend = 'Friend',
  Mentor = 'Mentor',
  Rival = 'Rival',
  Family = 'Family',
  Romantic = 'Romantic',
}

export enum Mood {
  Calm = 'Calm',
  Excited = 'Excited',
  Concerned = 'Concerned',
  Focused = 'Focused',
  Curious = 'Curious',
  Amused = 'Amused',
  Serious = 'Serious',
  Contemplative = 'Contemplative',
}

export enum CharacterStatus {
  Active = 'Active',
  Inactive = 'Inactive',
  Busy = 'Busy',
  AwayFromStation = 'AwayFromStation',
  InMeeting = 'InMeeting',
  EmergencyProtocol = 'EmergencyProtocol',
}

export interface StoryTemplate {
  id: string;
  holodeckId: string;
  title: string;
  topic: string;
  scenes: SceneTemplate[];
  storyGraph: StoryGraph;
  metadata: StoryMetadata;
  createdAt: Date;
}

export interface SceneTemplate {
  id: string;
  sequenceNumber: number;
  title: string;
  description: string;
  environmentContext: string;
  charactersPresent: Character[];
  decisionPoints: DecisionPoint[];
  safetyRating: SafetyLevel | string;
}

export interface StoryGraph {
  nodes: { [key: string]: GraphNode };
  edges: GraphEdge[];
  startNode: string;
  endNodes: string[];
}

export interface GraphNode {
  id: string;
  sceneId: string;
  nodeType: NodeType;
  position: [number, number];
}

export enum NodeType {
  Start = 'Start',
  Scene = 'Scene',
  Decision = 'Decision',
  End = 'End',
}

export interface GraphEdge {
  from: string;
  to: string;
  condition?: string;
}

export interface DecisionPoint {
  id: string;
  description: string;
  decisionType: DecisionType;
  consequences: Consequence[];
  nextSceneId?: string;
}

export enum DecisionType {
  Continue = 'Continue',
  BranchLeft = 'BranchLeft',
  BranchRight = 'BranchRight',
  GoBack = 'GoBack',
  InteractCharacter = 'InteractCharacter',
  ExamineEnvironment = 'ExamineEnvironment',
}

export interface Consequence {
  type: string;
  description: string;
  impact: number; // -1.0 to 1.0
}

export interface StoryMetadata {
  generatedAt: Date;
  complexity: number;
  estimatedDuration: number;
  language: string;
  wordsPerScene: string;
}

export interface StoryBook {
  id: string;
  templateId: string;
  holodeckId: string;
  playerName: string;
  sessionName: string;
  playedScenes: PlayedScene[];
  currentPosition: Position3D;
  playerDecisions: PlayerDecision[];
  sessionStatistics: SessionStatistics;
  status: SessionStatus;
  startedAt: Date;
  lastPlayed: Date;
  completedAt?: Date;
}

export interface PlayedScene {
  id: string;
  title: string;
  description: string;
  characterInteractions: string[];
  playerDecisions: string[];
  completedAt: Date;
}

export interface PlayerDecision {
  id: string;
  sceneId: string;
  decision: string;
  timestamp: Date;
  responseTimeMs: number;
}

export interface SessionStatistics {
  totalScenesPlayed: number;
  totalDecisionsMade: number;
  averageResponseTimeMs: number;
  charactersInteractedWith: string[];
  storyPathsExplored: string[];
  safetyInterventions: number;
  achievementUnlocked: Achievement[];
}

export interface Achievement {
  id: string;
  name: string;
  description: string;
  unlockedAt: Date;
  iconUrl?: string;
}

// UI-specific types
export interface WelcomeData {
  playerName: string;
  storyTopic: string;
}

export interface PrepareStoryData {
  topic: string;
  sceneCount: number;
  language: string;
  storyType: HolodeckStoryType;
  wordsPerScene: string;
  selectedCharacters: Character[];
}

export interface MCPServerStatus {
  [serverName: string]: {
    status: 'online' | 'offline';
    latency: number;
  };
}

// System monitoring types
export interface ServiceStatus {
  name: string;
  status: 'online' | 'offline' | 'degraded';
  lastCheck: Date;
  responseTime?: number;
  errorCount?: number;
}

export interface SystemStatus {
  overallHealth: 'healthy' | 'degraded' | 'unhealthy';
  coordinator: ServiceStatus;
  servers: Record<string, ServiceStatus>;
  lastUpdated: Date;
  totalServers: number;
  healthyServers: number;
}