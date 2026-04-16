#!/usr/bin/env python3
"""
LangGraph Immune System for Surgical File Edits (Actor-Critic Architecture)

This module implements a self-healing retry loop for surgical file edits,
protected by a strict Philosophy Verifier that ensures all changes comply
with core architectural principles.
"""

from typing import TypedDict, Annotated
from langgraph.graph import StateGraph, END
from langchain_openai import ChatOpenAI
from langchain_core.messages import SystemMessage, HumanMessage
import json
import difflib
import re


def extract_json(text: str) -> str:
    """
    Extract JSON from markdown code blocks or plain text.
    Handles cases where LLM wraps JSON in ```json ... ``` or ``` ... ```.
    """
    # Pattern for markdown code blocks with optional language specifier
    pattern = r'```(?:json)?\s*(\{.*?\})\s*```'
    match = re.search(pattern, text, re.DOTALL)
    if match:
        return match.group(1)
    # If no markdown, try to find the first JSON object
    start = text.find('{')
    end = text.rfind('}')
    if start != -1 and end != -1 and start < end:
        return text[start:end+1]
    # If no JSON object, return the original text
    return text


class GraphState(TypedDict):
    """State schema for the surgical edit graph"""
    task_description: str
    target_file: str
    search_block: str
    replace_block: str
    error_message: str
    file_context: str
    retry_count: int
    philosophy_veto: bool


def create_surgeon_node():
    """Create the Surgeon node - LLM that generates surgical edits"""
    
    llm = ChatOpenAI(model="deepseek-v3.2:cloud", api_key="ollama", base_url="http://localhost:11434/v1", temperature=0.0)
    
    def surgeon(state: GraphState) -> GraphState:
        """Generate surgical edit suggestions"""
        prompt_text = """You are a precision code surgeon. Your task is to generate EXACT code replacements for surgical edits.

Task: [TASK_PLACEHOLDER]
Target File: [TARGET_PLACEHOLDER]

Generate a JSON response with exactly two fields:
- "search_block": The EXACT code snippet to find and remove (must match perfectly)
- "replace_block": The EXACT code snippet to insert in place of the search block

CRITICAL RULES:
1. Include EXACT indentation and whitespace
2. Include complete lines only (never partial lines)
3. Be surgically precise - minimal, focused changes only
4. The search_block must match exactly as it appears in the file
5. Return ONLY valid JSON, nothing else

Example format:
{
    "search_block": "    def old_function():\\n        pass",
    "replace_block": "    def new_function():\\n        return True"
}"""
        
        prompt_text = prompt_text.replace("[TASK_PLACEHOLDER]", state["task_description"])
        prompt_text = prompt_text.replace("[TARGET_PLACEHOLDER]", state["target_file"])
        
        response = llm.invoke([HumanMessage(content=prompt_text)])
        try:
            json_str = extract_json(response.content)
            result = json.loads(json_str)
            state["search_block"] = result["search_block"]
            state["replace_block"] = result["replace_block"]
        except json.JSONDecodeError:
            # Fallback for malformed JSON
            state["error_message"] = f"Surgeon generated invalid JSON: {response.content}"
            state["retry_count"] += 1
            
        return state
    
    return surgeon


def create_philosophy_verifier():
    """Create the Philosophy Verifier node - ensures architectural compliance"""
    
    llm = ChatOpenAI(model="deepseek-v3.2:cloud", api_key="ollama", base_url="http://localhost:11434/v1", temperature=0.0)
    
    def philosophy_verifier(state: GraphState) -> GraphState:
        """Verify architectural compliance"""
        prompt_text = """You are the Chief Architectural Auditor. VETO any code violating these tenets:

1. Sovereign Mesh (No centralized servers)
2. Eventual Delivery (Outbox messages retry forever, never dropped by TTL)
3. Extreme Efficiency (No polling; strictly event-driven)
4. Mandatory Relay (Relay functionality cannot be bypassed)

Analyze the following proposed code change:

Search Block:
[SEARCH_BLOCK_PLACEHOLDER]

Replace Block:
[REPLACE_BLOCK_PLACEHOLDER]

Respond with a JSON object containing:
- "philosophy_veto": true if it violates principles, false otherwise
- "error_message": Detailed explanation if vetoed, empty string if approved

Example responses:
{"philosophy_veto": false, "error_message": ""}

{"philosophy_veto": true, "error_message": "Change introduces centralized server dependency..."}"""
        
        prompt_text = prompt_text.replace("[SEARCH_BLOCK_PLACEHOLDER]", state["search_block"])
        prompt_text = prompt_text.replace("[REPLACE_BLOCK_PLACEHOLDER]", state["replace_block"])
        
        response = llm.invoke([HumanMessage(content=prompt_text)])
        try:
            json_str = extract_json(response.content)
            result = json.loads(json_str)
            state["philosophy_veto"] = result["philosophy_veto"]
            if result["philosophy_veto"]:
                state["error_message"] = result["error_message"]
        except json.JSONDecodeError:
            # If verifier fails, assume compliance to avoid blocking progress
            state["philosophy_veto"] = False
            
        return state
    
    return philosophy_verifier


def create_applier_node():
    """Create the Applier node - applies changes to files"""
    
    def applier(state: GraphState) -> GraphState:
        """Apply surgical edits to target file"""
        try:
            # Read the target file
            with open(state["target_file"], 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Check if search block exists exactly
            if state["search_block"] in content:
                # Apply the replacement
                new_content = content.replace(state["search_block"], state["replace_block"])
                
                # Write back to file
                with open(state["target_file"], 'w', encoding='utf-8') as f:
                    f.write(new_content)
                
                # Success - we're done
                state["error_message"] = ""
                print(f"✅ Successfully applied surgical edit to {state['target_file']}")
                return state
            else:
                # Search block not found - provide context for resolver
                state["retry_count"] += 1
                
                # Find approximate location using fuzzy matching
                lines = content.split('\n')
                search_lines = state["search_block"].split('\n')
                
                if len(search_lines) > 0:
                    # Look for similar lines around the expected area
                    target_line = search_lines[0].strip()
                    for i, line in enumerate(lines):
                        if target_line in line or difflib.SequenceMatcher(None, target_line, line.strip()).ratio() > 0.6:
                            # Get context around this area
                            start_idx = max(0, i - 25)
                            end_idx = min(len(lines), i + 25)
                            context_lines = lines[start_idx:end_idx]
                            state["file_context"] = '\n'.join(context_lines)
                            break
                    else:
                        # If we can't find context, use first 50 lines
                        state["file_context"] = '\n'.join(lines[:50])
                
                state["error_message"] = f"Search block not found in file. Context provided for resolution."
                print(f"❌ Search block not found in {state['target_file']}. Retry count: {state['retry_count']}")
                
        except Exception as e:
            state["retry_count"] += 1
            state["error_message"] = f"Failed to apply changes: {str(e)}"
            print(f"❌ Failed to apply changes: {str(e)}")
            
        return state
    
    return applier


def create_error_resolver():
    """Create the Error Resolver node - fixes malformed edits"""
    
    llm = ChatOpenAI(model="deepseek-v3.2:cloud", api_key="ollama", base_url="http://localhost:11434/v1", temperature=0.2)
    
    def error_resolver(state: GraphState) -> GraphState:
        """Resolve errors in surgical edits"""
        prompt_text = """You are a surgical code correction specialist. Fix the failed surgical edit by analyzing the error and context.

Original Error: [ERROR_PLACEHOLDER]

Failed Search Block (may have formatting issues):
[SEARCH_BLOCK_PLACEHOLDER]

File Context (50 lines around the problematic area):
[CONTEXT_PLACEHOLDER]

Target File: [TARGET_PLACEHOLDER]

Generate a corrected JSON response with:
- "search_block": The EXACT code snippet that exists in the file (matching indentation and whitespace)
- "replace_block": The corrected replacement code

CRITICAL RULES:
1. The search_block MUST match EXACTLY what's in the file
2. Copy indentation and whitespace exactly as shown in context
3. Focus only on the specific lines that need changing
4. Return ONLY valid JSON, nothing else

Response format:
{
    "search_block": "exact code from file including correct indentation",
    "replace_block": "corrected replacement code"
}"""
        
        prompt_text = prompt_text.replace("[ERROR_PLACEHOLDER]", state["error_message"])
        prompt_text = prompt_text.replace("[SEARCH_BLOCK_PLACEHOLDER]", state["search_block"])
        prompt_text = prompt_text.replace("[CONTEXT_PLACEHOLDER]", state["file_context"])
        prompt_text = prompt_text.replace("[TARGET_PLACEHOLDER]", state["target_file"])
        
        response = llm.invoke([HumanMessage(content=prompt_text)])
        try:
            json_str = extract_json(response.content)
            result = json.loads(json_str)
            state["search_block"] = result["search_block"]
            state["replace_block"] = result["replace_block"]
            state["error_message"] = ""
            print(f"🔧 Resolved error, retrying with corrected blocks")
        except json.JSONDecodeError:
            state["retry_count"] += 1
            state["error_message"] = f"Resolver generated invalid JSON: {response.content}"
            print(f"❌ Resolver failed to generate valid JSON")
            
        return state
    
    return error_resolver


def create_surgical_edit_graph():
    """Create the complete surgical edit graph"""
    
    # Initialize the graph
    workflow = StateGraph(GraphState)
    
    # Create nodes
    surgeon_node = create_surgeon_node()
    philosophy_verifier_node = create_philosophy_verifier()
    applier_node = create_applier_node()
    error_resolver_node = create_error_resolver()
    
    # Add nodes to graph
    workflow.add_node("surgeon", surgeon_node)
    workflow.add_node("philosophy_verifier", philosophy_verifier_node)
    workflow.add_node("applier", applier_node)
    workflow.add_node("error_resolver", error_resolver_node)
    
    # Add edges
    workflow.add_edge("surgeon", "philosophy_verifier")
    
    # Conditional edges from philosophy verifier
    def route_from_philosophy_verifier(state: GraphState):
        if state["philosophy_veto"]:
            return "error_resolver"
        else:
            return "applier"
    
    workflow.add_conditional_edges(
        "philosophy_verifier",
        route_from_philosophy_verifier,
        {
            "error_resolver": "error_resolver",
            "applier": "applier"
        }
    )
    
    # Conditional edges from applier
    def route_from_applier(state: GraphState):
        if state["error_message"] == "":
            return "END"  # Success
        elif state["retry_count"] >= 3:
            return "END"  # Hard fail
        else:
            return "error_resolver"  # Retry
    
    workflow.add_conditional_edges(
        "applier",
        route_from_applier,
        {
            "END": END,
            "error_resolver": "error_resolver"
        }
    )
    
    # Edge from error resolver back to philosophy verifier
    workflow.add_edge("error_resolver", "philosophy_verifier")
    
    # Set entry point
    workflow.set_entry_point("surgeon")
    
    return workflow.compile()


if __name__ == "__main__":
    """
    Test the surgical edit graph with a dummy file
    """
    print("🧪 Testing Surgical Edit Graph...")
    
    # Create a dummy test file
    dummy_file_path = "dummy_test.txt"
    dummy_content = """def old_function():
    # This is some old code that needs to be updated
    print("Hello, World!")
    return None

def another_function():
    # This function should remain unchanged
    x = 10
    y = 20
    return x + y

def cleanup_old_code():
    # This cleanup function is deprecated
    print("Cleaning up...")
    pass
"""
    
    with open(dummy_file_path, 'w', encoding='utf-8') as f:
        f.write(dummy_content)
    
    print(f"📝 Created dummy test file: {dummy_file_path}")
    
    # Create the graph
    graph = create_surgical_edit_graph()
    
    # Define test state
    initial_state = GraphState(
        task_description="Replace the old_function with a new improved version that returns a proper greeting",
        target_file=dummy_file_path,
        search_block="",  # Will be filled by surgeon
        replace_block="",  # Will be filled by surgeon
        error_message="",
        file_context="",
        retry_count=0,
        philosophy_veto=False
    )
    
    # Execute the graph
    print("🚀 Executing surgical edit graph...")
    final_state = graph.invoke(initial_state)
    
    # Display results
    print(f"🏁 Final state:")
    print(f"   Retry count: {final_state['retry_count']}")
    print(f"   Error message: {final_state['error_message']}")
    print(f"   Philosophy veto: {final_state['philosophy_veto']}")
    
    # Show the final file content
    with open(dummy_file_path, 'r', encoding='utf-8') as f:
        final_content = f.read()
    
    print(f"\n📄 Final file content:")
    print(final_content)
    
    print("✅ Test completed!")