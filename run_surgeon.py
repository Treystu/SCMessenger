#!/usr/bin/env python3
"""
Run surgical edit to add calculate_dynamic_ttl function to AdaptiveTTLManager.
"""

import sys
import os

# Add the scmessenger_swarm directory to the path so we can import surgeon_graph
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'AgentSwarmCline', 'scmessenger_swarm'))

from surgeon_graph import create_surgical_edit_graph, GraphState

def main():
    """Run the surgical edit graph."""
    graph = create_surgical_edit_graph()
    
    state = GraphState(
        task_description="Add calculate_dynamic_ttl function to AdaptiveTTLManager. If battery < 20, halve TTL. If peers < 3, double TTL.",
        target_file="core/src/routing/adaptive_ttl.rs",
        search_block="",
        replace_block="",
        error_message="",
        file_context="",
        retry_count=0,
        philosophy_veto=False
    )
    
    print("🚀 Starting surgical edit...")
    result = graph.invoke(state)
    print(f"🏁 Surgical edit completed with retry_count={result['retry_count']}, error_message={result['error_message']}")
    
    if result['error_message']:
        print(f"❌ Failed: {result['error_message']}")
        return 1
    else:
        print("✅ Success!")
        return 0

if __name__ == "__main__":
    sys.exit(main())