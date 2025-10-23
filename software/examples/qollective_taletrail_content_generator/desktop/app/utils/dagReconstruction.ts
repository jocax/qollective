/**
 * DAG Reconstruction Utility
 *
 * Reconstructs DAG (Directed Acyclic Graph) structure from trail_steps array.
 * The backend stores trails in normalized form with trail_steps containing
 * sequential content references. This utility rebuilds the complete DAG structure
 * needed for interactive navigation, linear reading, and graph visualization.
 */

import type { TrailStep, DAG, ContentNode, Edge, Choice } from '~/types/trails'

/**
 * Validate a choice has a valid next_node_id
 *
 * @param choice - Choice to validate
 * @param nodeId - ID of the node containing this choice
 * @returns true if choice is valid, false otherwise
 */
function validateChoice(choice: Choice, nodeId: string): boolean {
  if (!choice.next_node_id || choice.next_node_id === '') {
    console.warn('[DAG Reconstruction] Invalid choice in node ${nodeId}:', {
      choiceId: choice.id,
      choiceText: choice.text,
      issue: 'Missing or empty next_node_id'
    })
    return false
  }
  return true
}

/**
 * Reconstruct DAG structure from trail_steps array
 *
 * Builds the complete DAG with nodes, edges, and convergence points from
 * database-normalized trail_steps format. Each trail_step contains:
 * - step_order: Sequential order (1-indexed)
 * - content_reference: { temp_node_id, content }
 * - metadata: { node_id, convergence_point, incoming_edges, outgoing_edges }
 *
 * @param trail_steps - Array of trail steps from backend
 * @param start_node_id - ID of the starting node
 * @returns Reconstructed DAG with nodes, edges, and convergence points
 * @throws Error if trail_steps is empty or malformed
 */
export function reconstructDAG(
  trail_steps: TrailStep[],
  start_node_id: string
): DAG {
  if (!trail_steps || trail_steps.length === 0) {
    throw new Error('[DAG Reconstruction] Cannot reconstruct DAG: trail_steps array is empty')
  }

  if (!start_node_id) {
    throw new Error('[DAG Reconstruction] Cannot reconstruct DAG: start_node_id is missing')
  }

  console.log('[DAG Reconstruction] Starting reconstruction...', {
    stepCount: trail_steps.length,
    startNodeId: start_node_id
  })

  const nodes: Record<string, ContentNode> = {}
  const edges: Edge[] = []
  const convergence_points: string[] = []

  // Process each trail step
  for (const step of trail_steps) {
    try {
      // Extract node data from content_reference
      if (!step.content_reference) {
        console.warn('[DAG Reconstruction] Step missing content_reference:', step)
        continue
      }

      const node_id = step.content_reference.temp_node_id
      const content = step.content_reference.content

      if (!node_id || !content) {
        console.warn('[DAG Reconstruction] Step has invalid content_reference:', step)
        continue
      }

      // Build node structure
      const contentNode: ContentNode = {
        id: node_id,
        content: {
          text: content.text || '',
          choices: content.choices || []
        },
        incoming_edges: (step.metadata?.incoming_edges as number) || 0,
        outgoing_edges: (step.metadata?.outgoing_edges as number) || 0
      }

      // Add generation metadata if available (educational_content)
      if (content.educational_content) {
        contentNode.generation_metadata = {
          ...content.educational_content,
          timestamp: step.metadata?.timestamp as string | undefined,
          llm_model: step.metadata?.llm_model as string | undefined
        }
      }

      nodes[node_id] = contentNode

      // Extract edges from choices with validation
      if (content.choices && Array.isArray(content.choices)) {
        for (const choice of content.choices) {
          // Validate choice structure
          if (!choice.id) {
            console.warn('[DAG Reconstruction] Choice missing id in node ${node_id}:', choice)
            continue
          }

          // Validate next_node_id
          if (!validateChoice(choice, node_id)) {
            // Warning already logged by validateChoice
            continue
          }

          // Create edge for valid choice
          edges.push({
            from_node_id: node_id,
            to_node_id: choice.next_node_id,
            choice_id: choice.id
          })
        }
      }

      // Collect convergence points
      if (step.metadata?.convergence_point === true || content.convergence_point === true) {
        convergence_points.push(node_id)
      }

    } catch (stepErr) {
      console.error('[DAG Reconstruction] Error processing step:', stepErr, step)
      // Continue processing other steps even if one fails
    }
  }

  const dag: DAG = {
    nodes,
    edges,
    start_node_id,
    convergence_points
  }

  console.log('[DAG Reconstruction] Reconstruction complete:', {
    nodeCount: Object.keys(nodes).length,
    edgeCount: edges.length,
    convergencePointCount: convergence_points.length,
    startNodeId: start_node_id,
    hasStartNode: start_node_id in nodes
  })

  // Validate that start node exists
  if (!(start_node_id in nodes)) {
    console.error('[DAG Reconstruction] Warning: start_node_id not found in reconstructed nodes!', {
      startNodeId: start_node_id,
      availableNodeIds: Object.keys(nodes).slice(0, 5) // Show first 5 for debugging
    })
  }

  return dag
}

/**
 * Validate reconstructed DAG structure
 *
 * Performs sanity checks on the reconstructed DAG to ensure it's valid
 * for navigation and visualization.
 *
 * @param dag - Reconstructed DAG to validate
 * @returns Object with validation results and any warnings
 */
export function validateDAG(dag: DAG): {
  valid: boolean
  warnings: string[]
  stats: {
    nodeCount: number
    edgeCount: number
    convergencePointCount: number
    orphanNodes: number
    deadEndNodes: number
  }
} {
  const warnings: string[] = []
  const nodeIds = Object.keys(dag.nodes)
  const nodeCount = nodeIds.length
  const edgeCount = dag.edges.length
  const convergencePointCount = dag.convergence_points?.length || 0

  // Check for start node
  if (!(dag.start_node_id in dag.nodes)) {
    warnings.push(`Start node "${dag.start_node_id}" not found in DAG nodes`)
  }

  // Count nodes with no incoming edges (orphans)
  const nodesWithIncoming = new Set(dag.edges.map(e => e.to_node_id))
  const orphanNodes = nodeIds.filter(id => id !== dag.start_node_id && !nodesWithIncoming.has(id))

  if (orphanNodes.length > 0) {
    warnings.push(`Found ${orphanNodes.length} orphan nodes with no incoming edges`)
  }

  // Count nodes with no outgoing edges (dead ends)
  const nodesWithOutgoing = new Set(dag.edges.map(e => e.from_node_id))
  const deadEndNodes = nodeIds.filter(id => !nodesWithOutgoing.has(id))

  // Check for broken edge references
  const brokenEdges = dag.edges.filter(edge =>
    !(edge.from_node_id in dag.nodes) || !(edge.to_node_id in dag.nodes)
  )

  if (brokenEdges.length > 0) {
    warnings.push(`Found ${brokenEdges.length} edges with invalid node references`)
  }

  return {
    valid: warnings.length === 0 || (warnings.length === 1 && warnings[0].includes('dead ends')),
    warnings,
    stats: {
      nodeCount,
      edgeCount,
      convergencePointCount,
      orphanNodes: orphanNodes.length,
      deadEndNodes: deadEndNodes.length
    }
  }
}
