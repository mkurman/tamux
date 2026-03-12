export interface PluginAssistantToolDefinition {
    type: "function";
    function: {
        name: string;
        description: string;
        parameters: Record<string, unknown>;
    };
}

export interface PluginAssistantToolCall {
    id: string;
    type: "function";
    function: {
        name: string;
        arguments: string;
    };
}

export interface PluginAssistantToolResult {
    toolCallId: string;
    name: string;
    content: string;
}

export type PluginAssistantToolExecutor = (
    call: PluginAssistantToolCall,
    args: Record<string, unknown>,
) => Promise<PluginAssistantToolResult> | PluginAssistantToolResult;

type RegisteredPluginAssistantTool = {
    pluginId: string;
    tool: PluginAssistantToolDefinition;
    executor: PluginAssistantToolExecutor;
};

const registeredToolsByName = new Map<string, RegisteredPluginAssistantTool>();
const registeredNamesByPlugin = new Map<string, string[]>();

export function registerPluginAssistantTools(
    pluginId: string,
    tools?: PluginAssistantToolDefinition[],
    executors?: Record<string, PluginAssistantToolExecutor>,
): void {
    if (!tools?.length) {
        return;
    }

    const addedNames: string[] = [];

    for (const tool of tools) {
        const name = tool.function.name.trim();
        if (!name) {
            continue;
        }

        if (registeredToolsByName.has(name)) {
            console.warn(`Assistant tool '${name}' is already registered; skipping duplicate from plugin '${pluginId}'.`);
            continue;
        }

        const executor = executors?.[name];
        if (!executor) {
            console.warn(`Assistant tool '${name}' from plugin '${pluginId}' is missing an executor; skipping registration.`);
            continue;
        }

        registeredToolsByName.set(name, {
            pluginId,
            tool,
            executor,
        });
        addedNames.push(name);
    }

    if (addedNames.length > 0) {
        registeredNamesByPlugin.set(pluginId, addedNames);
    }
}

export function unregisterPluginAssistantTools(pluginId: string): void {
    const names = registeredNamesByPlugin.get(pluginId);
    if (!names?.length) {
        return;
    }

    for (const name of names) {
        registeredToolsByName.delete(name);
    }

    registeredNamesByPlugin.delete(pluginId);
}

export function listPluginAssistantTools(): PluginAssistantToolDefinition[] {
    return Array.from(registeredToolsByName.values(), (entry) => entry.tool);
}

export async function executePluginAssistantTool(
    call: PluginAssistantToolCall,
    args: Record<string, unknown>,
): Promise<PluginAssistantToolResult | null> {
    const registered = registeredToolsByName.get(call.function.name);
    if (!registered) {
        return null;
    }

    return registered.executor(call, args);
}