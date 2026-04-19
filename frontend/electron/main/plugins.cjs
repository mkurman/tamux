const fs = require('fs');
const path = require('path');

const { ensureTamuxDataDir, readJsonFile } = require('./app-data.cjs');

function unsafeRendererPluginsEnabled(env = process.env) {
    return String(env.TAMUX_ENABLE_UNSAFE_RENDERER_PLUGINS || '').trim() === '1';
}

function getPluginsRootDir() {
    const pluginsDir = path.join(ensureTamuxDataDir(), 'plugins');
    fs.mkdirSync(pluginsDir, { recursive: true });
    return pluginsDir;
}

function normalizeInstalledPluginRecord(entry) {
    if (!entry || typeof entry !== 'object') {
        return null;
    }

    const entryPath = typeof entry.entry_path === 'string' ? entry.entry_path.trim() : '';
    if (!entryPath) {
        return null;
    }

    return {
        packageName: String(entry.package_name || ''),
        packageVersion: String(entry.package_version || ''),
        pluginName: String(entry.plugin_name || entry.package_name || ''),
        entryPath,
        format: String(entry.format || 'script'),
        installedAt: Number(entry.installed_at || 0),
    };
}

function listInstalledPlugins() {
    const registry = readJsonFile('plugins/registry.json');
    const plugins = Array.isArray(registry?.plugins) ? registry.plugins : [];
    return plugins
        .map(normalizeInstalledPluginRecord)
        .filter(Boolean);
}

function resolveInstalledPluginEntryPath(entryPath) {
    const pluginsRoot = path.resolve(getPluginsRootDir());
    const resolvedPath = path.resolve(entryPath);

    if (resolvedPath !== pluginsRoot && !resolvedPath.startsWith(`${pluginsRoot}${path.sep}`)) {
        throw new Error('Installed plugin entry path escapes the tamux plugins directory.');
    }

    return resolvedPath;
}

function loadInstalledPluginScripts() {
    return listInstalledPlugins().map((entry) => {
        try {
            if (entry.format !== 'script') {
                return {
                    packageName: entry.packageName,
                    pluginName: entry.pluginName,
                    entryPath: entry.entryPath,
                    format: entry.format,
                    status: 'skipped',
                    error: `Unsupported plugin format '${entry.format}'`,
                };
            }

            if (!unsafeRendererPluginsEnabled()) {
                return {
                    packageName: entry.packageName,
                    pluginName: entry.pluginName,
                    entryPath: entry.entryPath,
                    format: entry.format,
                    status: 'skipped',
                    error: 'Renderer plugin loading is disabled by default. Set TAMUX_ENABLE_UNSAFE_RENDERER_PLUGINS=1 to opt in.',
                };
            }

            const resolvedEntryPath = resolveInstalledPluginEntryPath(entry.entryPath);
            if (!fs.existsSync(resolvedEntryPath) || !fs.statSync(resolvedEntryPath).isFile()) {
                return {
                    packageName: entry.packageName,
                    pluginName: entry.pluginName,
                    entryPath: entry.entryPath,
                    format: entry.format,
                    status: 'error',
                    error: 'Plugin entry file does not exist.',
                };
            }

            return {
                packageName: entry.packageName,
                pluginName: entry.pluginName,
                entryPath: entry.entryPath,
                format: entry.format,
                sourceUrl: resolvedEntryPath.replace(/\\/g, '/'),
                source: fs.readFileSync(resolvedEntryPath, 'utf8'),
            };
        } catch (error) {
            return {
                packageName: entry.packageName,
                pluginName: entry.pluginName,
                entryPath: entry.entryPath,
                format: entry.format,
                status: 'error',
                error: error?.message ?? String(error),
            };
        }
    });
}

module.exports = {
    getPluginsRootDir,
    listInstalledPlugins,
    loadInstalledPluginScripts,
    unsafeRendererPluginsEnabled,
};
