import type React from "react";
import type { PluginAssistantToolDefinition, PluginAssistantToolExecutor } from "./assistantToolRegistry";
import { registerPluginAssistantTools, unregisterPluginAssistantTools } from "./assistantToolRegistry";
import { ComponentRegistryAPI } from "../registry/componentRegistry";
import { CommandRegistryAPI, type CommandAction } from "../registry/commandRegistry";

export interface Plugin {
  id: string;
  name: string;
  version: string;
  components?: Record<string, React.ComponentType<any>>;
  commands?: Record<string, CommandAction>;
  views?: Record<string, unknown>;
  assistantTools?: PluginAssistantToolDefinition[];
  assistantToolExecutors?: Record<string, PluginAssistantToolExecutor>;
  onLoad?: () => void;
  onUnload?: () => void;
}

export class PluginManager {
  private plugins = new Map<string, Plugin>();

  registerPlugin(plugin: Plugin): void {
    if (this.plugins.has(plugin.id)) {
      console.warn(`Plugin ${plugin.id} already registered`);
      return;
    }

    if (plugin.components) {
      Object.entries(plugin.components).forEach(([name, component]) => {
        ComponentRegistryAPI.register(`${plugin.id}:${name}`, component);
      });
    }

    if (plugin.commands) {
      Object.entries(plugin.commands).forEach(([name, command]) => {
        CommandRegistryAPI.register(`${plugin.id}:${name}`, command);
      });
    }

    registerPluginAssistantTools(plugin.id, plugin.assistantTools, plugin.assistantToolExecutors);

    plugin.onLoad?.();
    this.plugins.set(plugin.id, plugin);
  }

  unregisterPlugin(pluginId: string): void {
    const plugin = this.plugins.get(pluginId);
    if (!plugin) {
      return;
    }

    plugin.onUnload?.();
    unregisterPluginAssistantTools(pluginId);
    this.plugins.delete(pluginId);
  }

  listPlugins(): string[] {
    return Array.from(this.plugins.keys());
  }
}
