// ABOUTME: Comprehensive mock data service simulating all MCP server interactions  
// ABOUTME: Provides realistic delays, character responses, and holodeck simulation for Phase 4 UI

import type { 
  Holodeck, 
  StoryTemplate, 
  StoryBook, 
  Character,
  HolodeckConfig,
} from '../types/holodeck';
import { HolodeckStoryType, SessionStatus } from '../types/holodeck';

interface WelcomeData {
  playerName: string;
  storyTopic: string;
}

interface PrepareStoryData {
  topic: string;
  sceneCount: number;
  language: string;
  storyType: HolodeckStoryType;
  wordsPerScene: string;
  selectedCharacters: Character[];
}

interface PlayedScene {
  id: string;
  title: string;
  description: string;
  characterInteractions: string[];
  playerDecisions: string[];
  completedAt: Date;
}

export class MockDataService {
  private static instance: MockDataService;
  private holodecks: Holodeck[] = [];
  private storyTemplates: StoryTemplate[] = [];
  private storyBooks: StoryBook[] = [];
  private currentSession: StoryBook | null = null;
  
  static getInstance(): MockDataService {
    if (!MockDataService.instance) {
      MockDataService.instance = new MockDataService();
    }
    return MockDataService.instance;
  }

  private constructor() {
    this.initializeMockData();
  }

  private initializeMockData() {
    // Initialize with some sample historical sessions
    const historicalSession: StoryBook = {
      id: crypto.randomUUID(),
      templateId: crypto.randomUUID(),
      holodeckId: crypto.randomUUID(),
      playerName: "Commander Test",
      sessionName: "First Contact Protocol - Previous Session",
      playedScenes: [
        {
          id: crypto.randomUUID(),
          title: "Bridge Conference",
          description: "Captain Picard calls a senior staff meeting to discuss the approaching alien vessel. The viewscreen shows a sleek, unfamiliar design with energy readings that don't match any known species.",
          characterInteractions: [
            "Picard: 'Number One, what's our current status?'",
            "Riker: 'All departments report ready, Captain. The vessel is maintaining position at the edge of sensor range.'",
            "Data: 'Fascinating. Their hull configuration suggests a propulsion system I have not encountered before.'"
          ],
          playerDecisions: ["Approached diplomatically", "Asked Data for analysis"],
          completedAt: new Date(Date.now() - 86400000) // 1 day ago
        }
      ],
      currentPosition: { x: 0, y: 0, z: 0 },
      playerDecisions: [],
      sessionStatistics: {
        totalScenesPlayed: 1,
        totalDecisionsMade: 2,
        averageResponseTimeMs: 5000,
        charactersInteractedWith: ["picard", "riker", "data"],
        storyPathsExplored: ["diplomatic_approach"],
        safetyInterventions: 0,
        achievementUnlocked: []
      },
      status: SessionStatus.Completed,
      startedAt: new Date(Date.now() - 90000000),
      lastPlayed: new Date(Date.now() - 86400000),
      completedAt: new Date(Date.now() - 86000000)
    };

    this.storyBooks.push(historicalSession);
  }

  // Mock holodeck creation with realistic delays
  async createHolodeck(data: WelcomeData): Promise<Holodeck> {
    await this.simulateDelay(800, 1200);
    
    const holodeck: Holodeck = {
      id: crypto.randomUUID(),
      name: `${data.storyTopic} - ${data.playerName}`,
      topic: data.storyTopic,
      storyType: HolodeckStoryType.Adventure,
      participants: [],
      currentScene: undefined,
      configuration: this.getDefaultConfig(),
      createdAt: new Date(),
      updatedAt: new Date(),
    };
    
    this.holodecks.push(holodeck);
    return holodeck;
  }

  // Mock story template generation with realistic AI delay
  async generateStoryTemplate(prepareData: PrepareStoryData): Promise<StoryTemplate> {
    await this.simulateDelay(3000, 5000); // Simulate AI generation time
    
    const template: StoryTemplate = {
      id: crypto.randomUUID(),
      holodeckId: crypto.randomUUID(),
      title: `${prepareData.topic} - Adventure Template`,
      topic: prepareData.topic,
      scenes: this.generateMockScenes(prepareData),
      storyGraph: this.generateMockStoryGraph(prepareData),
      metadata: {
        generatedAt: new Date(),
        complexity: this.calculateComplexity(prepareData),
        estimatedDuration: this.estimateDuration(prepareData),
        language: prepareData.language,
        wordsPerScene: prepareData.wordsPerScene,
      },
      createdAt: new Date(),
    };
    
    this.storyTemplates.push(template);
    return template;
  }

  // Create new story session from template
  async createStorySession(template: StoryTemplate, playerName: string): Promise<StoryBook> {
    await this.simulateDelay(500, 800);
    
    const session: StoryBook = {
      id: crypto.randomUUID(),
      templateId: template.id,
      holodeckId: template.holodeckId,
      playerName,
      sessionName: template.title,
      playedScenes: [],
      currentPosition: { x: 0, y: 0, z: 0 },
      playerDecisions: [],
      sessionStatistics: {
        totalScenesPlayed: 0,
        totalDecisionsMade: 0,
        averageResponseTimeMs: 0,
        charactersInteractedWith: [],
        storyPathsExplored: [],
        safetyInterventions: 0,
        achievementUnlocked: [],
      },
      status: SessionStatus.Active,
      startedAt: new Date(),
      lastPlayed: new Date(),
      completedAt: undefined,
    };
    
    this.currentSession = session;
    this.storyBooks.push(session);
    return session;
  }

  // Get next scene for current session
  async getNextScene(sessionId: string, decision?: string): Promise<{
    scene: {
      id: string;
      title: string;
      description: string;
      environmentContext: string;
      characterDialogue: string[];
      availableDecisions: {
        id: string;
        text: string;
        consequence: string;
      }[];
    };
    isLastScene: boolean;
  }> {
    await this.simulateDelay(1000, 2000);
    
    const session = this.storyBooks.find(s => s.id === sessionId);
    if (!session) {
      throw new Error(`Session ${sessionId} not found`);
    }

    const sceneNumber = session.playedScenes.length + 1;
    const isLastScene = sceneNumber >= 5; // Assume 5 scenes per story

    const scene = {
      id: crypto.randomUUID(),
      title: `Scene ${sceneNumber}: ${this.generateSceneTitle(sceneNumber)}`,
      description: this.generateSceneDescription(sceneNumber, decision),
      environmentContext: this.generateEnvironmentDescription(sceneNumber),
      characterDialogue: this.generateCharacterDialogue(sceneNumber),
      availableDecisions: isLastScene ? [] : [
        {
          id: 'continue',
          text: 'Continue forward with the current plan',
          consequence: 'Proceed to the next objective location'
        },
        {
          id: 'investigate',
          text: 'Take time to investigate the surroundings',
          consequence: 'Gather additional information before proceeding'
        },
        {
          id: 'consult_crew',
          text: 'Consult with crew members for their opinions',
          consequence: 'Get insights from your selected characters'
        }
      ]
    };

    return { scene, isLastScene };
  }

  // Record player decision
  async recordDecision(sessionId: string, sceneId: string, decision: string): Promise<void> {
    await this.simulateDelay(200, 400);
    
    const session = this.storyBooks.find(s => s.id === sessionId);
    if (!session) {
      throw new Error(`Session ${sessionId} not found`);
    }

    session.playerDecisions.push({
      id: crypto.randomUUID(),
      sceneId,
      decision,
      timestamp: new Date(),
      responseTimeMs: Math.floor(Math.random() * 10000) + 2000,
    });

    session.sessionStatistics.totalDecisionsMade++;
    session.lastPlayed = new Date();
  }

  // Complete a scene
  async completeScene(sessionId: string, sceneData: PlayedScene): Promise<void> {
    await this.simulateDelay(300, 600);
    
    const session = this.storyBooks.find(s => s.id === sessionId);
    if (!session) {
      throw new Error(`Session ${sessionId} not found`);
    }

    session.playedScenes.push(sceneData);
    session.sessionStatistics.totalScenesPlayed++;
    session.lastPlayed = new Date();

    // Update statistics
    const totalResponseTime = session.playerDecisions.reduce((sum, d) => sum + d.responseTimeMs, 0);
    session.sessionStatistics.averageResponseTimeMs = Math.floor(totalResponseTime / session.playerDecisions.length);
  }

  // Complete session
  async completeSession(sessionId: string): Promise<void> {
    await this.simulateDelay(500, 800);
    
    const session = this.storyBooks.find(s => s.id === sessionId);
    if (!session) {
      throw new Error(`Session ${sessionId} not found`);
    }

    session.status = SessionStatus.Completed;
    session.completedAt = new Date();
    session.lastPlayed = new Date();
    
    if (this.currentSession?.id === sessionId) {
      this.currentSession = null;
    }
  }

  // Get session history
  async getSessionHistory(): Promise<StoryBook[]> {
    await this.simulateDelay(300, 600);
    return [...this.storyBooks].sort((a, b) => b.lastPlayed.getTime() - a.lastPlayed.getTime());
  }

  // Get session by ID
  async getSessionById(sessionId: string): Promise<StoryBook | null> {
    await this.simulateDelay(200, 400);
    return this.storyBooks.find(s => s.id === sessionId) || null;
  }

  // Resume session
  async resumeSession(sessionId: string): Promise<StoryBook> {
    await this.simulateDelay(500, 800);
    
    const session = this.storyBooks.find(s => s.id === sessionId);
    if (!session) {
      throw new Error(`Session ${sessionId} not found`);
    }

    session.status = SessionStatus.Active;
    session.lastPlayed = new Date();
    this.currentSession = session;
    
    return session;
  }

  // Mock character interaction with personality-based responses
  async getCharacterResponse(
    characterId: string, 
    context: string, 
    playerAction: string
  ): Promise<string> {
    await this.simulateDelay(500, 1000);
    
    const character = MOCK_CHARACTERS.find(c => c.id === characterId);
    if (!character) {
      throw new Error(`Character ${characterId} not found`);
    }
    
    return this.generateCharacterResponse(character, context, playerAction);
  }

  // Get real-time MCP server status (simulated)
  async getMCPServerStatus(): Promise<{
    holodeck_coordinator: { status: 'online' | 'offline', latency: number };
    holodeck_designer: { status: 'online' | 'offline', latency: number };
    holodeck_validator: { status: 'online' | 'offline', latency: number };
    holodeck_environment: { status: 'online' | 'offline', latency: number };
    holodeck_safety: { status: 'online' | 'offline', latency: number };
    holodeck_character: { status: 'online' | 'offline', latency: number };
    holodeck_storybook: { status: 'online' | 'offline', latency: number };
  }> {
    await this.simulateDelay(100, 300);
    
    return {
      holodeck_coordinator: { status: 'online', latency: Math.floor(Math.random() * 50) + 10 },
      holodeck_designer: { status: 'online', latency: Math.floor(Math.random() * 200) + 50 },
      holodeck_validator: { status: 'online', latency: Math.floor(Math.random() * 100) + 20 },
      holodeck_environment: { status: 'online', latency: Math.floor(Math.random() * 150) + 30 },
      holodeck_safety: { status: 'online', latency: Math.floor(Math.random() * 80) + 15 },
      holodeck_character: { status: 'online', latency: Math.floor(Math.random() * 120) + 40 },
      holodeck_storybook: { status: 'online', latency: Math.floor(Math.random() * 60) + 20 },
    };
  }

  // Private helper methods
  private simulateDelay(min: number, max: number): Promise<void> {
    const delay = Math.random() * (max - min) + min;
    return new Promise(resolve => setTimeout(resolve, delay));
  }

  private getDefaultConfig(): HolodeckConfig {
    return {
      safetyLevel: 'Standard',
      maxParticipants: 5,
      durationMinutes: undefined,
      autoSaveEnabled: true,
      voiceRecognition: true,
      hapticFeedback: true,
      replicatorAccess: false,
      transporterIntegration: false,
      environmentalControls: {
        temperatureCelsius: 22.0,
        humidityPercent: 45.0,
        atmosphericPressure: 101.325,
        oxygenLevel: 21.0,
        windSimulation: false,
        weatherEffects: false,
      }
    };
  }

  private generateMockScenes(prepareData: PrepareStoryData): any[] {
    const sceneCount = prepareData.sceneCount;
    const scenes = [];
    
    for (let i = 1; i <= sceneCount; i++) {
      scenes.push({
        id: crypto.randomUUID(),
        sequenceNumber: i,
        title: `Scene ${i}: ${this.generateSceneTitle(i)}`,
        description: this.generateSceneDescription(i),
        environmentContext: this.generateEnvironmentDescription(i),
        charactersPresent: prepareData.selectedCharacters.slice(0, 3),
        decisionPoints: [],
        safetyRating: 'Standard',
      });
    }
    
    return scenes;
  }

  private generateMockStoryGraph(prepareData: PrepareStoryData): any {
    return {
      nodes: {},
      edges: [],
      startNode: crypto.randomUUID(),
      endNodes: [],
    };
  }

  private calculateComplexity(prepareData: PrepareStoryData): number {
    return prepareData.sceneCount * 0.8 + prepareData.selectedCharacters.length * 0.2;
  }

  private estimateDuration(prepareData: PrepareStoryData): number {
    const wordsMultiplier = {
      'Short': 1,
      'Medium': 1.5,
      'Long': 2,
    };
    return prepareData.sceneCount * 10 * (wordsMultiplier[prepareData.wordsPerScene as keyof typeof wordsMultiplier] || 1);
  }

  private generateSceneTitle(sceneNumber: number): string {
    const titles = [
      "The Mission Begins",
      "Unexpected Discovery",
      "Critical Decision Point",
      "Confronting the Challenge", 
      "Resolution and Consequences",
      "New Revelations",
      "The Final Moment",
    ];
    return titles[sceneNumber - 1] || `Scene ${sceneNumber}`;
  }

  private generateSceneDescription(sceneNumber: number, previousDecision?: string): string {
    const descriptions = [
      "The bridge of the USS Enterprise hums with quiet efficiency as your mission begins. Captain Picard stands at the center of the command area, studying the main viewscreen which displays the swirling colors of a distant nebula. The mission parameters are clear, but the path ahead holds many unknowns.",
      
      `${previousDecision ? 'Following your previous decision, you' : 'You'} discover something unexpected that changes the nature of your mission. The corridor ahead splits into multiple paths, each leading to a different potential outcome. The crew looks to you for guidance on how to proceed.`,
      
      "The situation has reached a critical juncture. All eyes are on you as the team faces a decision that will determine the success or failure of the entire mission. The environmental systems warn of changing conditions that require immediate action.",
      
      "Your earlier choices have led to this moment of confrontation. Whether diplomatic, scientific, or tactical in nature, the challenge before you tests not only your skills but your principles. The crew stands ready to support whatever course of action you choose.",
      
      "The consequences of your decisions become clear as the mission nears its conclusion. The outcomes, both intended and unforeseen, shape the final moments of this adventure. What seemed like a simple task has evolved into something far more significant.",
    ];
    
    return descriptions[sceneNumber - 1] || "The adventure continues with new challenges and opportunities ahead.";
  }

  private generateEnvironmentDescription(sceneNumber: number): string {
    const environments = [
      "The bridge of the USS Enterprise hums with quiet efficiency. The main viewscreen shows the swirling colors of a nebula, casting ethereal light across the command center. Console displays flicker with streams of data as the crew monitors ship systems.",
      
      "You find yourself in Ten Forward, the ship's recreational area. Large windows provide a stunning view of passing stars, while crew members converse quietly at nearby tables. The atmosphere is relaxed but tinged with anticipation.",
      
      "The holodeck safety arch flickers as the program loads around you. Stone walls materialize, creating a medieval castle environment. Torches flicker in wall sconces, casting dancing shadows. The air carries the scent of aged wood and distant cooking fires.",
      
      "Engineering thrums with the steady pulse of the warp core. Diagnostic panels flash with streams of technical data as the engineering team monitors critical ship systems. The ambient temperature is warmer here, and the constant hum of machinery creates a rhythmic backdrop.",
      
      "The observation lounge offers a panoramic view of space through its large windows. Stars streak past as the ship travels at warp speed. The room is quietly furnished with comfortable seating arranged to take advantage of the spectacular view.",
    ];
    
    return environments[sceneNumber - 1] || "The environment shifts to match the needs of your current situation.";
  }

  private generateCharacterDialogue(sceneNumber: number): string[] {
    const dialogueSets = [
      [
        "Picard: 'Number One, report on our current status.'",
        "Riker: 'All departments ready, Captain. We're approaching the designated coordinates.'",
        "Data: 'I am detecting unusual readings from this sector. Fascinating.'"
      ],
      [
        "Riker: 'Captain, we have a situation developing here.'", 
        "Picard: 'Options, Number One?'",
        "Data: 'I calculate several possible courses of action, each with distinct probabilities of success.'"
      ],
      [
        "Picard: 'The decision rests with you. What are your recommendations?'",
        "Riker: 'Whatever you decide, the crew is behind you.'",
        "Data: 'The logical choice may not always be the correct one in situations involving sentient beings.'"
      ],
      [
        "Data: 'The situation has evolved beyond our initial parameters.'",
        "Picard: 'Adaptability is one of our greatest strengths as explorers.'",
        "Riker: 'Let's see what we can make of this new development.'"
      ],
      [
        "Picard: 'Mission accomplished, though not quite as we anticipated.'",
        "Riker: 'The unexpected outcomes may prove as valuable as our original objectives.'",
        "Data: 'I find this result... illuminating. It will require further analysis.'"
      ]
    ];
    
    return dialogueSets[sceneNumber - 1] || ["The crew exchanges meaningful glances, ready for whatever comes next."];
  }

  private generateCharacterResponse(character: any, context: string, playerAction: string): string {
    const responses: { [key: string]: string[] } = {
      'picard': [
        "Make it so. Your approach demonstrates sound judgment.",
        "I concur with your assessment. Proceed with caution.",
        "An intriguing proposition. Let us see where this path leads us.",
        "As Shakespeare once wrote, 'We know what we are, but know not what we may be.'",
      ],
      'riker': [
        "Understood. I'll coordinate with the department heads.",
        "Good thinking. That approach gives us more options.",
        "I like it. Sometimes the direct approach is the best approach.",
        "You've got it. The crew is ready to execute on your decision.",
      ],
      'data': [
        "Fascinating. I had not considered that possibility.",
        "Your logic is sound. I calculate a 73.6% probability of success.",
        "I do not understand the emotional component, but I concur with the strategy.",
        "That approach exhibits what humans call 'thinking outside the box.'",
      ]
    };
    
    const characterResponses = responses[character.id] || ["An interesting choice. Let us proceed."];
    return characterResponses[Math.floor(Math.random() * characterResponses.length)] || "An interesting choice. Let us proceed.";
  }
}

// Mock character database with authentic Star Trek personalities
const MOCK_CHARACTERS = [
  {
    id: "picard",
    name: "Jean-Luc Picard", 
    rank: "Captain",
    position: "Commanding Officer",
    personalityTraits: ["Diplomatic", "Intellectual", "Principled", "Strategic"],
    specialties: ["Leadership", "Diplomacy", "History", "Archaeology"],
    availability: "Always",
  },
  {
    id: "riker", 
    name: "William Thomas Riker",
    rank: "Commander",
    position: "First Officer",
    personalityTraits: ["Charismatic", "Bold", "Loyal", "Decisive"],
    specialties: ["Tactics", "Leadership", "Piloting", "Music"],
    availability: "Always",
  },
  {
    id: "data",
    name: "Data",
    rank: "Lt. Commander", 
    position: "Operations Officer",
    personalityTraits: ["Logical", "Curious", "Analytical", "Loyal"],
    specialties: ["Computing", "Science", "Engineering", "Research"],
    availability: "Always",
  },
];